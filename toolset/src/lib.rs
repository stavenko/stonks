use std::{borrow::Cow, time::Duration};

use serde::{de, Deserialize, Deserializer};

pub fn deser_duration_from_integer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    let number = <u64>::deserialize(deserializer)?;
    Ok(Duration::from_millis(number))
}

pub fn deser_float_from_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<f64, D::Error> {
    let string_value = Cow::<str>::deserialize(deserializer)?;
    string_value.as_ref().parse().map_err(de::Error::custom)
}
