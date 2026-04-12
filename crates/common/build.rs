use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let proto_root = PathBuf::from("proto");

    let protos: Vec<_> = fs::read_dir(&proto_root)
        .expect("Failed to read proto directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? == "proto" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let mut builder = tonic_prost_build::configure();

    #[cfg(debug_assertions)]
    {
        builder = builder.file_descriptor_set_path(out_dir.join("proto_descriptor.bin"));
    }

    builder.compile_protos(protos.as_slice(), &[proto_root])?;
    Ok(())
}
