use std::env;
use std::process::Command;

fn main() {
    // Set the target triple
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_owned());
    println!("cargo:rustc-env=TARGET={target}");

    // Get git hash if available
    if let Ok(output) = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output() {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            println!("cargo:rustc-env=GIT_HASH={git_hash}");
        }
    }

    // Get build date
    if let Ok(date) = Command::new("date").args(["+%Y-%m-%d"]).output() {
        if date.status.success() {
            let build_date = String::from_utf8_lossy(&date.stdout).trim().to_owned();
            println!("cargo:rustc-env=BUILD_DATE={build_date}");
        }
    }

    // Get git version for legacy kibi compatibility
    let version = match Command::new("git").args(["describe", "--tags", "--match=v*"]).output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout[1..]).replacen('-', ".r", 1).replace('-', ".")
        }
        _ => env!("CARGO_PKG_VERSION").into(),
    };
    println!("cargo:rustc-env=KIBI_VERSION={version}");

    // Tell cargo to rerun if git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");
}
