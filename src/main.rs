mod bigquery_client;
mod exporter_service;
mod state_client;
mod tibber_client;
mod types;

use bigquery_client::BigqueryClient;
use exporter_service::ExporterService;
use state_client::StateClient;
use std::error::Error;
use tibber_client::TibberClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let bigquery_client = BigqueryClient::from_env().await?;
    let tibber_client = TibberClient::from_env()?;
    let state_client = StateClient::from_env().await?;

    let exporter_service = ExporterService::from_env(bigquery_client, tibber_client, state_client)?;

    exporter_service.run().await
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
