use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Response<T> {
    Success(T),
    Error { code: i64, msg: String },
}

#[derive(Serialize, Debug)]
pub struct SubscribeMessage {
    pub method: String,
    pub params: Vec<String>,
    pub id: u32,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum StreamData {
    SubscribeResponse {
        response: Option<serde_json::Value>,
        id: u32,
    },
    Package(StreamPackage),
}

#[derive(Debug, Deserialize)]
pub struct StreamPackage {
    pub stream: String,
    #[serde(rename = "data")]
    pub event: Event,
}

#[derive(Deserialize, Debug)]
pub struct Event {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(flatten)]
    data: serde_json::Map<String, serde_json::Value>,
}

impl Event {
    pub fn data(self) -> serde_json::Value {
        serde_json::Value::Object(self.data)
    }
}

#[cfg(test)]
mod test {


    use crate::spot::orderbook::WsOrderBook;

    use super::StreamData;

    #[test]
    fn partial_deser() {
        let raw_msg = r#"{
            "stream":"ethusdt@depth",
            "data":{
                "e":"depthUpdate",
                "E":1670329901651,
                "s":"ETHUSDT",
                "U":22391449165,
                "u":22391449377,
                "b":[["1225.79000000","0.18790000"]],
                "a":[["1974.05000000","0.25110000"]]
            }
        }"#;

        let sd: StreamData = serde_json::from_str(raw_msg).unwrap();
        let StreamData::Package(package) = sd else {
            panic!("nope");
        };

        assert_eq!(package.event.event_type, "depthUpdate");

        let ob: WsOrderBook = serde_json::from_value(package.event.data()).unwrap();

        assert_eq!(ob.asks.len(), 1)
    }
}
