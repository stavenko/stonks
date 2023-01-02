#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Symbol {
    pub ticker: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub source: String,
}
