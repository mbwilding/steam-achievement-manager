use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")?;

    let lib_name = match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
            "libsteam_api.so"
        }
        "macos" => {
            println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
            "libsteam_api.dylib"
        }
        "windows" if target_arch == "x86" => "steam_api.dll",
        "windows" => "steam_api64.dll",
        os => return Err(format!("Unsupported target OS: {os}").into()),
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .ok_or("Failed to locate target directory")?;

    let pattern = profile_dir
        .join("build/steamworks-sys-*/out")
        .join(lib_name);
    let src = glob(pattern.to_str().ok_or("Non UTF-8 target path")?)?
        .filter_map(Result::ok)
        .max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
        .ok_or_else(|| format!("{lib_name} not found"))?;

    fs::copy(&src, profile_dir.join(lib_name))?;

    Ok(())
}
