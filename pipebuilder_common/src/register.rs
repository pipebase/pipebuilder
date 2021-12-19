// registry implemented with [etcd-client](https://crates.io/crates/etcd-client)
use crate::{
    read_file, BlobResource, Resource, ResourceKeyBuilder, ResourceType, Result, Snapshot,
};
use etcd_client::{
    Certificate, Client, ConnectOptions, DeleteOptions, DeleteResponse, GetOptions, GetResponse,
    Identity, KeyValue, LeaseGrantResponse, LockOptions, LockResponse, PutOptions, PutResponse,
    TlsOptions, WatchOptions, WatchStream, Watcher,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

    async fn list_json_kvs<K, V>(&mut self, prefix: K) -> Result<Vec<(String, V)>>
    where
        K: Into<Vec<u8>>,
        V: DeserializeOwned,
    {
        let resp = self.list(prefix).await?;
        let kvs = Self::deserialize_json_kvs::<V>(resp.kvs())?;
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

    pub async fn is_exist<K>(&mut self, key: K) -> Result<bool>
    where
        K: Into<Vec<u8>>,
    {
        let count = self.count(key).await?;
        Ok(count > 0)
    }

    pub async fn count_prefix<K>(&mut self, prefix: K) -> Result<usize>
    where
        K: Into<Vec<u8>>,
    {
        let options = GetOptions::new().with_prefix().with_count_only();
        let resp = self.get(prefix, Some(options)).await?;
        Ok(resp.count() as usize)
    }

    pub async fn is_prefix_exist<K>(&mut self, prefix: K) -> Result<bool>
    where
        K: Into<Vec<u8>>,
    {
        let count = self.count_prefix(prefix).await?;
        Ok(count > 0)
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
            info!(lease_id = resp.id(), ttl = resp.ttl(), "lease keep alive");
        }
        Ok(())
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

    pub async fn watch_nodes(&mut self) -> Result<(Watcher, WatchStream)> {
        let prefix_key = ResourceKeyBuilder::new()
            .resource(ResourceType::Node)
            .build();
        self.watch_prefix(prefix_key.as_str()).await
    }

    pub async fn lock(&mut self, name: &str, options: Option<LockOptions>) -> Result<LockResponse> {
        info!(lock_name = name, "acquire lock ...");
        let resp = self.client.lock(name, options).await?;
        Ok(resp)
    }

    pub async fn unlock(&mut self, name: &str, key: &[u8]) -> Result<()> {
        self.client.unlock(key).await?;
        info!(lock_name = name, "released lock ...");
        Ok(())
    }

    async fn do_update_snapshot_resource<S>(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<(PutResponse, S)>
    where
        S: Resource + Snapshot + Serialize + DeserializeOwned,
    {
        // get current snapshot and incr version
        let new_snapshot = match self.do_get_resource::<S>(Some(namespace), id, None).await? {
            Some(mut snapshot) => {
                snapshot.incr_version();
                snapshot
            }
            None => S::default(),
        };
        let key = ResourceKeyBuilder::new()
            .resource(S::ty())
            .namespace(namespace)
            .id(id)
            .build();
        let value = serde_json::to_vec(&new_snapshot)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, new_snapshot))
    }

    async fn do_get_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: &str,
        version: Option<u64>,
    ) -> Result<Option<R>>
    where
        R: Resource + DeserializeOwned,
    {
        let builder = ResourceKeyBuilder::new().resource(R::ty()).id(id);
        let builder = match namespace {
            Some(namespace) => builder.namespace(namespace),
            None => builder,
        };
        let key = match version {
            Some(version) => builder.version(version).build(),
            None => builder.build(),
        };
        let resource = self.get_json_value::<String, R>(key, None).await?;
        Ok(resource)
    }

    pub async fn get_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: &str,
        version: Option<u64>,
        lease_id: i64,
    ) -> Result<Option<R>>
    where
        R: Resource + DeserializeOwned,
    {
        let builder = ResourceKeyBuilder::new()
            .lock(true)
            .resource(R::ty())
            .id(id);
        let builder = match namespace {
            Some(namespace) => builder.namespace(namespace),
            None => builder,
        };
        let lock_name = match version {
            Some(version) => builder.version(version).build(),
            None => builder.build(),
        };
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_get_resource::<R>(namespace, id, version).await?;
        self.unlock(lock_name.as_str(), key).await?;
        Ok(resp)
    }

    pub async fn update_snapshot_resource<S>(
        &mut self,
        namespace: &str,
        id: &str,
        lease_id: i64,
    ) -> Result<(PutResponse, S)>
    where
        S: Resource + Snapshot + Serialize + DeserializeOwned,
    {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = ResourceKeyBuilder::new()
            .lock(true)
            .resource(S::ty())
            .namespace(namespace)
            .id(id)
            .build();
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_update_snapshot_resource::<S>(namespace, id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    // list resource metadata or snapshot given resource type namespace, id
    pub async fn list_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: Option<&str>,
    ) -> Result<Vec<(String, R)>>
    where
        R: Resource + DeserializeOwned,
    {
        let builder = ResourceKeyBuilder::new().resource(R::ty());
        let builder = match namespace {
            Some(namespace) => builder.namespace(namespace),
            None => builder,
        };
        let prefix = match id {
            Some(id) => builder.id(id).build(),
            None => builder.build(),
        };
        let builds = self.list_json_kvs::<&str, R>(prefix.as_str()).await?;
        Ok(builds)
    }

    async fn do_put_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: &str,
        version: Option<u64>,
        resource: R,
    ) -> Result<(PutResponse, R)>
    where
        R: Resource + Serialize,
    {
        let builder = ResourceKeyBuilder::new().resource(R::ty()).id(id);
        let builder = match namespace {
            Some(namespace) => builder.namespace(namespace),
            None => builder,
        };
        let key = match version {
            Some(version) => builder.version(version).build(),
            None => builder.build(),
        };
        let value = serde_json::to_vec(&resource)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, resource))
    }

    pub async fn put_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: &str,
        version: Option<u64>,
        resource: R,
        lease_id: i64,
    ) -> Result<(PutResponse, R)>
    where
        R: Resource + Serialize,
    {
        let builder = ResourceKeyBuilder::new()
            .lock(true)
            .resource(R::ty())
            .id(id);
        let builder = match namespace {
            Some(namespace) => builder.namespace(namespace),
            None => builder,
        };
        let lock_name = match version {
            Some(version) => builder.version(version).build(),
            None => builder.build(),
        };
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_put_resource(namespace, id, version, resource).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_update_blob_resource<R>(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
        size: usize,
    ) -> Result<(PutResponse, R)>
    where
        R: Resource + BlobResource + Serialize + DeserializeOwned,
    {
        let new_metadata = match self
            .do_get_resource::<R>(Some(namespace), id, Some(version))
            .await?
        {
            Some(mut metadata) => {
                metadata.incr_usage();
                metadata
            }
            None => R::new(size),
        };
        let key = ResourceKeyBuilder::new()
            .resource(R::ty())
            .namespace(namespace)
            .id(id)
            .version(version)
            .build();
        let value = serde_json::to_vec(&new_metadata)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, new_metadata))
    }

    pub async fn update_blob_resource<R>(
        &mut self,
        namespace: &str,
        id: &str,
        version: u64,
        size: usize,
        lease_id: i64,
    ) -> Result<(PutResponse, R)>
    where
        R: Resource + BlobResource + Serialize + DeserializeOwned,
    {
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_name = ResourceKeyBuilder::new()
            .lock(true)
            .resource(R::ty())
            .namespace(namespace)
            .id(id)
            .version(version)
            .build();
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self
            .do_update_blob_resource::<R>(namespace, id, version, size)
            .await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    async fn do_update_default_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: &str,
    ) -> Result<(PutResponse, R)>
    where
        R: Resource + Default + Serialize + DeserializeOwned,
    {
        let resource = match self.do_get_resource::<R>(namespace, id, None).await? {
            Some(resource) => resource,
            None => R::default(),
        };
        let builder = ResourceKeyBuilder::new().resource(R::ty()).id(id);
        let key = match namespace {
            Some(namespace) => builder.namespace(namespace).build(),
            None => builder.build(),
        };
        let value = serde_json::to_vec(&resource)?;
        let resp = self.put(key, value, None).await?;
        Ok((resp, resource))
    }

    pub async fn update_default_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: &str,
        lease_id: i64,
    ) -> Result<(PutResponse, R)>
    where
        R: Resource + Default + Serialize + DeserializeOwned,
    {
        let builder = ResourceKeyBuilder::new()
            .lock(true)
            .resource(R::ty())
            .id(id);
        let lock_name = match namespace {
            Some(namespace) => builder.namespace(namespace).build(),
            None => builder.build(),
        };
        let lock_options = LockOptions::new().with_lease(lease_id);
        let lock_resp = self.lock(lock_name.as_str(), lock_options.into()).await?;
        let key = lock_resp.key();
        let resp = self.do_update_default_resource::<R>(namespace, id).await;
        self.unlock(lock_name.as_str(), key).await?;
        resp
    }

    pub async fn delete_resource<R>(
        &mut self,
        namespace: Option<&str>,
        id: &str,
        version: Option<u64>,
    ) -> Result<()>
    where
        R: Resource,
    {
        let builder = ResourceKeyBuilder::new().resource(R::ty()).id(id);
        let builder = match namespace {
            Some(namespace) => builder.namespace(namespace),
            None => builder,
        };
        let key = match version {
            Some(version) => builder.version(version).build(),
            None => builder.build(),
        };
        let _ = self.delete(key, None).await?;
        Ok(())
    }

    pub async fn is_resource_exist<R>(&mut self, namespace: &str, id: Option<&str>) -> Result<bool>
    where
        R: Resource,
    {
        let builder = ResourceKeyBuilder::new()
            .resource(R::ty())
            .namespace(namespace);
        let prefix = match id {
            Some(id) => builder.id(id).build(),
            None => builder.build(),
        };
        self.is_prefix_exist(prefix).await
    }

    fn deserialize_json_kvs<T>(kvs: &[KeyValue]) -> Result<Vec<(String, T)>>
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
