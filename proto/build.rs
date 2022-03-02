use std::io::Result;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/items.proto");

    prost_build::compile_protos(&["src/items.proto"], &["src/"])?;
    Ok(())
}