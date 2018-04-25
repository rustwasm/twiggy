use ir::Id;

#[derive(Debug, Default, Serialize)]
pub struct CsvRecord {
    pub name: String,
    pub shallow_size: u32,
    pub shallow_size_percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_size_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_dominator: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>
}