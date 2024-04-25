use rocket::{
    data::{ByteUnit, Limits, ToByteUnit as _},
    http::{ContentType, Status},
    response::status::Custom,
    Data, State,
};
use rpc::prpc::{
    self as pb,
    admin_server::Admin,
    server::{Error as RpcError, Service as PrpcService},
    user_server::User,
    NodeInfo,
};
use tracing::{error, info, warn};
use wapod_rpc as rpc;

use wapo_host::{
    objects::{get_object, put_object},
    service::Command,
    ShortId,
};

type Result<T, E = RpcError> = std::result::Result<T, E>;

use super::{read_data, App};

impl Admin for App {
    async fn info(&self, _request: ()) -> Result<NodeInfo> {
        Ok(App::info(self).await)
    }

    async fn put_object(&self, request: pb::Object) -> Result<()> {
        let objects_path = self.objects_path().await;
        put_object(
            &objects_path,
            &request.hash,
            &mut &request.body[..],
            &request.hash_algrithm,
        )
        .await
        .map_err(|err| {
            warn!("Failed to put object: {err}");
            RpcError::AppError(format!("Failed to put object: {err}"))
        })
    }

    async fn upload_manifest(&self, request: pb::Manifest) -> Result<pb::Address> {
        let objects_path = self.objects_path().await;
        let wasm_blob = get_object(&objects_path, &request.code_hash, &request.hash_algorithm)
            .map_err(|err| {
                warn!("Failed to get wasm object: {err}");
                RpcError::AppError(format!("Failed to get wasm object: {err}"))
            })?
            .ok_or(RpcError::AppError("Wasm object not found".to_string()))?;
        let address = sp_core::blake2_256(&scale::Encode::encode(&request));
        let vmid = ShortId(address);
        let fixme = "Race condition here";
        if let Some(handle) = self.take_handle(address).await {
            info!("Stopping VM {vmid}...");
            if let Err(err) = handle.sender.send(Command::Stop).await {
                warn!("Failed to send stop command to the VM: {err:?}");
            }
            match handle.handle.await {
                Ok(reason) => info!("VM exited: {reason:?}"),
                Err(err) => warn!("Failed to wait VM exit: {err:?}"),
            }
        };
        self.run_wasm(wasm_blob, 0, address).await.map_err(|err| {
            warn!("Failed to run wasm: {err}");
            RpcError::AppError(format!("Failed to run wasm: {err}"))
        })?;
        Ok(pb::Address {
            address: address.to_vec(),
        })
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
                (code, pb::codec::encode_message_to_vec(&err))
            }
        }
    };
    (code, data)
}
