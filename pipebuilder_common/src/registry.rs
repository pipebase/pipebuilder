// registry implemented with [etcd-client](https://crates.io/crates/etcd-client)
use crate::{read_file, Result};
use etcd_client::{Certificate, Client, ConnectOptions, Identity, TlsOptions};
use serde::Deserialize;

use crate::Period;

#[derive(Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct KeepAliveConfig {
    pub keep_alive_interval: Period,
    pub keep_alive_timeout: Period,
}

#[derive(Deserialize)]
pub struct IdentityConfig {
    pub cert: String,
    pub key: String,
}

impl IdentityConfig {
    fn into_identity(self) -> Result<Identity> {
        let cert = read_file(self.cert)?;
        let key = read_file(self.key)?;
        Ok(Identity::from_pem(cert, key))
    }
}

#[derive(Deserialize)]
pub struct TlsConfig {
    pub domain: Option<String>,
    // CA Certificate file, verify the server's TLS certificate
    pub ca_cert: Option<String>,
    // Client identity present to server
    pub identity: Option<IdentityConfig>,
}

impl TlsConfig {
    pub fn into_tls_options(self) -> Result<TlsOptions> {
        let tls = TlsOptions::new();
        let tls = match self.domain {
            Some(domain) => tls.domain_name(domain),
            None => tls,
        };
        let tls = match self.ca_cert {
            Some(ca_cert) => {
                let pem = read_file(ca_cert)?;
                tls.ca_certificate(Certificate::from_pem(pem))
            }
            None => tls,
        };
        let tls = match self.identity {
            Some(identity) => {
                let identity = identity.into_identity()?;
                tls.identity(identity)
            }
            None => tls,
        };
        Ok(tls)
    }
}

#[derive(Deserialize)]
pub struct ConnectConfig {
    pub user: Option<UserConfig>,
    pub keep_alive: Option<KeepAliveConfig>,
    pub tls: Option<TlsConfig>,
}

impl ConnectConfig {
    fn into_connect_opts(self) -> Result<ConnectOptions> {
        let opts = ConnectOptions::new();
        let opts = match self.user {
            Some(user) => opts.with_user(user.name, user.password),
            None => opts,
        };
        let opts = match self.keep_alive {
            Some(keep_alive) => opts.with_keep_alive(
                keep_alive.keep_alive_interval.into(),
                keep_alive.keep_alive_timeout.into(),
            ),
            None => opts,
        };
        let opts = match self.tls {
            Some(tls) => opts.with_tls(tls.into_tls_options()?),
            None => opts,
        };
        Ok(opts)
    }
}

// etcd client config
#[derive(Deserialize)]
pub struct RegistryConfig {
    // etcd endpoints
    pub endpoints: Vec<String>,
    pub connect: Option<ConnectConfig>,
}

impl RegistryConfig {
    pub async fn into_registry(self) -> Result<Registry> {
        let connect_opts: Option<ConnectOptions> = match self.connect {
            Some(connect) => Some(connect.into_connect_opts()?),
            None => None,
        };
        let client = Client::connect(self.endpoints, connect_opts).await?;
        Ok(Registry { client })
    }
}

#[derive(Clone)]
pub struct Registry {
    client: Client,
}
