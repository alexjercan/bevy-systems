use std::{env, process::Command};

fn main() {
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap();
    let debug_enabled = env::var("CARGO_FEATURE_DEBUG").is_ok();

    let version = if debug_enabled {
        let output = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .expect("failed to get git revision");

        let hash = String::from_utf8(output.stdout).unwrap();
        format!("{pkg_version}+{}", hash.trim())
    } else {
        pkg_version
    };

    println!("cargo:rustc-env=APP_VERSION={}", version);
}
