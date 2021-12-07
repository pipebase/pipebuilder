#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BuildRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// project manifest version
    #[prost(uint64, tag = "3")]
    pub manifest_version: u64,
    /// target platform
    #[prost(string, tag = "4")]
    pub target_platform: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BuildResponse {
    /// version: build version
    #[prost(uint64, tag = "1")]
    pub version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelBuildRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// project build version
    #[prost(uint64, tag = "3")]
    pub build_version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelBuildResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScanBuildRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BuildMetadataKey {
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
pub struct ScanBuildResponse {
    #[prost(message, repeated, tag = "1")]
    pub builds: ::prost::alloc::vec::Vec<BuildMetadataKey>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBuildLogRequest {
    /// project namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// project id
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// project build version
    #[prost(uint64, tag = "3")]
    pub build_version: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBuildLogResponse {
    /// log context
    #[prost(bytes = "vec", tag = "1")]
    pub buffer: ::prost::alloc::vec::Vec<u8>,
}
#[doc = r" Generated client implementations."]
pub mod builder_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct BuilderClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BuilderClient<tonic::transport::Channel> {
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
    impl<T> BuilderClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + Sync + 'static,
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
        ) -> BuilderClient<InterceptedService<T, F>>
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
            BuilderClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn build(
            &mut self,
            request: impl tonic::IntoRequest<super::BuildRequest>,
        ) -> Result<tonic::Response<super::BuildResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/build.Builder/Build");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn cancel_build(
            &mut self,
            request: impl tonic::IntoRequest<super::CancelBuildRequest>,
        ) -> Result<tonic::Response<super::CancelBuildResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/build.Builder/CancelBuild");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_build_log(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBuildLogRequest>,
        ) -> Result<tonic::Response<super::GetBuildLogResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/build.Builder/GetBuildLog");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn scan_build(
            &mut self,
            request: impl tonic::IntoRequest<super::ScanBuildRequest>,
        ) -> Result<tonic::Response<super::ScanBuildResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/build.Builder/ScanBuild");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod builder_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with BuilderServer."]
    #[async_trait]
    pub trait Builder: Send + Sync + 'static {
        async fn build(
            &self,
            request: tonic::Request<super::BuildRequest>,
        ) -> Result<tonic::Response<super::BuildResponse>, tonic::Status>;
        async fn cancel_build(
            &self,
            request: tonic::Request<super::CancelBuildRequest>,
        ) -> Result<tonic::Response<super::CancelBuildResponse>, tonic::Status>;
        async fn get_build_log(
            &self,
            request: tonic::Request<super::GetBuildLogRequest>,
        ) -> Result<tonic::Response<super::GetBuildLogResponse>, tonic::Status>;
        async fn scan_build(
            &self,
            request: tonic::Request<super::ScanBuildRequest>,
        ) -> Result<tonic::Response<super::ScanBuildResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct BuilderServer<T: Builder> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Builder> BuilderServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for BuilderServer<T>
    where
        T: Builder,
        B: Body + Send + Sync + 'static,
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
                "/build.Builder/Build" => {
                    #[allow(non_camel_case_types)]
                    struct BuildSvc<T: Builder>(pub Arc<T>);
                    impl<T: Builder> tonic::server::UnaryService<super::BuildRequest> for BuildSvc<T> {
                        type Response = super::BuildResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BuildRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).build(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BuildSvc(inner);
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
                "/build.Builder/CancelBuild" => {
                    #[allow(non_camel_case_types)]
                    struct CancelBuildSvc<T: Builder>(pub Arc<T>);
                    impl<T: Builder> tonic::server::UnaryService<super::CancelBuildRequest> for CancelBuildSvc<T> {
                        type Response = super::CancelBuildResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CancelBuildRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).cancel_build(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CancelBuildSvc(inner);
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
                "/build.Builder/GetBuildLog" => {
                    #[allow(non_camel_case_types)]
                    struct GetBuildLogSvc<T: Builder>(pub Arc<T>);
                    impl<T: Builder> tonic::server::UnaryService<super::GetBuildLogRequest> for GetBuildLogSvc<T> {
                        type Response = super::GetBuildLogResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBuildLogRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_build_log(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBuildLogSvc(inner);
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
                "/build.Builder/ScanBuild" => {
                    #[allow(non_camel_case_types)]
                    struct ScanBuildSvc<T: Builder>(pub Arc<T>);
                    impl<T: Builder> tonic::server::UnaryService<super::ScanBuildRequest> for ScanBuildSvc<T> {
                        type Response = super::ScanBuildResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ScanBuildRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).scan_build(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ScanBuildSvc(inner);
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
    impl<T: Builder> Clone for BuilderServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Builder> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Builder> tonic::transport::NamedService for BuilderServer<T> {
        const NAME: &'static str = "build.Builder";
    }
}
