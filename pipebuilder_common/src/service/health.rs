use crate::grpc::health::{self, health_server};
use tonic::Response;

#[derive(Default)]
pub struct HealthService {}

#[tonic::async_trait]
impl health_server::Health for HealthService {
    async fn health(
        &self,
        _request: tonic::Request<health::HealthRequest>,
    ) -> Result<tonic::Response<health::HealthResponse>, tonic::Status> {
        Ok(Response::new(health::HealthResponse {}))
    }
}
