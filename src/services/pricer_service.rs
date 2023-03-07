use std::io;

use esl_utils::generic_esl::GenericEsl;
use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};

use super::pricer::{item::update_item, labels::map_esl_to_id, status::items_result};

custom_error! {
    /// An error that can occur while handling pricer Esls.
    ///
    /// This error can be seamlessly converted to an `io::Error` and `reqwest::Error` via a `From`
    /// implementation.
    pub PricerError
        Reqwest{source: reqwest::Error} = "An issue occured within this request: {source}",
        Io{source: io::Error}= "An I/O error occured: {source}",
        MissingItem = "Cannot find an item linked to this barcode",
        UpdateFailed{id: String} = "PricerError, cannot update this item: {id}"
}
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PricerFishProperties {
    #[serde(rename = "FISH_CALIBRE")]
    #[serde_as(as = "NoneAsEmptyString")]
    fish_calibre: Option<String>,
    #[serde(rename = "FISH_ENGIN")]
    #[serde_as(as = "NoneAsEmptyString")]
    fish_engin: Option<String>,
    #[serde(rename = "FISH_ENGIN_2")]
    #[serde_as(as = "NoneAsEmptyString")]
    fish_engin_2: Option<String>,
    #[serde(rename = "FISH_ENGIN_3")]
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
    #[serde(rename = "FISH_ORIGIN_2")]
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
    price_infos: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PricerEsl {
    #[serde(rename = "eslId")]
    pub barcode: String,
    #[serde(rename = "objectId")]
    pub item_id: String,
    #[serde(rename = "itemName")]
    pub item_name: String,
    pub price: String,
    pub properties: PricerFishProperties,
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
            fish_origin: Some(format!(
                "Zone FAO: ({} {})",
                value.zone_code.unwrap(),
                value.sous_zone_code.unwrap()
            )),
            fish_origin_2: Some(format!(
                "{} / {}",
                value.zone.unwrap(),
                value.sous_zone.unwrap()
            )),

            fish_production: None,
            fish_size: Some(value.taille),
            plu: Some(value.plu),
            price_infos: Some(value.infos_prix),
        };
        Self {
            item_id: value.object_id.unwrap(),
            barcode: value.id,
            item_name: value.nom,
            price: value.prix,
            properties,
        }
    }
}







// pub async fn check_status(
//     esl: PricerEsl,
//     esl_server_url: &str,
//     pricer_user: String,
//     pricer_password: String
// )-> Result<PricerEsl, PricerError> {}

pub async fn on_poll(esl: PricerEsl,
    esl_server_url: &str,
    pricer_user: String,
    pricer_password: String,
) -> Result<PricerEsl, PricerError> {
    //first: We need to map the esl barcode to a pricer item_id
    let mapped_esl = map_esl_to_id(esl, esl_server_url, pricer_user.clone(), pricer_password.clone()).await?;

    // then we can request pricer to update the item with the matching id
    let update_request = update_item(mapped_esl.clone(), esl_server_url, pricer_user.clone(), pricer_password.clone()).await?;

    // then we can send the request id back to the api
    let update_status = items_result(update_request, esl_server_url,pricer_user.clone(),pricer_password.clone()).await?;

    Ok(mapped_esl)
}