use std::io;

use super::pricer::{item::update_item, labels::map_esl_to_id, status::items_result};
use esl_utils::generic_esl::GenericEsl;
use indicatif::ProgressBar;
use log::debug;
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
    // #[serde_as(as = "NoneAsEmptyString")]
    fish_engin: Option<String>,
    #[serde(rename = "FISH_ENGIN_2")]
    // #[serde_as(as = "NoneAsEmptyString")]
    fish_engin_2: Option<String>,
    #[serde(rename = "FISH_ENGIN_3")]
    // #[serde_as(as = "NoneAsEmptyString")]
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
    #[serde(rename = "ALLERGENES")]
    #[serde_as(as = "NoneAsEmptyString")]
    allergenes: Option<String>,
    #[serde(rename = "PROMO")]
    #[serde_as(as = "NoneAsEmptyString")]
    promo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PricerEsl {
    #[serde(rename = "eslId")]
    pub barcode: String,
    #[serde(alias = "objectId", rename = "itemId")]
    pub item_id: String,
    #[serde(rename = "itemName")]
    pub item_name: String,
    /// price can be None because in some case
    /// it will be filled by an other software
    pub price: Option<String>,
    pub properties: PricerFishProperties,
}

impl From<GenericEsl> for PricerEsl {
    fn from(value: GenericEsl) -> Self {
        let properties = PricerFishProperties {
            fish_name: Some(value.nom.clone()),
            fish_calibre: None,
            /// origin = the product was not fished therefore there is no fishing gear
            fish_engin: if value.origine.is_some() {
                None
            } else {
                value.engin
            },
            fish_engin_2: None,
            fish_engin_3: None,
            // guessing this is congel infos
            fish_info: value.congel_infos,
            fish_name_2: None,
            fish_name_scien: Some(value.nom_scientifique),
            // if peche: origin= Zone FAO: (zoneCode, sousZoneCode)
            // if peche: origin2=  zoneCode / sousZone
            fish_origin: Some(value.origine.unwrap_or(value.zone.unwrap_or_default())),
            fish_origin_2: Some(value.sous_zone.unwrap_or_default()),

            fish_production: value.production,
            fish_size: Some(value.taille),
            plu: Some(value.plu),
            allergenes: value.allergenes,
            promo: None,
        };
        Self {
            item_id: value.object_id.unwrap(),
            barcode: value.id,
            item_name: value.nom,
            price: None,
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

pub async fn on_poll(
    esl: PricerEsl,
    esl_server_url: &str,
    pricer_user: String,
    pricer_password: String,
    pb: &ProgressBar,
) -> Result<PricerEsl, PricerError> {
    //first: We need to map the esl barcode to a pricer item_id
    pb.inc(1);
    pb.set_message(format!("[1/3] Getting items for esl id {}", esl.barcode));
    let mapped_esl = map_esl_to_id(
        esl,
        esl_server_url,
        pricer_user.clone(),
        pricer_password.clone(),
    )
    .await?;

    debug!("Got mapped ESL: {:?}", mapped_esl);
    pb.inc(1);
    pb.set_message(format!("[2/3] Updating item id {}", mapped_esl.item_id));
    // then we can request pricer to update the item with the matching id
    let update_request = update_item(
        mapped_esl.clone(),
        esl_server_url,
        pricer_user.clone(),
        pricer_password.clone(),
    )
    .await?;
    debug!("Got request status: {:?}", update_request);
    pb.inc(1);
    pb.set_message(format!(
        "[3/3] Checking update status for request_id {}",
        update_request.request_id
    ));
    // then we can send the request id back to the api
    let update_status = items_result(
        update_request,
        esl_server_url,
        pricer_user.clone(),
        pricer_password.clone(),
    )
    .await?;
    debug!("Got update_status {:?}", update_status);

    Ok(mapped_esl)
}
