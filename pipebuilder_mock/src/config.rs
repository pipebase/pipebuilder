use serde::Deserialize;

#[derive(Deserialize)]
pub struct MockRepository {
    pub app: String,
    pub catalogs: String,
}

#[derive(Deserialize)]
pub struct MockConfig {
    pub address: String,
    pub repository: MockRepository,
}
