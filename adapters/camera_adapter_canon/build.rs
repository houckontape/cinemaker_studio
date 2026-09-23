use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let edsdk_dir = PathBuf::from(&manifest_dir).join("EDSDK_64");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=EDSDK_64");

    #[cfg(target_os = "windows")]
    {
        let lib_dir = edsdk_dir.join("Library");
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=EDSDK");

        let dll_dir = edsdk_dir.join("Dll");
        if let Ok(out_dir) = env::var("OUT_DIR") {
            let out_path = PathBuf::from(out_dir);
            if let Some(target_dir) = out_path.ancestors().nth(3) {
                copy_dlls(&dll_dir, target_dir);
                let deps_dir = target_dir.join("deps");
                if deps_dir.exists() {
                    copy_dlls(&dll_dir, &deps_dir);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=EDSDK");
        println!("cargo:rustc-link-search=framework={}", edsdk_dir.display());
    }
}

fn copy_dlls(src_dir: &Path, dst_dir: &Path) {
    if !dst_dir.exists() {
        return;
    }
    for dll in &["EDSDK.dll", "EdsImage.dll"] {
        let src = src_dir.join(dll);
        let dst = dst_dir.join(dll);
        if src.exists() {
            let _ = fs::copy(&src, &dst);
        }
    }
}
