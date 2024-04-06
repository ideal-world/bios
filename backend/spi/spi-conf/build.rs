use std::io::Result;

fn main() -> Result<()> {
    // std::env::set_var("OUT_DIR", "tests/grpc/rust");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let proto_dir = format!("{}/proto", manifest_dir);
    poem_grpc_build::Config::new()
        .build_server(true)
        .build_client(false)
        // enable when you need to generate descriptor file
        // .file_descriptor_set_path(format!("{proto_dir}/nacos_grpc_service.desc"))
        .compile(&[format!("{proto_dir}/nacos_grpc_service.proto")], &[&proto_dir])?;
    Ok(())
}
