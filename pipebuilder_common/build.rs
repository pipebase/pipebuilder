fn main() {
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/api.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/build.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/health.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/schedule.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/manifest.proto"], &["proto"])
        .unwrap();
}
