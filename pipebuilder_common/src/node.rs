use serde::Deserialize;

#[derive(Deserialize)]
pub enum NodeRole {
    Api,
    Builder,
}

#[derive(Deserialize)]
pub struct NodeConfig {
    pub id: String,
    pub role: NodeRole,
    pub internal_address: String,
    pub external_address: Option<String>,
}
