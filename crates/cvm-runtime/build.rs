extern crate prost_build;

const PROTOS_DIR: &str = "proto";

fn main() -> std::io::Result<()> {
    let mut files = Vec::new();
    let path = std::path::Path::new(PROTOS_DIR);
    let dir = std::fs::read_dir(path)?;

    add_files(dir, &mut files)?;

    if files.len() == 0 {
        println!("No proto files found in {:?}.", path);
    }
    prost_build::compile_protos(&files, &[PROTOS_DIR])
}

fn add_files(dir: std::fs::ReadDir, files: &mut Vec<String>) -> Result<(), std::io::Error> {
    Ok(for entry in dir {
        if let Ok(name) = entry?.file_name().into_string() {
            if !name.starts_with('.') && name.ends_with(".proto") {
                files.push([PROTOS_DIR, "/", name.as_str()].concat())
            }
        }
    })
}
