use std::io;

use esl_services_api::types::generic_esl::GenericEsl;
use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};

custom_error! {
 /// An error that can occur while handling pricer Esls.
    ///
    /// This error can be seamlessly converted to an `io::Error` and `reqwest::Error` via a `From`
    /// implementation.
    pub PricerError
        Reqwest{source: reqwest::Error} = "An issue occured within this request: {source}",
        Io{source: io::Error}= "An I/O error occured: {source}",
}
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
struct PricerFishProperties {
    #[serde(rename = "FISH_CALIBRE")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_calibre: Option<String>,
    #[serde(rename = "FISH_ENGIN")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_engin: Option<String>,
    #[serde(rename = "FISH_ENGIN2")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_engin_2: Option<String>,
    #[serde(rename = "FISH_ENGIN3")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_engin_3: Option<String>,
    #[serde(rename = "FISH_INFO")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_info: Option<String>,
    #[serde(rename = "FISH_NAME")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_name: Option<String>,
    #[serde(rename = "FISH_NAME_2")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_name_2: Option<String>,
    #[serde(rename = "FISH_NAME_SCIEN")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_name_scien: Option<String>,
    #[serde(rename = "FISH_ORIGIN")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_origin: Option<String>,
    #[serde(rename = "FISH_ORIGIN2")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_origin_2: Option<String>,
    #[serde(rename = "FISH_PRODUCTION")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_production: Option<String>,
    #[serde(rename = "FISH_SIZE")]
    #[serde_as(as = "NoneAsEmptyString")]

    fish_size: Option<String>,
    #[serde(rename = "PLU")]
    #[serde_as(as = "NoneAsEmptyString")]

    plu: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "PRICE_INFOS")]
    price_infos: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PricerEsl {
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "itemName")]
    item_name: String,
    price: String,
    properties: PricerFishProperties,
}

impl From<GenericEsl> for PricerEsl {
    fn from(value: GenericEsl) -> Self {
        let properties = PricerFishProperties {
            fish_name: Some(value.nom.clone()),
            fish_calibre: None,
            fish_engin: value.engin.map(|engin| format!("Peché: {}", engin)),
            fish_engin_2: None,
            fish_engin_3: None,
            // guessing this is congel infos
            fish_info: value.congel_infos,
            fish_name_2: None,
            fish_name_scien: Some(value.nom_scientifique),

            // if peche: origin= Zone FAO: (zoneCode, sousZoneCode)
            // if peche: origin2=  zoneCode / sousZone
            fish_origin: Some(format!("Zone FAO: ({} {})", value.zone_code.unwrap(), value.sous_zone_code.unwrap())) ,
            fish_origin_2:  Some(format!("{} / {}", value.zone.unwrap(), value.sous_zone.unwrap())),

            fish_production: None,
            fish_size: Some(value.taille),
            plu: Some(value.plu),
            price_infos: Some(value.infos_prix)
        };
        Self {
            item_id: value.id,
            item_name: value.nom,
            price: value.prix,
            properties,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]

struct PricerAccepted {
    #[serde(rename = "requestId")]
    request_id: i32
}
pub async fn update_esl(
    esl: PricerEsl,
    esl_server_url: &str,
    pricer_user: Option<String>,
    pricer_password: Option<String>,
) -> Result<PricerEsl, PricerError> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/public/core/v1/items", esl_server_url);
    let user = pricer_user.expect("Pricer user is empty in the config file, please add 'pricer_user=<user name>' in hublot-config.toml");
    let pwd = pricer_password.expect("Pricer password is empty in the config file, please add 'pricer_password=<password>' in hublot-config.toml");
    let payload = vec![&esl];
    let response = client
        .patch(url)
        .basic_auth(user, Some(pwd))
        .json(&payload)
        .send()
        .await?;
    match response.status() {
        StatusCode::OK | StatusCode::ACCEPTED => {
            let body: PricerAccepted = response.json().await?;
            debug!("Esl server accepted our update")
        }
        _ => {
            debug!("Esl server denied the update: {}", response.status())
        }
    }
    Ok(esl)
}
