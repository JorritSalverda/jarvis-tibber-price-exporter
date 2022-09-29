use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotPriceResponse {
    pub data: SpotPriceData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotPriceData {
    pub viewer: SpotPriceViewer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotPriceViewer {
    pub homes: Vec<SpotPriceHome>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotPriceHome {
    pub current_subscription: SpotPriceSubscription,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotPriceSubscription {
    pub price_info: SpotPriceInfo,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotPriceInfo {
    pub today: Vec<SpotPricePrice>,
    pub tomorrow: Vec<SpotPricePrice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotPricePrice {
    pub energy: f64,
    pub tax: f64,
    pub currency: String,
    pub starts_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotPrice {
    pub id: Option<String>,
    pub source: Option<String>,
    pub from: DateTime<Utc>,
    pub till: DateTime<Utc>,
    pub market_price: f64,
    pub market_price_tax: f64,
    pub sourcing_markup_price: f64,
    pub energy_tax_price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub future_spot_prices: Vec<SpotPrice>,
    pub last_from: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fs;

    #[test]
    fn deserialize_spot_price_response() -> Result<(), Box<dyn Error>> {
        let spot_price_predictions_content = fs::read_to_string("spot_price_predictions.json")?;

        let spot_price_response: SpotPriceResponse =
            serde_json::from_str(&spot_price_predictions_content)?;

        assert_eq!(
            spot_price_response.data.viewer.homes[0]
                .current_subscription
                .price_info
                .today
                .len(),
            24
        );
        assert_eq!(
            spot_price_response.data.viewer.homes[0]
                .current_subscription
                .price_info
                .tomorrow
                .len(),
            0
        );
        Ok(())
    }
}
