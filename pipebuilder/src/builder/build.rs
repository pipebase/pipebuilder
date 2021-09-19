use pipebuilder_common::{
    grpc::build::{builder_server::Builder, BuildRequest, BuildResponse},
    Register, VersionBuild, VersionBuildStatus,
};
use tonic::Response;

pub struct BuilderService {
    lease_id: i64,
    register: Register,
}

#[tonic::async_trait]
impl Builder for BuilderService {
    async fn build(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::BuildRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::BuildResponse>, tonic::Status>
    {
        // lock build snapshot with manifest url
        // update latest version, trigger version build
        Ok(Response::new(BuildResponse { version: 0 }))
    }
}
