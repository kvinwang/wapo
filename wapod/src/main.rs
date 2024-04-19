use anyhow::Context;
use clap::Parser;
use tracing::info;
use web_api::crate_app;

mod web_api;

#[derive(Parser)]
#[clap(about = "wapo - a WASM runtime", version, author)]
pub struct Args {
    #[arg(long, default_value_t = 1)]
    workers: usize,
    /// Max memory pages
    #[arg(long, default_value_t = 256)]
    max_memory_pages: u32,
    /// Max number of instances to run
    #[arg(long, default_value_t = 8)]
    max_instances: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let app = crate_app(Args::parse());
    let admin_service = web_api::serve_admin(app.clone());
    let user_service = async move {
        // Wait for the admin service to start
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        web_api::serve_user(app).await
    };
    tokio::select! {
        result = user_service => {
            result.context("User service terminated")?;
        },
        result = admin_service => {
            result.context("Admin service terminated")?;
        },
    }
    info!("Server exited.");
    Ok(())
}
