use std::process::Command;

fn main() {
    let git_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    let (git_sha, git_available) = match git_output {
        Ok(output) if output.status.success() => {
            let sha = String::from_utf8(output.stdout)
                .map_or_else(|_| "unknown".to_string(), |s| s.trim().to_string());
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
