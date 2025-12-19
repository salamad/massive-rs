//! Build script for massive-rs.
//!
//! This script handles OpenAPI code generation when the `codegen` feature is enabled.
//! For normal builds, it simply prints rerun-if-changed directives.

use std::path::Path;

fn main() {
    // Always rerun if these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=openapi/spec.json");
    println!("cargo:rerun-if-changed=openapi/checksum.sha256");

    #[cfg(feature = "codegen")]
    {
        generate_from_openapi();
    }
}

#[cfg(feature = "codegen")]
fn generate_from_openapi() {
    use std::fs;

    let spec_path = Path::new("openapi/spec.json");
    let checksum_path = Path::new("openapi/checksum.sha256");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    // Check if spec exists
    if !spec_path.exists() {
        println!(
            "cargo:warning=OpenAPI spec not found at {}. Skipping code generation.",
            spec_path.display()
        );
        return;
    }

    // Read and verify checksum if available
    if checksum_path.exists() {
        let spec_content = fs::read(spec_path).expect("Failed to read OpenAPI spec");
        let expected_hash = fs::read_to_string(checksum_path)
            .expect("Failed to read checksum")
            .trim()
            .to_string();

        // Simple length-based verification (replace with SHA256 in production)
        let computed_check = format!("len:{}", spec_content.len());
        if !expected_hash.starts_with("len:") || computed_check != expected_hash {
            println!(
                "cargo:warning=OpenAPI spec checksum mismatch. Expected: {}, Got: {}",
                expected_hash, computed_check
            );
            println!("cargo:warning=Run `cargo xtask update-spec` to regenerate checksum.");
        }
    }

    // Generate placeholder files
    let models_path = Path::new(&out_dir).join("generated_models.rs");
    let requests_path = Path::new(&out_dir).join("generated_requests.rs");

    fs::write(
        &models_path,
        r#"//! Generated models from OpenAPI spec.
//!
//! This file is auto-generated. Do not edit manually.

// Placeholder for generated models.
// Run with codegen feature and valid OpenAPI spec to generate actual models.
"#,
    )
    .expect("Failed to write generated models");

    fs::write(
        &requests_path,
        r#"//! Generated request builders from OpenAPI spec.
//!
//! This file is auto-generated. Do not edit manually.

// Placeholder for generated request builders.
// Run with codegen feature and valid OpenAPI spec to generate actual requests.
"#,
    )
    .expect("Failed to write generated requests");

    println!(
        "cargo:warning=Generated placeholder model files in {}",
        out_dir
    );
}
