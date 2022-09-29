use crate::types::{SpotPrice, SpotPriceResponse};
use chrono::Duration;
use log::debug;
use std::env;
use std::error::Error;

pub struct TibberClientConfig {
    access_token: String,
}

impl TibberClientConfig {
    pub fn new(access_token: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            access_token: access_token.to_string(),
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let access_token = env::var("TIBBER_ACCESS_TOKEN")?;

        Self::new(&access_token)
    }
}

pub struct TibberClient {
    config: TibberClientConfig,
}

impl TibberClient {
    pub fn new(config: TibberClientConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self::new(TibberClientConfig::from_env()?))
    }

    pub async fn get_spot_prices(&self) -> Result<Vec<SpotPrice>, Box<dyn Error>> {
        let request_body = r#"{
              viewer {
                homes {
                  currentSubscription {
                    priceInfo {
                      today {
                        energy
                        tax
                        currency
                        startsAt
                      }
                      tomorrow {
                        energy
                        tax
                        currency
                        startsAt
                      }
                    }
                  }
                }
              }
            }"#;

        debug!("request body:\n{}", request_body);

        let response = reqwest::Client::new()
            .post("https://api.tibber.com/v1-beta/gql")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.access_token),
            )
            .body(request_body)
            .send()
            .await?;

        let status_code = response.status();
        debug!("response status: {}", status_code);

        let response_body = response.text().await?;
        debug!("response body:\n{}", response_body);

        if !status_code.is_success() {
            return Err(Box::<dyn Error>::from(format!(
                "Status code {} indicates failure",
                status_code
            )));
        }

        let spot_price_response = serde_json::from_str::<SpotPriceResponse>(&response_body)?;

        let mut spot_prices: Vec<SpotPrice> = vec![];

        for spot_price in &spot_price_response.data.viewer.homes[0]
            .current_subscription
            .price_info
            .today
        {
            spot_prices.push(SpotPrice {
                id: None,
                source: None,
                from: spot_price.starts_at,
                till: spot_price.starts_at + Duration::hours(1),
                market_price: spot_price.energy,
                market_price_tax: spot_price.tax,
                sourcing_markup_price: 0.0,
                energy_tax_price: 0.0,
            })
        }

        for spot_price in &spot_price_response.data.viewer.homes[0]
            .current_subscription
            .price_info
            .tomorrow
        {
            spot_prices.push(SpotPrice {
                id: None,
                source: None,
                from: spot_price.starts_at,
                till: spot_price.starts_at + Duration::hours(1),
                market_price: spot_price.energy,
                market_price_tax: spot_price.tax,
                sourcing_markup_price: 0.0,
                energy_tax_price: 0.0,
            })
        }

        Ok(spot_prices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn get_spot_prices() {
        let spot_price_client = TibberClient::from_env().expect("Failed creating TibberClient");

        // act
        let spot_prices = spot_price_client
            .get_spot_prices()
            .await
            .expect("Failed retrieving spot prices");

        assert_eq!(spot_prices.len(), 24);
    }
}
