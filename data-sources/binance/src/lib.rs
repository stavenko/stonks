pub mod error;
pub mod fut;
pub mod protocol;
pub mod spot;

pub trait ToChannel {
    fn to_channel(&self) -> String;
}
