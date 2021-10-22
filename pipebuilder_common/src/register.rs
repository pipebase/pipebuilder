// registry implemented with [etcd-client](https://crates.io/crates/etcd-client)
use crate::{
    read_file, resource_namespace, resource_namespace_id, resource_namespace_id_version,
    BuildSnapshot, ManifestSnapshot, NodeState, Result, VersionBuild, REGISTER_KEY_PREFIX_BUILDER,
    REGISTER_KEY_PREFIX_BUILD_SNAPSHOT, REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT,
    REGISTER_KEY_PREFIX_VERSION_BUILD,
};
use etcd_client::{
    Certificate, Client, ConnectOptions, GetOptions, GetResponse, Identity, KeyValue,
    LeaseGrantResponse, LockOptions, LockResponse, PutOptions, PutResponse, TlsOptions,
    WatchOptions, WatchStream, Watcher,
};
use serde::{de::DeserializeOwned, Deserialize};
use tracing::info;

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
pub struct RegisterConfig {
    // etcd endpoints
    pub endpoints: Vec<String>,
    pub connect: Option<ConnectConfig>,
}

#[derive(Clone)]
pub struct Register {
    client: Client,
}

impl Register {
    pub async fn new(config: RegisterConfig) -> Result<Register> {
        let connect_opts: Option<ConnectOptions> = match config.connect {
            Some(connect) => Some(connect.into_connect_opts()?),
            None => None,
        };
        let client = Client::connect(config.endpoints, connect_opts).await?;
        Ok(Register { client })
    }

    pub async fn put<K, V>(
        &mut self,
        key: K,
        value: V,
        options: Option<PutOptions>,
    ) -> Result<PutResponse>
    where
        K: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
    {
        let resp = self.client.put(key, value, options).await?;
        Ok(resp)
    }

    pub async fn get<K>(&mut self, key: K, options: Option<GetOptions>) -> Result<GetResponse>
    where
        K: Into<Vec<u8>>,
    {
        let resp = self.client.get(key, options).await?;
        Ok(resp)
    }

    // list with key prefix
    async fn list<V>(&mut self, prefix: &str) -> Result<Vec<(String, V)>>
    where
        V: DeserializeOwned,
    {
        let resp = self
            .get(prefix, Some(GetOptions::new().with_prefix()))
            .await?;
        let kvs = Self::deserialize_kvs::<V>(resp.kvs())?;
        Ok(kvs)
    }

    pub async fn get_json_value<K, V>(
        &mut self,
        key: K,
        options: Option<GetOptions>,
    ) -> Result<Option<V>>
    where
        K: Into<Vec<u8>>,
        V: DeserializeOwned,
    {
        let get_resp = self.get(key, options).await?;
        let object = match get_resp.kvs().first() {
            Some(kv) => {
                let object = serde_json::from_slice::<V>(kv.value())?;
                Some(object)
            }
            None => None,
        };
        Ok(object)
    }

    pub async fn lease_grant(&mut self, ttl: i64) -> Result<LeaseGrantResponse> {
        let resp = self.client.lease_grant(ttl, None).await?;
        Ok(resp)
    }

    pub async fn lease_keep_alive(&mut self, id: i64) -> Result<()> {
        let (mut keeper, mut stream) = self.client.lease_keep_alive(id).await?;
        keeper.keep_alive().await?;
        if let Some(resp) = stream.message().await? {
            info!("lease {:?} keep alive, new ttl {:?}", resp.id(), resp.ttl());
        }
        Ok(())
    }

    pub async fn put_node_state(
        &mut self,
        prefix: &str,
        state: &NodeState,
        lease_id: i64,
    ) -> Result<PutResponse> {
        let id = &state.id;
        let value = serde_json::to_vec(state)?;
        let opts = PutOptions::new().with_lease(lease_id);
        let key = format!("{}/{}", prefix, id);
        let resp = self.put(key, value, opts.into()).await?;
        Ok(resp)
    }

    pub async fn list_node_state(&mut self, prefix: &str) -> Result<Vec<(String, NodeState)>> {
        let node_states = self.list::<NodeState>(prefix).await?;
        Ok(node_states)
    }

    pub async fn watch(
        &mut self,
        key: &str,
        options: Option<WatchOptions>,
    ) -> Result<(Watcher, WatchStream)> {
        let (watcher, stream) = self.client.watch(key, options).await?;
        Ok((watcher, stream))
    }

    pub async fn watch_prefix(&mut self, prefix: &str) -> Result<(Watcher, WatchStream)> {
        let opts = WatchOptions::new().with_prefix();
        self.watch(prefix, opts.into()).await
    }

    pub async fn watch_builders(&mut self) -> Result<(Watcher, WatchStream)> {
        self.watch_prefix(REGISTER_KEY_PREFIX_BUILDER).await
    }

    pub async fn lock(&mut self, name: &str, options: Option<LockOptions>) -> Result<LockResponse> {
        info!("acquire lock, name {} ...", name);
        let resp = self.client.lock(name, options).await?;
        Ok(resp)
    }

    pub async fn unlock(&mut self, name: &str, key: &[u8]) -> Result<()> {
        self.client.unlock(key).await?;
        info!("released lock, name {} ...", name);
        Ok(())
    }

