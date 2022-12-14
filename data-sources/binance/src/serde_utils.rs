use std::time::Duration;

use serde::{Deserialize, Deserializer};

pub fn deser_duration_from_integer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    let number = <u64>::deserialize(deserializer)?;
    Ok(Duration::from_millis(number))
}

