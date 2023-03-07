use log::debug;
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};

use crate::services::pricer_service::PricerError;

use super::item::PricerAccepted;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PricerItemResult{
    #[serde(rename = "itemId")]
    item_id: String,
    status: String,
    errors: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PricerItemsResult {
    status: String,
    #[serde(rename = "itemResults")]
    item_results: Vec<PricerItemResult>
}

pub async fn items_result(
    request_status: PricerAccepted,
    esl_server_url: &str,
    pricer_user: String,
    pricer_password: String,
) -> Result<PricerItemsResult, PricerError> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/public/core/v1/items-result/{}", esl_server_url, request_status.request_id);
    let response = client
        .get(url)
        .basic_auth(pricer_user, Some(pricer_password))
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => {
            let body: PricerItemsResult = response.json().await?;
            debug!("Esl server accepted our update");
            Ok(body)
        }
        reqwest_error => {
            debug!("Esl server denied the update: {}", response.status());
            Err(PricerError::MissingItem)
        }
    }
}