    async fn do_incr_build_snapshot(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<(PutResponse, BuildSnapshot)> {
        // get current snapshot and incr version
        let new_snapshot = match self.do_get_build_snapshot(namespace, id).await? {
            Some(mut snapshot) => {
                snapshot.latest_version += 1;
                snapshot
            }
            None => BuildSnapshot::default(),
        };
        let key = resource_namespace_id(REGISTER_KEY_PREFIX_BUILD_SNAPSHOT, namespace, id);
        let value = serde_json::to_vec(&new_snapshot)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, new_snapshot))
    }

    async fn do_get_build_snapshot(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<BuildSnapshot>> {
        let key = resource_namespace_id(REGISTER_KEY_PREFIX_BUILD_SNAPSHOT, namespace, id);
        let snapshot = self
            .get_json_value::<String, BuildSnapshot>(key, None)
            .await?;
        Ok(snapshot)
    }

    pub async fn incr_build_snapshot(
        &mut self,
        namespace: &str,
        id: &str,
        lease_id: i64,
    ) -> Result<(PutResponse, BuildSnapshot)> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_namespace_id(REGISTER_KEY_PREFIX_BUILD_SNAPSHOT, namespace, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_incr_build_snapshot(namespace, id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    pub async fn do_get_version_build(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Option<VersionBuild>> {
        let key = resource_namespace_id_version(
            REGISTER_KEY_PREFIX_VERSION_BUILD,
            namespace,
            id,
            version,
        );
        let state = self
            .get_json_value::<String, VersionBuild>(key, None)
            .await?;
        Ok(state)
    }

    pub async fn get_version_build(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Option<VersionBuild>> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_namespace_id_version(
            REGISTER_KEY_PREFIX_VERSION_BUILD,
            namespace,
            id,
            version,
        );
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_version_build(namespace, id, version).await?;
        self.unlock(lock_name.as_str(), key).await?;
        Ok(resp)
    }

    pub async fn list_version_build(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Vec<(String, VersionBuild)>> {
        let prefix = resource_namespace_id(REGISTER_KEY_PREFIX_VERSION_BUILD, namespace, id);
        let version_builds = self.list::<VersionBuild>(prefix.as_str()).await?;
        Ok(version_builds)
    }

    pub async fn do_put_version_build(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
        state: VersionBuild,
    ) -> Result<(PutResponse, VersionBuild)> {
        let key = resource_namespace_id_version(
            REGISTER_KEY_PREFIX_VERSION_BUILD,
            namespace,
            id,
            version,
        );
        let value = serde_json::to_vec(&state)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, state))
    }

    pub async fn put_version_build(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
        version: u64,
        state: VersionBuild,
    ) -> Result<(PutResponse, VersionBuild)> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_namespace_id_version(
            REGISTER_KEY_PREFIX_VERSION_BUILD,
            namespace,
            id,
            version,
        );
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self
            .do_put_version_build(namespace, id, version, state)
            .await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_incr_manifest_snapshot(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<(PutResponse, ManifestSnapshot)> {
        let new_snapshot = match self.do_get_manifest_snapshot(namespace, id).await? {
            Some(mut snapshot) => {
                snapshot.latest_version += 1;
                snapshot
            }
            None => ManifestSnapshot::new(),
        };
        let key = resource_namespace_id(REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT, namespace, id);
        let value = serde_json::to_vec(&new_snapshot)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, new_snapshot))
    }

    pub async fn incr_manifest_snapshot(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
    ) -> Result<(PutResponse, ManifestSnapshot)> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_namespace_id(REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT, namespace, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_incr_manifest_snapshot(namespace, id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_get_manifest_snapshot(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<ManifestSnapshot>> {
        let key = resource_namespace_id(REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT, namespace, id);
        let snapshot = self
            .get_json_value::<String, ManifestSnapshot>(key, None)
            .await?;
        Ok(snapshot)
    }

    pub async fn get_manifest_snapshot(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
    ) -> Result<Option<ManifestSnapshot>> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_namespace_id(REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT, namespace, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_manifest_snapshot(namespace, id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    // list manifest snapshot in namespace
    pub async fn list_manifest_snapshot(
        &mut self,
        namespace: &str,
    ) -> Result<Vec<(String, ManifestSnapshot)>> {
        let prefix = resource_namespace(REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT, namespace);
        let manifest_snapshots = self.list::<ManifestSnapshot>(prefix.as_str()).await?;
        Ok(manifest_snapshots)
    }

    // list build snapshot in namespace
    pub async fn list_build_snapshot(
        &mut self,
        namespace: &str,
    ) -> Result<Vec<(String, BuildSnapshot)>> {
        let prefix = resource_namespace(REGISTER_KEY_PREFIX_BUILD_SNAPSHOT, namespace);
        let build_snapshots = self.list::<BuildSnapshot>(prefix.as_str()).await?;
        Ok(build_snapshots)
    }

    fn deserialize_kvs<T>(kvs: &[KeyValue]) -> Result<Vec<(String, T)>>
    where
        T: DeserializeOwned,
    {
        let mut deserialize_kvs: Vec<(String, T)> = vec![];
        for kv in kvs {
            let key = kv.key_str()?;
            let value = serde_json::from_slice::<T>(kv.value())?;
            deserialize_kvs.push((key.to_owned(), value))
        }
        Ok(deserialize_kvs)
    }
}
