use std::fmt;

pub const RESOURCE_NODE: &str = "node";
pub const RESOURCE_BUILD_SNAPSHOT: &str = "build/snapshot";
pub const RESOURCE_BUILD_METADATA: &str = "build/metadata";
pub const RESOURCE_MANIFEST_SNAPSHOT: &str = "manifest/snapshot";
pub const RESOURCE_APP_METADATA: &str = "app/metadata";
pub const RESOURCE_MANIFEST_METADATA: &str = "manifest/metadata";
pub const RESOURCE_NAMESPACE: &str = "namespace";
pub const RESOURCE_PROJECT: &str = "project";
pub const RESOURCE_CATALOG_SCHEMA_SNAPSHOT: &str = "catalog-schema/snapshot";
pub const RESOURCE_CATALOG_SCHEMA_METADATA: &str = "catalog-schema/metadata";
pub const RESOURCE_CATALOGS_SNAPSHOT: &str = "catalogs/snapshot";
pub const RESOURCE_CATALOGS_METADATA: &str = "catalogs/metadata";

#[derive(Clone)]
pub enum ResourceType {
    Node,
    AppMetadata,
    BuildSnapshot,
    BuildMetadata,
    ManifestSnapshot,
    ManifestMetadata,
    Namespace,
    Project,
    CatalogSchemaSnapshot,
    CatalogSchemaMetadata,
    CatalogsSnapshot,
    CatalogsMetadata,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Node => write!(f, "{}", RESOURCE_NODE),
            ResourceType::AppMetadata => write!(f, "{}", RESOURCE_APP_METADATA),
            ResourceType::BuildSnapshot => write!(f, "{}", RESOURCE_BUILD_SNAPSHOT),
            ResourceType::BuildMetadata => write!(f, "{}", RESOURCE_BUILD_METADATA),
            ResourceType::ManifestSnapshot => write!(f, "{}", RESOURCE_MANIFEST_SNAPSHOT),
            ResourceType::ManifestMetadata => write!(f, "{}", RESOURCE_MANIFEST_METADATA),
            ResourceType::Namespace => write!(f, "{}", RESOURCE_NAMESPACE),
            ResourceType::Project => write!(f, "{}", RESOURCE_PROJECT),
            ResourceType::CatalogSchemaSnapshot => {
                write!(f, "{}", RESOURCE_CATALOG_SCHEMA_SNAPSHOT)
            }
            ResourceType::CatalogSchemaMetadata => {
                write!(f, "{}", RESOURCE_CATALOG_SCHEMA_METADATA)
            }
            ResourceType::CatalogsSnapshot => write!(f, "{}", RESOURCE_CATALOGS_SNAPSHOT),
            ResourceType::CatalogsMetadata => write!(f, "{}", RESOURCE_CATALOGS_METADATA),
        }
    }
}
pub struct ResourceKeyBuilder<'a> {
    resource: Option<ResourceType>,
    namespace: Option<&'a str>,
    id: Option<&'a str>,
    version: Option<u64>,
    // flag, key for resource locking or not
    lock: bool,
}

impl<'a> ResourceKeyBuilder<'a> {
    pub fn new() -> Self {
        ResourceKeyBuilder {
            resource: None,
            namespace: None,
            id: None,
            version: None,
            lock: false,
        }
    }

    pub fn resource(mut self, resource_type: ResourceType) -> Self {
        self.resource = Some(resource_type);
        self
    }

    pub fn namespace(mut self, namespace: &'a str) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn version(mut self, version: u64) -> Self {
        self.version = Some(version);
        self
    }

    pub fn lock(mut self, lock: bool) -> Self {
        self.lock = lock;
        self
    }

    pub fn build(self) -> String {
        let mut key = String::from("/pipebuilder");
        if self.lock {
            // lock name for resource locking
            key = format!("{}/lock", key)
        }
        let resource = self.resource.expect("resource undefined");
        key = format!("{}/{}", key, resource);
        key = match self.namespace {
            Some(namespace) => format!("{}/{}", key, namespace),
            None => key,
        };
        key = match self.id {
            Some(id) => format!("{}/{}", key, id),
            None => key,
        };
        match self.version {
            Some(version) => format!("{}/{}", key, version),
            None => key,
        }
    }
}

impl<'a> Default for ResourceKeyBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

// remove '/resource/namespace/' and return id/<suffix> given a key
pub fn remove_resource_namespace<'a, R>(origin_key: &'a str, namespace: &str) -> &'a str
where
    R: Resource,
{
    let prefix_key = ResourceKeyBuilder::new()
        .resource(R::ty())
        .namespace(namespace)
        .build();
    let pattern = format!("{}/", prefix_key);
    origin_key
        .strip_prefix(pattern.as_str())
        .unwrap_or_else(|| {
            panic!(
                "key '{}' not start with '/{}/{}/'",
                origin_key,
                R::ty(),
                namespace
            )
        })
}

// remove '/resource/' and return suffix
pub fn remove_resource<R>(origin_key: &'_ str) -> &'_ str
where
    R: Resource,
{
    let prefix_key = ResourceKeyBuilder::new().resource(R::ty()).build();
    let pattern = format!("{}/", prefix_key);
    origin_key
        .strip_prefix(pattern.as_str())
        .unwrap_or_else(|| panic!("key '{}' not start with '/{}/'", origin_key, R::ty()))
}

#[derive(Clone, Copy)]
pub struct BlobDescriptor<'a>(pub &'a str, pub &'a str, pub u64);

impl<'a> BlobDescriptor<'a> {
    pub fn into_tuple(self) -> (&'a str, &'a str, u64) {
        (self.0, self.1, self.2)
    }
}

#[derive(Clone, Copy)]
pub struct SnapshotDescriptor<'a>(pub &'a str, pub &'a str);

impl<'a> SnapshotDescriptor<'a> {
    pub fn into_tuple(self) -> (&'a str, &'a str) {
        (self.0, self.1)
    }
}

// snapshot resource
pub trait Snapshot: Default {
    fn incr_version(&mut self);
    fn get_version(&self) -> u64;
}

pub trait Resource {
    fn ty() -> ResourceType;
}

// metadata + data
pub trait BlobResource {
    fn incr_usage(&mut self);
    fn new(size: usize) -> Self;
}
