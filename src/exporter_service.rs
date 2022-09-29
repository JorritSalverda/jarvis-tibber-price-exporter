use crate::bigquery_client::BigqueryClient;
use crate::state_client::StateClient;
use crate::tibber_client::TibberClient;
use crate::types::*;
use chrono::{DateTime, Utc};
use log::info;
use std::env;
use std::error::Error;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;
use uuid::Uuid;

pub struct ExporterServiceConfig {
    bigquery_client: BigqueryClient,
    tibber_client: TibberClient,
    state_client: StateClient,
    source: String,
}

impl ExporterServiceConfig {
    pub fn new(
        bigquery_client: BigqueryClient,
        tibber_client: TibberClient,
        state_client: StateClient,
        source: &str,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            bigquery_client,
            tibber_client,
            state_client,
            source: source.to_string(),
        })
    }

    pub fn from_env(
        bigquery_client: BigqueryClient,
        tibber_client: TibberClient,
        state_client: StateClient,
    ) -> Result<Self, Box<dyn Error>> {
        let source = env::var("SOURCE")?;

        Self::new(bigquery_client, tibber_client, state_client, &source)
    }
}

pub struct ExporterService {
    config: ExporterServiceConfig,
}

impl ExporterService {
    pub fn new(config: ExporterServiceConfig) -> Self {
        Self { config }
    }

    pub fn from_env(
        bigquery_client: BigqueryClient,
        tibber_client: TibberClient,
        state_client: StateClient,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self::new(ExporterServiceConfig::from_env(
            bigquery_client,
            tibber_client,
            state_client,
        )?))
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let now: DateTime<Utc> = Utc::now();

        info!("Initalizing BigQuery table...");
        self.config.bigquery_client.init_table().await?;

        info!("Reading previous state...");
        let state = self.config.state_client.read_state()?;

        info!("Retrieving day-ahead prices...");
        let spot_prices = Retry::spawn(
            ExponentialBackoff::from_millis(100).map(jitter).take(3),
            || self.config.tibber_client.get_spot_prices(),
        )
        .await?;

        info!("Retrieved {} day-ahead prices", spot_prices.len());

        info!("Storing retrieved day-ahead prices...");
        let mut future_spot_prices: Vec<SpotPrice> = vec![];
        let mut last_from: Option<DateTime<Utc>> = None;
        for spot_price in &spot_prices {
            let spot_price = SpotPrice {
                id: Some(Uuid::new_v4().to_string()),
                source: Some(self.config.source.clone()),
                ..spot_price.clone()
            };

            info!("{:?}", spot_price);
            if spot_price.till > now {
                future_spot_prices.push(spot_price.clone());
            }

            let write_spot_price = if let Some(st) = &state {
                spot_price.from > st.last_from
            } else {
                true
            };

            if write_spot_price {
                Retry::spawn(
                    ExponentialBackoff::from_millis(100).map(jitter).take(3),
                    || self.config.bigquery_client.insert_spot_price(&spot_price),
                )
                .await?;
                last_from = Some(spot_price.from);
            } else {
                info!("Skipping writing to BigQuery, already present");
            }
        }

        if last_from.is_some() {
            info!("Writing new state...");
            let new_state = State {
                future_spot_prices,
                last_from: last_from.unwrap(),
            };

            self.config.state_client.store_state(&new_state).await?;
        }

        Ok(())
    }
}
