fn main() {
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/api.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/builder.proto"], &["proto"])
        .unwrap();
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(&["proto/health.proto"], &["proto"])
        .unwrap();
}
