fn main() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("stellar_gateway_descriptor.bin"))
        .compile_protos(&["proto/stellar_gateway.proto"], &["proto/"])
        .unwrap();
}
