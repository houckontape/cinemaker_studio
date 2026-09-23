fn main() {
    #[cfg(feature = "windows")]
    {
        println!("cargo:rustc-link-lib=EDSDK");
    }

    #[cfg(feature = "macos")]
    {
        println!("cargo:rustc-link-lib=EDSDK");
        println!("cargo:rustc-link-search=framework=EDSDK");
    }
}
