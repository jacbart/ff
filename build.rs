use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Set basic build information
    println!(
        "cargo:rustc-env=VERGEN_BUILD_TIMESTAMP={}",
        std::env::var("SOURCE_DATE_EPOCH").unwrap_or_else(|_| {
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64)
                .to_string()
        })
    );

    println!(
        "cargo:rustc-env=VERGEN_RUSTC_SEMVER={}",
        std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string())
    );

    println!(
        "cargo:rustc-env=VERGEN_RUSTC_COMMIT_HASH={}",
        std::env::var("RUSTC_COMMIT_HASH").unwrap_or_else(|_| "unknown".to_string())
    );
}
