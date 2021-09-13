use serde::Deserialize;

// registry implemented with [etcd-client](https://crates.io/crates/etcd-client)

#[derive(Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct KeepAliveConfig {
    // keep_alive_interval
// keep_alive_timeout
}

#[derive(Deserialize)]
pub struct TlsConfig {
    pub domain: Option<String>,
    // CA Certificate file, verify the server's TLS certificate
    pub ca_cert: Option<String>,
    // Client identity present to server
    pub cert: Option<String>,
    pub key: Option<String>,
}

#[derive(Deserialize)]
pub struct ConnectionConfig {
    pub user: Option<UserConfig>,
    pub keep_alive: Option<KeepAliveConfig>,
    pub tls: Option<TlsConfig>,
}

// etcd client config
#[derive(Deserialize)]
pub struct RegistryConfig {
    // etcd endpoints
    pub endpoints: Vec<String>,
    pub connection: Option<ConnectionConfig>,
}
