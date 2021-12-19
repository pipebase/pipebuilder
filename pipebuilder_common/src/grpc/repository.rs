#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetManifestRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// project manifest version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetManifestResponse {
    /// manifest binaries
    #[prost(bytes = "vec", tag = "1")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutManifestRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// project manifest binaries
    #[prost(bytes = "vec", tag = "3")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutManifestResponse {
    /// manifest version
    #[prost(uint64, tag = "1")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteManifestRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// manifest version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteManifestResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAppRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// project build version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAppResponse {
    /// app binaries
    #[prost(bytes = "vec", tag = "1")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PostAppRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// project build version
    #[prost(uint64, tag = "3")]
    pub version: u64,
    /// app binaries
    #[prost(bytes = "vec", tag = "4")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PostAppResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteAppRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// app build version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteAppResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutCatalogSchemaRequest {
    /// catalog schema namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// catalog schema id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// catalog schema context
    #[prost(bytes = "vec", tag = "3")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutCatalogSchemaResponse {
    /// catalog schema version
    #[prost(uint64, tag = "1")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCatalogSchemaRequest {
    /// catalog schema namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// catalog schema id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// catalog schema version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCatalogSchemaResponse {
    /// catalog schema binaries
    #[prost(bytes = "vec", tag = "1")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCatalogSchemaRequest {
    /// catalog schema namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// catalog schema id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// catalog schema version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCatalogSchemaResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutCatalogsRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// catalogs context
    #[prost(bytes = "vec", tag = "3")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutCatalogsResponse {
    /// catalogs version
    #[prost(uint64, tag = "1")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCatalogsRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// catalogs version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCatalogsResponse {
    /// catalogs context
    #[prost(bytes = "vec", tag = "1")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCatalogsRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// catalogs version
    #[prost(uint64, tag = "3")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCatalogsResponse {}
