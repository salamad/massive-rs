//! Build script for massive-rs.
//!
//! This is a minimal build script that ensures cargo reruns
//! the build if the script itself changes.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
