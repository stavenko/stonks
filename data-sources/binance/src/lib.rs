
pub mod error;
pub mod protocol;
pub mod serde_utils;
pub mod spot;
pub mod fut;

pub trait ToChannel {
    fn to_channel(&self) -> String;
}

