use std::io;

use crate::utils::unicode_string;
use esl_utils::generic_esl::GenericEsl;
use log::trace;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

/// The representation of the state of an Electronic Shelf Label
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Esl {
    pub template: String,
    pub sid: String,
    pub back_url: String,
    pub epd_type: String,
    pub block_num: i64,
    pub page: i64,
    pub esl_id: String,
    pub encoding: String,
    #[serde(rename = "prixUnitaire")]
    pub prix_unitaire: f64,
    pub direction: String,
    #[serde(with = "unicode_string")]
    pub name: String,
    #[serde(rename = "itemScientName", with = "unicode_string")]
    pub item_scient_name: String,
    #[serde(rename = "barcodeProduit")]
    pub barcode_produit: String,
    #[serde(rename = "itemName", with = "unicode_string")]
    pub item_name: String,
    #[serde(with = "unicode_string")]
    pub mentiontraca: String,
    #[serde(with = "unicode_string")]
    pub zone: String,
    #[serde(with = "unicode_string")]
    pub souszone: String,
    #[serde(with = "unicode_string")]
    pub engin: String,
    #[serde(with = "unicode_string")]
    pub detailengin: String,
    #[serde(rename = "NameLogo", with = "unicode_string")]
    pub name_logo: String,
    #[serde(with = "unicode_string")]
    pub flagpromo: String,
    #[serde(rename = "flagCongel", with = "unicode_string")]
    pub flag_congel: String,
    #[serde(rename = "TypeArt", with = "unicode_string")]
    pub type_art: String,
    #[serde(rename = "IDEtiquette", with = "unicode_string")]
    pub idetiquette: String,
    pub priority: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrintRequestWrapper {
    pub serial: String,
    pub rid: String,
    pub esl: Esl,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrintRequest {
    pub state: i32,
    pub rid: String,
    pub esl: PrintRequestWrapper,
}

custom_error! {
    /// An error that can occur when during the API.
    ///
    /// This error can be seamlessly converted to an `io::Error` and `reqwest::Error` via a `From`
    /// implementation.
    pub EslServiceError
        Reqwest{source: reqwest::Error} = "An issue occured within this request: {source}",
        Io{source: io::Error}= "An I/O error occured: {source}"
}

/// Fetches the polling route of the provided `server_url` to get the ESLs that we have to print
pub async fn get_print_requests(
    hublot_server_url: &str,
    client: &Client,
    client_serial: &str,
) -> Result<Vec<GenericEsl>, EslServiceError> {
    let url = format!("{}/esl-api/poll/{}", hublot_server_url, client_serial);
    trace!("Fetching esls status: {}", url);
    let response = client.get(url).send().await?;
    let as_json: Vec<GenericEsl> = response.json().await?;
    trace!("Got esl status: {:?}", as_json);
    Ok(as_json)
}

pub async fn status(hublot_server_url: &str, client: &Client) -> Result<bool, EslServiceError> {
    let url = format!("{}/esl-api/status", hublot_server_url);
    let response = client.get(url).send().await?;
    Ok(response.status() == StatusCode::OK)
}
