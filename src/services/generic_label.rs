use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EslType {
    Hanshow,
    Pricer,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenericEsl {
    pub r#type: EslType,
    /// The ESL  id.
    ///
    /// It can be either a long string randomly generated for Hanshow or
    /// a manually set id for Pricer
    pub id: String,
    pub nom: String,
    pub nom_scientifique: String,
    pub prix: String,
    pub engin: String,
    pub zone: String,
    pub sous_zone: String,
    pub plu: String,
    pub taille: String,
    pub congel_infos: Option<String>,
    pub origine: Option<String>, // #[serde(rename = "itemId")]
                                 // item_id: String,
                                 // #[serde(rename = "itemName")]
                                 // item_name: String,
                                 // #[serde(rename = "presentation")]
                                 // presentation: String,
                                 // properties: PricerFishProperties,
}
