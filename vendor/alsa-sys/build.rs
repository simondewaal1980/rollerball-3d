// Local build shim for alsa-sys: the build machine has the ALSA runtime
// library but not the -dev package, so link the bundled headers/library path
// directly instead of asking pkg-config.
fn main() {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let libdir = manifest.join("../alsa/lib");
    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-lib=asound");
    println!("cargo:rerun-if-changed=build.rs");
}
