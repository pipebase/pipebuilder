// registry implemented with [etcd-client](https://crates.io/crates/etcd-client)
use crate::{
    read_file, resource_id, resource_namespace, resource_namespace_id,
    resource_namespace_id_version, root_resource, AppMetadata, BuildSnapshot, ManifestMetadata,
    ManifestSnapshot, Namespace, NodeState, Project, Result, VersionBuild, RESOURCE_APP_METADATA,
    RESOURCE_BUILD_SNAPSHOT, RESOURCE_MANIFEST_METADATA, RESOURCE_MANIFEST_SNAPSHOT,
    RESOURCE_NAMESPACE, RESOURCE_NODE_BUILDER, RESOURCE_PROJECT, RESOURCE_VERSION_BUILD,
};
use etcd_client::{
    Certificate, Client, ConnectOptions, DeleteOptions, DeleteResponse, GetOptions, GetResponse,
    Identity, KeyValue, LeaseGrantResponse, LockOptions, LockResponse, PutOptions, PutResponse,
    TlsOptions, WatchOptions, WatchStream, Watcher,
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
    async fn into_identity(self) -> Result<Identity> {
        let cert = read_file(self.cert).await?;
        let key = read_file(self.key).await?;
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
    pub async fn into_tls_options(self) -> Result<TlsOptions> {
        let tls = TlsOptions::new();
        let tls = match self.domain {
            Some(domain) => tls.domain_name(domain),
            None => tls,
        };
        let tls = match self.ca_cert {
            Some(ca_cert) => {
                let pem = read_file(ca_cert).await?;
                tls.ca_certificate(Certificate::from_pem(pem))
            }
            None => tls,
        };
        let tls = match self.identity {
            Some(identity) => {
                let identity = identity.into_identity().await?;
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
    async fn into_connect_opts(self) -> Result<ConnectOptions> {
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
            Some(tls) => opts.with_tls(tls.into_tls_options().await?),
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
            Some(connect) => Some(connect.into_connect_opts().await?),
            None => None,
        };
        let client = Client::connect(config.endpoints, connect_opts).await?;
        Ok(Register { client })
    }

    async fn put<K, V>(
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

    async fn get<K>(&mut self, key: K, options: Option<GetOptions>) -> Result<GetResponse>
    where
        K: Into<Vec<u8>>,
    {
        let resp = self.client.get(key, options).await?;
        Ok(resp)
    }

    pub async fn delete<K>(
        &mut self,
        key: K,
        options: Option<DeleteOptions>,
    ) -> Result<DeleteResponse>
    where
        K: Into<Vec<u8>>,
    {
        let resp = self.client.delete(key, options).await?;
        Ok(resp)
    }

    // list with key prefix
    async fn list<K>(&mut self, prefix: K) -> Result<GetResponse>
    where
        K: Into<Vec<u8>>,
    {
        let resp = self
            .get(prefix, Some(GetOptions::new().with_prefix()))
            .await?;
        Ok(resp)
    }

    async fn list_kvs<K, V>(&mut self, prefix: K) -> Result<Vec<(String, V)>>
    where
        K: Into<Vec<u8>>,
        V: DeserializeOwned,
    {
        let resp = self.list(prefix).await?;
        let kvs = Self::deserialize_kvs::<V>(resp.kvs())?;
        Ok(kvs)
    }

    pub async fn list_keys<K>(&mut self, prefix: K) -> Result<Vec<String>>
    where
        K: Into<Vec<u8>>,
    {
        let resp = self
            .get(
                prefix,
                Some(GetOptions::new().with_prefix().with_keys_only()),
            )
            .await?;
        let mut keys = vec![];
        for kv in resp.kvs() {
            keys.push(kv.key_str()?.to_owned());
        }
        Ok(keys)
    }

    pub async fn count<K>(&mut self, key: K) -> Result<usize>
    where
        K: Into<Vec<u8>>,
    {
        let options = GetOptions::new().with_count_only();
        let resp = self.get(key, Some(options)).await?;
        Ok(resp.count() as usize)
    }

    pub async fn count_prefix<K>(&mut self, prefix: K) -> Result<usize>
    where
        K: Into<Vec<u8>>,
    {
        let options = GetOptions::new().with_prefix().with_count_only();
        let resp = self.get(prefix, Some(options)).await?;
        Ok(resp.count() as usize)
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
        role_prefix: &str,
        state: &NodeState,
        lease_id: i64,
    ) -> Result<PutResponse> {
        let id = &state.id;
        let value = serde_json::to_vec(state)?;
        let opts = PutOptions::new().with_lease(lease_id);
        let key = format!("{}/{}", role_prefix, id);
        let resp = self.put(key, value, opts.into()).await?;
        Ok(resp)
    }

    pub async fn list_node_state(&mut self, prefix: &str) -> Result<Vec<(String, NodeState)>> {
        let node_states = self.list_kvs::<&str, NodeState>(prefix).await?;
        Ok(node_states)
    }

    async fn do_get_node_state(
        &mut self,
        role_prefix: &str,
        id: &str,
    ) -> Result<Option<NodeState>> {
        let key = format!("{}/{}", role_prefix, id);
        let node_state = self.get_json_value::<String, NodeState>(key, None).await?;
        Ok(node_state)
    }

    pub async fn get_node_state(
        &mut self,
        lease_id: i64,
        role_prefix: &str,
        id: &str,
    ) -> Result<Option<NodeState>> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_id(role_prefix, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_node_state(role_prefix, id).await?;
        self.unlock(lock_name.as_str(), key).await?;
        Ok(resp)
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
        self.watch_prefix(RESOURCE_NODE_BUILDER).await
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
        let key = resource_namespace_id(RESOURCE_BUILD_SNAPSHOT, namespace, id);
        let value = serde_json::to_vec(&new_snapshot)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, new_snapshot))
    }

    async fn do_get_build_snapshot(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<BuildSnapshot>> {
        let key = resource_namespace_id(RESOURCE_BUILD_SNAPSHOT, namespace, id);
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
        let lock_name = resource_namespace_id(RESOURCE_BUILD_SNAPSHOT, namespace, id);
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
        let key = resource_namespace_id_version(RESOURCE_VERSION_BUILD, namespace, id, version);
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
        let lock_name =
            resource_namespace_id_version(RESOURCE_VERSION_BUILD, namespace, id, version);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_version_build(namespace, id, version).await?;
        self.unlock(lock_name.as_str(), key).await?;
        Ok(resp)
    }

    pub async fn list_version_build(
        &mut self,
        namespace: &str,
        id: Option<String>,
    ) -> Result<Vec<(String, VersionBuild)>> {
        let prefix = match id {
            Some(id) => resource_namespace_id(RESOURCE_VERSION_BUILD, namespace, id.as_str()),
            None => resource_namespace(RESOURCE_VERSION_BUILD, namespace),
        };
        let version_builds = self.list_kvs::<&str, VersionBuild>(prefix.as_str()).await?;
        Ok(version_builds)
    }

    pub async fn do_put_version_build(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
        state: VersionBuild,
    ) -> Result<(PutResponse, VersionBuild)> {
        let key = resource_namespace_id_version(RESOURCE_VERSION_BUILD, namespace, id, version);
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
        let lock_name =
            resource_namespace_id_version(RESOURCE_VERSION_BUILD, namespace, id, version);
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
        let key = resource_namespace_id(RESOURCE_MANIFEST_SNAPSHOT, namespace, id);
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
        let lock_name = resource_namespace_id(RESOURCE_MANIFEST_SNAPSHOT, namespace, id);
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
        let key = resource_namespace_id(RESOURCE_MANIFEST_SNAPSHOT, namespace, id);
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
        let lock_name = resource_namespace_id(RESOURCE_MANIFEST_SNAPSHOT, namespace, id);
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
        let prefix = resource_namespace(RESOURCE_MANIFEST_SNAPSHOT, namespace);
        let manifest_snapshots = self
            .list_kvs::<&str, ManifestSnapshot>(prefix.as_str())
            .await?;
        Ok(manifest_snapshots)
    }

    // list build snapshot in namespace
    pub async fn list_build_snapshot(
        &mut self,
        namespace: &str,
    ) -> Result<Vec<(String, BuildSnapshot)>> {
        let prefix = resource_namespace(RESOURCE_BUILD_SNAPSHOT, namespace);
        let build_snapshots = self
            .list_kvs::<&str, BuildSnapshot>(prefix.as_str())
            .await?;
        Ok(build_snapshots)
    }

    async fn do_get_app_metadata(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Option<AppMetadata>> {
        let key = resource_namespace_id_version(RESOURCE_APP_METADATA, namespace, id, version);
        let metadata = self
            .get_json_value::<String, AppMetadata>(key, None)
            .await?;
        Ok(metadata)
    }

    pub async fn get_app_metadata(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Option<AppMetadata>> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name =
            resource_namespace_id_version(RESOURCE_APP_METADATA, namespace, id, version);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_app_metadata(namespace, id, version).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_update_app_metadata(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
        size: usize,
    ) -> Result<(PutResponse, AppMetadata)> {
        let new_metadata = match self.do_get_app_metadata(namespace, id, version).await? {
            Some(mut metadata) => {
                metadata.pulls += 1;
                metadata
            }
            None => AppMetadata::new(size),
        };
        let key = resource_namespace_id_version(RESOURCE_APP_METADATA, namespace, id, version);
        let value = serde_json::to_vec(&new_metadata)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, new_metadata))
    }

    pub async fn update_app_metadata(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
        version: u64,
        size: usize,
    ) -> Result<(PutResponse, AppMetadata)> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name =
            resource_namespace_id_version(RESOURCE_APP_METADATA, namespace, id, version);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self
            .do_update_app_metadata(namespace, id, version, size)
            .await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    pub async fn list_app_metadata(
        &mut self,
        namespace: &str,
        id: Option<String>,
    ) -> Result<Vec<(String, AppMetadata)>> {
        let prefix = match id {
            Some(id) => resource_namespace_id(RESOURCE_APP_METADATA, namespace, id.as_str()),
            None => resource_namespace(RESOURCE_APP_METADATA, namespace),
        };
        let resp = self.list_kvs::<&str, AppMetadata>(prefix.as_str()).await?;
        Ok(resp)
    }

    async fn do_get_manifest_metadata(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Option<ManifestMetadata>> {
        let key = resource_namespace_id_version(RESOURCE_MANIFEST_METADATA, namespace, id, version);
        let metadata = self
            .get_json_value::<String, ManifestMetadata>(key, None)
            .await?;
        Ok(metadata)
    }

    pub async fn get_manifest_metadata(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Option<ManifestMetadata>> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name =
            resource_namespace_id_version(RESOURCE_MANIFEST_METADATA, namespace, id, version);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_manifest_metadata(namespace, id, version).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_update_manifest_metadata(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
        size: usize,
    ) -> Result<(PutResponse, ManifestMetadata)> {
        let new_metadata = match self
            .do_get_manifest_metadata(namespace, id, version)
            .await?
        {
            Some(mut metadata) => {
                metadata.pulls += 1;
                metadata
            }
            None => ManifestMetadata::new(size),
        };
        let key = resource_namespace_id_version(RESOURCE_MANIFEST_METADATA, namespace, id, version);
        let value = serde_json::to_vec(&new_metadata)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, new_metadata))
    }

    pub async fn update_manifest_metadata(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
        version: u64,
        size: usize,
    ) -> Result<(PutResponse, ManifestMetadata)> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name =
            resource_namespace_id_version(RESOURCE_MANIFEST_METADATA, namespace, id, version);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self
            .do_update_manifest_metadata(namespace, id, version, size)
            .await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    pub async fn list_manifest_metadata(
        &mut self,
        namespace: &str,
        id: Option<String>,
    ) -> Result<Vec<(String, ManifestMetadata)>> {
        let prefix = match id {
            Some(id) => resource_namespace_id(RESOURCE_MANIFEST_METADATA, namespace, id.as_str()),
            None => resource_namespace(RESOURCE_MANIFEST_METADATA, namespace),
        };
        let resp = self
            .list_kvs::<&str, ManifestMetadata>(prefix.as_str())
            .await?;
        Ok(resp)
    }

    async fn do_get_namespace(&mut self, id: &str) -> Result<Option<Namespace>> {
        let key = resource_id(RESOURCE_NAMESPACE, id);
        let namespace = self.get_json_value::<String, Namespace>(key, None).await?;
        Ok(namespace)
    }

    pub async fn get_namespace(&mut self, lease_id: i64, id: &str) -> Result<Option<Namespace>> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_id(RESOURCE_NAMESPACE, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_namespace(id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_update_namespace(&mut self, id: &str) -> Result<(PutResponse, Namespace)> {
        let namespace = match self.do_get_namespace(id).await? {
            Some(namespace) => namespace,
            None => Namespace::new(),
        };
        let key = resource_id(RESOURCE_NAMESPACE, id);
        let value = serde_json::to_vec(&namespace)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, namespace))
    }

    pub async fn update_namespace(
        &mut self,
        lease_id: i64,
        id: &str,
    ) -> Result<(PutResponse, Namespace)> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_id(RESOURCE_NAMESPACE, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_update_namespace(id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    pub async fn list_namespace(&mut self) -> Result<Vec<(String, Namespace)>> {
        let prefix = root_resource(RESOURCE_NAMESPACE);
        let resp = self.list_kvs::<&str, Namespace>(prefix.as_str()).await?;
        Ok(resp)
    }

    async fn do_get_project(&mut self, namespace: &str, id: &str) -> Result<Option<Project>> {
        let key = resource_namespace_id(RESOURCE_PROJECT, namespace, id);
        let project = self.get_json_value::<String, Project>(key, None).await?;
        Ok(project)
    }

    pub async fn get_project(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
    ) -> Result<Option<Project>> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_namespace_id(RESOURCE_PROJECT, namespace, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_project(namespace, id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_update_project(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<(PutResponse, Project)> {
        let project = match self.do_get_project(namespace, id).await? {
            Some(project) => project,
            None => Project::new(),
        };
        let key = resource_namespace_id(RESOURCE_PROJECT, namespace, id);
        let value = serde_json::to_vec(&project)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, project))
    }

    pub async fn update_project(
        &mut self,
        lease_id: i64,
        namespace: &str,
        id: &str,
    ) -> Result<(PutResponse, Project)> {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = resource_namespace_id(RESOURCE_PROJECT, namespace, id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_update_project(namespace, id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    pub async fn list_project(&mut self, namespace: &str) -> Result<Vec<(String, Project)>> {
        let prefix = resource_namespace(RESOURCE_PROJECT, namespace);
        let resp = self.list_kvs::<&str, Project>(prefix.as_str()).await?;
        Ok(resp)
    }

    pub async fn delete_version_build(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<()> {
        let key = resource_namespace_id_version(RESOURCE_VERSION_BUILD, namespace, id, version);
        let _ = self.delete(key, None).await?;
        Ok(())
    }

    pub async fn delete_build_snapshot(&mut self, namespace: &str, id: &str) -> Result<()> {
        let key = resource_namespace_id(RESOURCE_BUILD_SNAPSHOT, namespace, id);
        let _ = self.delete(key, None).await?;
        Ok(())
    }

    pub async fn delete_app_meta(&mut self, namespace: &str, id: &str, version: u64) -> Result<()> {
        let key = resource_namespace_id_version(RESOURCE_APP_METADATA, namespace, id, version);
        let _ = self.delete(key, None).await?;
        Ok(())
    }

    pub async fn delete_manifest_meta(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<()> {
        let key = resource_namespace_id_version(RESOURCE_MANIFEST_METADATA, namespace, id, version);
        let _ = self.delete(key, None).await?;
        Ok(())
    }

    pub async fn delete_manifest_snapshot(&mut self, namespace: &str, id: &str) -> Result<()> {
        let key = resource_namespace_id(RESOURCE_MANIFEST_SNAPSHOT, namespace, id);
        let _ = self.delete(key, None).await?;
        Ok(())
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
