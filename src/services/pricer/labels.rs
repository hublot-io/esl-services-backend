use log::debug;
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};

use crate::services::pricer_service::{PricerEsl, PricerError};


#[derive(Serialize, Deserialize, Clone, Debug)]
struct PricerLinks {
    barcode: String,
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "displayPosition")]
    display_position: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PricerLabels {
    barcode: String,
    #[serde(rename = "modelName")]
    model_name: String,
    links: Vec<PricerLinks>,
}

/// Returns the item_id linked to the esl_id
pub async fn map_esl_to_id(
    esl: PricerEsl,
    esl_server_url: &str,
    pricer_user: String,
    pricer_password: String,
) -> Result<PricerEsl, PricerError> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/public/core/v1/labels/{}",
        esl_server_url, esl.barcode
    );
    let response = client.get(url).basic_auth(pricer_user, Some(pricer_password)).send().await?;

    match response.status() {
        StatusCode::OK => {
            let body: PricerLabels = response.json().await?;
            // Default implem: use the first item linked to this barcode
            match body.links.get(0) {
                Some(link) => {
                    let id = &link.item_id;
                    let mut updated = esl.clone();
                    updated.item_id = id.clone();
                    Ok(updated)
                }
                None => Err(PricerError::MissingItem)
            }
        }
        _ => {
            debug!("No matching items found: {}", response.status());
            Err(PricerError::MissingItem)
        }
    }
}
