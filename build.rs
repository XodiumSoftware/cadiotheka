use leptos_i18n_build::{Config, TranslationsInfos};
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Generate i18n module from locales/
    let i18n_mod_directory =
        PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR should be set")).join("i18n");
    let cfg = Config::new("en").expect("default locale 'en' should be valid");
    let translations_infos =
        TranslationsInfos::parse(cfg).expect("locales should parse successfully");
    translations_infos.emit_diagnostics();
    translations_infos.rerun_if_locales_changed();
    translations_infos
        .generate_i18n_module(i18n_mod_directory)
        .expect("i18n module should generate successfully");

    let git_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    let (git_sha, git_available) = match git_output {
        Ok(output) if output.status.success() => {
            let sha = String::from_utf8(output.stdout)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            (sha, true)
        }
        _ => {
            println!(
                "cargo:warning=Unable to determine git SHA; using 'unknown' for build version."
            );
            ("unknown".to_string(), false)
        }
    };

    let is_dirty = if git_available {
        Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .is_some_and(|s| !s.trim().is_empty())
    } else {
        false
    };

    let version = if is_dirty {
        format!("{git_sha}-dirty")
    } else {
        git_sha
    };

    println!("cargo:rustc-env=GIT_SHA={version}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}
