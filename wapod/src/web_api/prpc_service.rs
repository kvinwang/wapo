use rocket::{
    data::{ByteUnit, Limits, ToByteUnit as _},
    http::{ContentType, Status},
    response::status::Custom,
    Data, State,
};
use rpc::prpc::{
    self,
    admin_server::Admin,
    server::{Error as RpcError, Service as PrpcService},
    user_server::User,
    NodeInfo,
};
use tracing::{error, info};
use wapod_rpc as rpc;

type Result<T, E = RpcError> = std::result::Result<T, E>;

use super::{read_data, App};

impl Admin for App {
    async fn info(&self, _request: ()) -> Result<NodeInfo> {
        Ok(App::info(self).await)
    }
}

impl User for App {
    async fn info(&self, _request: ()) -> Result<NodeInfo> {
        Admin::info(self, _request).await
    }
}

pub async fn handle_prpc<S>(
    app: &State<App>,
    method: String,
    data: Option<Data<'_>>,
    limits: &Limits,
    content_type: Option<&ContentType>,
    json: bool,
) -> Result<Vec<u8>, Custom<Vec<u8>>>
where
    S: From<App> + PrpcService,
{
    let data = match data {
        Some(data) => {
            let limit = limit_for_method(&method, limits);
            read_data(data, limit).await?
        }
        None => vec![],
    };
    let json = json || content_type.map(|t| t.is_json()).unwrap_or(false);
    let app = (*app).clone();
    let data = data.to_vec();
    let result = dispatch_prpc(method, data, json, S::from(app)).await;
    let (status_code, output) = result;
    if status_code == 200 {
        Ok(output)
    } else {
        let custom = if let Some(status) = Status::from_code(status_code) {
            Custom(status, output)
        } else {
            error!(status_code, "prpc: Invalid status code!");
            Custom(Status::ServiceUnavailable, vec![])
        };
        Err(custom)
    }
}

fn limit_for_method(method: &str, limits: &Limits) -> ByteUnit {
    if let Some(v) = limits.get(method) {
        return v;
    }
    10.mebibytes()
}

async fn dispatch_prpc(
    path: String,
    data: Vec<u8>,
    json: bool,
    server: impl PrpcService,
) -> (u16, Vec<u8>) {
    use rpc::prpc::server::{Error, ProtoError};

    info!("Dispatching request: {}", path);
    let result = server.dispatch_request(&path, data, json).await;
    let (code, data) = match result {
        Ok(data) => (200, data),
        Err(err) => {
            error!("Rpc error: {:?}", err);
            let (code, err) = match err {
                Error::NotFound => (404, ProtoError::new("Method Not Found")),
                Error::DecodeError(err) => (400, ProtoError::new(format!("DecodeError({err:?})"))),
                Error::AppError(msg) => (500, ProtoError::new(msg)),
                Error::ContractQueryError(msg) => (500, ProtoError::new(msg)),
            };
            if json {
                let error = format!("{err:?}");
                let body = serde_json::to_string_pretty(&serde_json::json!({ "error": error }))
                    .unwrap_or_else(|_| r#"{"error": "Failed to encode the error"}"#.to_string())
                    .into_bytes();
                (code, body)
            } else {
                (code, prpc::codec::encode_message_to_vec(&err))
            }
        }
    };
    (code, data)
}