#[doc = r" Generated client implementations."]
pub mod repository_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct RepositoryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RepositoryClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> RepositoryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> RepositoryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            RepositoryClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn get_manifest(
            &mut self,
            request: impl tonic::IntoRequest<super::GetManifestRequest>,
        ) -> Result<tonic::Response<super::GetManifestResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/repository.Repository/GetManifest");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn put_manifest(
            &mut self,
            request: impl tonic::IntoRequest<super::PutManifestRequest>,
        ) -> Result<tonic::Response<super::PutManifestResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/repository.Repository/PutManifest");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_manifest(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteManifestRequest>,
        ) -> Result<tonic::Response<super::DeleteManifestResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/repository.Repository/DeleteManifest");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_app(
            &mut self,
            request: impl tonic::IntoRequest<super::GetAppRequest>,
        ) -> Result<tonic::Response<super::GetAppResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/repository.Repository/GetApp");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn post_app(
            &mut self,
            request: impl tonic::IntoRequest<super::PostAppRequest>,
        ) -> Result<tonic::Response<super::PostAppResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/repository.Repository/PostApp");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_app(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteAppRequest>,
        ) -> Result<tonic::Response<super::DeleteAppResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/repository.Repository/DeleteApp");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_catalog_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCatalogSchemaRequest>,
        ) -> Result<tonic::Response<super::GetCatalogSchemaResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/repository.Repository/GetCatalogSchema");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn put_catalog_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::PutCatalogSchemaRequest>,
        ) -> Result<tonic::Response<super::PutCatalogSchemaResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/repository.Repository/PutCatalogSchema");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_catalog_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCatalogSchemaRequest>,
        ) -> Result<tonic::Response<super::DeleteCatalogSchemaResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/repository.Repository/DeleteCatalogSchema");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_catalogs(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCatalogsRequest>,
        ) -> Result<tonic::Response<super::GetCatalogsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/repository.Repository/GetCatalogs");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn put_catalogs(
            &mut self,
            request: impl tonic::IntoRequest<super::PutCatalogsRequest>,
        ) -> Result<tonic::Response<super::PutCatalogsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/repository.Repository/PutCatalogs");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_catalogs(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCatalogsRequest>,
        ) -> Result<tonic::Response<super::DeleteCatalogsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/repository.Repository/DeleteCatalogs");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod repository_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with RepositoryServer."]
    #[async_trait]
    pub trait Repository: Send + Sync + 'static {
        async fn get_manifest(
            &self,
            request: tonic::Request<super::GetManifestRequest>,
        ) -> Result<tonic::Response<super::GetManifestResponse>, tonic::Status>;
        async fn put_manifest(
            &self,
            request: tonic::Request<super::PutManifestRequest>,
        ) -> Result<tonic::Response<super::PutManifestResponse>, tonic::Status>;
        async fn delete_manifest(
            &self,
            request: tonic::Request<super::DeleteManifestRequest>,
        ) -> Result<tonic::Response<super::DeleteManifestResponse>, tonic::Status>;
        async fn get_app(
            &self,
            request: tonic::Request<super::GetAppRequest>,
        ) -> Result<tonic::Response<super::GetAppResponse>, tonic::Status>;
        async fn post_app(
            &self,
            request: tonic::Request<super::PostAppRequest>,
        ) -> Result<tonic::Response<super::PostAppResponse>, tonic::Status>;
        async fn delete_app(
            &self,
            request: tonic::Request<super::DeleteAppRequest>,
        ) -> Result<tonic::Response<super::DeleteAppResponse>, tonic::Status>;
        async fn get_catalog_schema(
            &self,
            request: tonic::Request<super::GetCatalogSchemaRequest>,
        ) -> Result<tonic::Response<super::GetCatalogSchemaResponse>, tonic::Status>;
        async fn put_catalog_schema(
            &self,
            request: tonic::Request<super::PutCatalogSchemaRequest>,
        ) -> Result<tonic::Response<super::PutCatalogSchemaResponse>, tonic::Status>;
        async fn delete_catalog_schema(
            &self,
            request: tonic::Request<super::DeleteCatalogSchemaRequest>,
        ) -> Result<tonic::Response<super::DeleteCatalogSchemaResponse>, tonic::Status>;
        async fn get_catalogs(
            &self,
            request: tonic::Request<super::GetCatalogsRequest>,
        ) -> Result<tonic::Response<super::GetCatalogsResponse>, tonic::Status>;
        async fn put_catalogs(
            &self,
            request: tonic::Request<super::PutCatalogsRequest>,
        ) -> Result<tonic::Response<super::PutCatalogsResponse>, tonic::Status>;
        async fn delete_catalogs(
            &self,
            request: tonic::Request<super::DeleteCatalogsRequest>,
        ) -> Result<tonic::Response<super::DeleteCatalogsResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct RepositoryServer<T: Repository> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Repository> RepositoryServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for RepositoryServer<T>
    where
        T: Repository,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/repository.Repository/GetManifest" => {
                    #[allow(non_camel_case_types)]
                    struct GetManifestSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::GetManifestRequest> for GetManifestSvc<T> {
                        type Response = super::GetManifestResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetManifestRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_manifest(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetManifestSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/PutManifest" => {
                    #[allow(non_camel_case_types)]
                    struct PutManifestSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::PutManifestRequest> for PutManifestSvc<T> {
                        type Response = super::PutManifestResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PutManifestRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).put_manifest(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PutManifestSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/DeleteManifest" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteManifestSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::DeleteManifestRequest>
                        for DeleteManifestSvc<T>
                    {
                        type Response = super::DeleteManifestResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteManifestRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_manifest(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteManifestSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/GetApp" => {
                    #[allow(non_camel_case_types)]
                    struct GetAppSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::GetAppRequest> for GetAppSvc<T> {
                        type Response = super::GetAppResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetAppRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_app(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAppSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/PostApp" => {
                    #[allow(non_camel_case_types)]
                    struct PostAppSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::PostAppRequest> for PostAppSvc<T> {
                        type Response = super::PostAppResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PostAppRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).post_app(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PostAppSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/DeleteApp" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteAppSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::DeleteAppRequest> for DeleteAppSvc<T> {
                        type Response = super::DeleteAppResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteAppRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_app(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteAppSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/GetCatalogSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetCatalogSchemaSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::GetCatalogSchemaRequest>
                        for GetCatalogSchemaSvc<T>
                    {
                        type Response = super::GetCatalogSchemaResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCatalogSchemaRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_catalog_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCatalogSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/PutCatalogSchema" => {
                    #[allow(non_camel_case_types)]
                    struct PutCatalogSchemaSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::PutCatalogSchemaRequest>
                        for PutCatalogSchemaSvc<T>
                    {
                        type Response = super::PutCatalogSchemaResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PutCatalogSchemaRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).put_catalog_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PutCatalogSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/DeleteCatalogSchema" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCatalogSchemaSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository>
                        tonic::server::UnaryService<super::DeleteCatalogSchemaRequest>
                        for DeleteCatalogSchemaSvc<T>
                    {
                        type Response = super::DeleteCatalogSchemaResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCatalogSchemaRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_catalog_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteCatalogSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/GetCatalogs" => {
                    #[allow(non_camel_case_types)]
                    struct GetCatalogsSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::GetCatalogsRequest> for GetCatalogsSvc<T> {
                        type Response = super::GetCatalogsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCatalogsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_catalogs(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCatalogsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/PutCatalogs" => {
                    #[allow(non_camel_case_types)]
                    struct PutCatalogsSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::PutCatalogsRequest> for PutCatalogsSvc<T> {
                        type Response = super::PutCatalogsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PutCatalogsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).put_catalogs(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PutCatalogsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/repository.Repository/DeleteCatalogs" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCatalogsSvc<T: Repository>(pub Arc<T>);
                    impl<T: Repository> tonic::server::UnaryService<super::DeleteCatalogsRequest>
                        for DeleteCatalogsSvc<T>
                    {
                        type Response = super::DeleteCatalogsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCatalogsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_catalogs(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteCatalogsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Repository> Clone for RepositoryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Repository> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Repository> tonic::transport::NamedService for RepositoryServer<T> {
        const NAME: &'static str = "repository.Repository";
    }
}
