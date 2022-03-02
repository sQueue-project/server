use std::io::Result;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/items.proto");

    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.compile_protos(&["src/items.proto"], &["src/"])?;

    Ok(())
}