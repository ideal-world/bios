use std::io::Result;

fn main() -> Result<()> {
    // std::env::set_var("OUT_DIR", "tests/grpc/rust");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let proto_dir = format!("{}/proto", manifest_dir);
    println!("cargo:warning=output dir: {}", std::env::var("OUT_DIR").unwrap());
    poem_grpc_build::Config::new()
    .build_server(true)
    .build_client(false)
    .compile(&[format!("{proto_dir}/nacos_grpc_service.proto")], &[&proto_dir])?;
    Ok(())
}
