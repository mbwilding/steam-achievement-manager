use glob::glob;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pattern = "./target/**/build/steamworks-sys-*/out/*";
    let src_path: PathBuf = glob(pattern)?
        .filter_map(Result::ok)
        .next()
        .ok_or("steam_api files not found")?;

    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);
    let target_dir = out_path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or("Failed to locate target directory")?;

    let file_name = src_path.file_name().ok_or("No file name")?;
    let dest_path = target_dir.join(file_name);

    fs::copy(&src_path, &dest_path)?;

    Ok(())
}
