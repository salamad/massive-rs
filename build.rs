//! Build script for massive-rs.
//!
//! This script handles OpenAPI code generation when the `codegen` feature is enabled.
//! For normal builds, it simply prints rerun-if-changed directives.
//!
//! # Code Generation Status
//!
//! **WARNING**: The `codegen` feature is currently a placeholder and does NOT generate
//! functional code. All endpoints and models are manually implemented in `src/rest/endpoints/`
//! and `src/models/`. The codegen infrastructure is reserved for future use when the
//! Massive.com OpenAPI specification becomes available.
//!
//! If you enable the `codegen` feature, the build will succeed but only generate
//! empty placeholder files with comments.

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
    // NOTE: This is placeholder infrastructure. Actual code generation from OpenAPI
    // spec is not implemented. All endpoints are manually implemented in src/rest/endpoints/.
    let models_path = Path::new(&out_dir).join("generated_models.rs");
    let requests_path = Path::new(&out_dir).join("generated_requests.rs");

    fs::write(
        &models_path,
        r#"//! Generated models from OpenAPI spec.
//!
//! **WARNING**: This file contains placeholder content only.
//! The `codegen` feature does NOT currently generate functional code.
//! All models are manually implemented in `src/models/` and `src/rest/endpoints/`.
//!
//! This infrastructure is reserved for future use when the Massive.com
//! OpenAPI specification becomes available.
"#,
    )
    .expect("Failed to write generated models");

    fs::write(
        &requests_path,
        r#"//! Generated request builders from OpenAPI spec.
//!
//! **WARNING**: This file contains placeholder content only.
//! The `codegen` feature does NOT currently generate functional code.
//! All request builders are manually implemented in `src/rest/endpoints/`.
//!
//! This infrastructure is reserved for future use when the Massive.com
//! OpenAPI specification becomes available.
"#,
    )
    .expect("Failed to write generated requests");

    println!(
        "cargo:warning=codegen feature enabled but OpenAPI code generation is not implemented. Generated placeholder files in {}",
        out_dir
    );
}
