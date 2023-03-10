use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::services::pricer_service::{PricerError, PricerEsl};

#[derive(Serialize, Deserialize, Clone, Debug)]

pub struct PricerAccepted {
    #[serde(rename = "requestId")]
    pub request_id: i32,
}

pub async fn update_item(
    esl: PricerEsl,
    esl_server_url: &str,
    pricer_user: String,
    pricer_password: String,
) -> Result<PricerAccepted, PricerError> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/public/core/v1/items", esl_server_url);
    let payload = vec![&esl];
    let response = client
        .patch(url)
        .basic_auth(pricer_user, Some(pricer_password))
        .json(&payload)
        .send()
        .await?;
    match response.status() {
        StatusCode::OK | StatusCode::ACCEPTED => {
            let body: PricerAccepted = response.json().await?;
            debug!("Esl server accepted our update");
            Ok(body)
        }
        _reqwest_error => {
            debug!("Esl server denied the update: {}", response.status());
            Err(PricerError::UpdateFailed { id: esl.item_id })
        }
    }
}
