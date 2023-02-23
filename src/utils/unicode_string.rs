use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(str: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = unidecode::unidecode(str);
    serializer.serialize_str(&s)
}

// The signature of a deserialize_with function must follow the pattern:
//
//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//    where
//        D: Deserializer<'de>
//
// although it may also be generic over the output types T.
pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
}
