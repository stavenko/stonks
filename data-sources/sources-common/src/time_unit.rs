use std::{str::FromStr, borrow::Cow, time::Duration};

use serde::{Deserialize, Serializer, Deserializer, de};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TimeUnit(#[serde(deserialize_with = "deser_time_unit_u32_from_string")] u32);


pub static SECOND: u32 = 1;
pub static MINUTE: u32 = 60 * SECOND;
pub static HOUR: u32 = 60 * MINUTE;
pub static DAY: u32 = 24 * HOUR;
pub static WEEK: u32 = 7 * DAY;

impl TimeUnit {
    pub fn secs(v: u32) -> Self {
        TimeUnit(v)
    }

    pub fn calc_n(&self, v:u32) -> Duration {
        let secs = (self.0 as u64) * (v as u64);
        Duration::from_secs(secs)

    }

    pub fn mins(v: u32) -> Self {
        TimeUnit(v * 60)
    }

    pub fn hours(v: u32) -> Self {
        TimeUnit(v * 60 * 60)
    }

    pub fn days(v: u32) -> Self {
        TimeUnit(v * 60 * 60 * 24)
    }

    pub fn weeks(v: u32) -> Self {
        TimeUnit(v * 60 * 60 * 24 * 7)
    }

    pub fn months(v: u32) -> Self {
        TimeUnit(v * 60 * 60 * 24 * 7 * 4) // maybe
    }

    pub fn fmt(&self) -> String {
        match self.0 {
            v if SECOND == v => "1s".to_string(),
            v if 5 * SECOND == v => "5s".to_string(),
            v if MINUTE == v => "1m".to_string(),
            v if 3 * MINUTE == v => "3m".to_string(),
            v if 5 * MINUTE == v => "5m".to_string(),
            v if 15 * MINUTE == v => "15m".to_string(),
            v if 30 * MINUTE == v => "30m".to_string(),
            v if 1 * HOUR == v => "1h".to_string(),
            v if 2 * HOUR == v => "2h".to_string(),
            v if 4 * HOUR == v => "4h".to_string(),
            v if 6 * HOUR == v => "6h".to_string(),
            v if 8 * HOUR == v => "8h".to_string(),
            v if 12 * HOUR == v => "12h".to_string(),
            v if 1 * DAY == v => "1d".to_string(),
            v if 3 * DAY == v => "3d".to_string(),
            v if 1 * WEEK == v => "1w".to_string(),
            v if 4 * WEEK == v => "1M".to_string(),
            _ => unreachable!("incorrect time_unit: {}", self.0),
        }
    }
}

impl FromStr for TimeUnit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1s" => Ok(TimeUnit::secs(1)),
            "1m" => Ok(TimeUnit::mins(1)),
            "3m" => Ok(TimeUnit::mins(3)),
            "5m" => Ok(TimeUnit::mins(5)),
            "15m" => Ok(TimeUnit::mins(15)),
            "30m" => Ok(TimeUnit::mins(30)),
            "1h" => Ok(TimeUnit::hours(1)),
            "2h" => Ok(TimeUnit::hours(2)),
            "4h" => Ok(TimeUnit::hours(4)),
            "6h" => Ok(TimeUnit::hours(6)),
            "8h" => Ok(TimeUnit::hours(8)),
            "12h" => Ok(TimeUnit::hours(12)),
            "1d" => Ok(TimeUnit::days(1)),
            "3d" => Ok(TimeUnit::days(3)),
            "1w" => Ok(TimeUnit::weeks(1)),
            "1M" => Ok(TimeUnit::months(1)),
            _ => Err(format!("Incorrect time_unit: {s}")),
        }
    }
}

pub fn ser_time_unit<S: Serializer>(time_unit: &TimeUnit, serializer: S) -> Result<S::Ok, S::Error> {
    let value = time_unit.fmt();
    serializer.serialize_str(&value)
}

pub fn deser_time_unit_from_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<TimeUnit, D::Error> {
    let string_value = Cow::<str>::deserialize(deserializer)?;
    TimeUnit::from_str(&string_value).map_err(de::Error::custom)
}

pub fn deser_time_unit_u32_from_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    let string_value = Cow::<str>::deserialize(deserializer)?;
    TimeUnit::from_str(&string_value)
        .map_err(de::Error::custom)
        .map(|tu| tu.0)
}
