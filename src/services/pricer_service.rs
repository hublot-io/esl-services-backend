use std::io;

use super::esl_service::Esl;
use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

custom_error! {
 /// An error that can occur while handling pricer Esls.
    ///
    /// This error can be seamlessly converted to an `io::Error` and `reqwest::Error` via a `From`
    /// implementation.
    pub PricerError
        Reqwest{source: reqwest::Error} = "An issue occured within this request: {source}",
        Io{source: io::Error}= "An I/O error occured: {source}",
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PricerFishProperties {
    #[serde(rename = "FISH_CALIBRE")]
    fish_calibre: Option<String>,
    #[serde(rename = "FISH_ENGIN")]
    fish_engin: Option<String>,
    #[serde(rename = "FISH_ENGIN_2")]
    fish_engin_2: Option<String>,
    #[serde(rename = "FISH_ENGIN_3")]
    fish_engin_3: Option<String>,
    #[serde(rename = "FISH_INFO")]
    fish_info: Option<String>,
    #[serde(rename = "FISH_NAME")]
    fish_name: Option<String>,
    #[serde(rename = "FISH_NAME_2")]
    fish_name_2: Option<String>,
    #[serde(rename = "FISH_NAME_SCIEN")]
    fish_name_scien: Option<String>,
    #[serde(rename = "FISH_ORIGIN")]
    fish_origin: Option<String>,
    #[serde(rename = "FISH_ORIGIN_2")]
    fish_origin_2: Option<String>,
    #[serde(rename = "FISH_PRODUCTION")]
    fish_production: Option<String>,
    #[serde(rename = "FISH_SIZE")]
    fish_size: Option<String>,
    #[serde(rename = "PLU")]
    plu: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PricerEsl {
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "itemName")]
    item_name: String,
    #[serde(rename = "presentation")]
    presentation: String,
    properties: PricerFishProperties,
}

pub fn from_esl(esl: Esl) -> PricerEsl {
    let properties = PricerFishProperties {
        fish_name: Some(esl.name.clone()),
        fish_calibre: None,
        fish_engin: Some(esl.engin),
        fish_engin_2: None,
        fish_engin_3: None,
        fish_info: None,
        fish_name_2: None,
        fish_name_scien: Some(esl.item_scient_name),
        fish_origin: None,
        fish_origin_2: None,
        fish_production: None,
        fish_size: None,
        plu: None,
    };

    let pricer_esl = PricerEsl {
        item_id: esl.esl_id,
        item_name: esl.name,
        presentation: "".to_string(),
        properties,
    };

    pricer_esl
}

pub async fn update_esl(
    esl: PricerEsl,
    esl_server_url: &str,
    pricer_user: Option<String>,
    pricer_password: Option<String>,
) -> Result<PricerEsl, PricerError> {
    let client = reqwest::Client::new();

    let url = format!("{} /api/public/core/v1/items", esl_server_url);
    let response = client.patch(url).json(&esl).send().await?;

    match response.status() {
        StatusCode::OK => {
            debug!("Esl server accepted our update")
        }
        _ => {
            debug!("Esl server denied the update: {}", response.status())
        }
    }
    Ok(esl)
}
