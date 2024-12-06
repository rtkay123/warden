pub mod handler;
pub mod server;
pub mod state;

use anyhow::Result;

pub async fn run() -> Result<()> {
    server::serve().await
}
