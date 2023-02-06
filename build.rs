use std::process::Command;

fn main() {
    let mut version = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let git_available = matches!(
        Command::new("git")
            .arg("-v")
            .status()
            .map(|status| status.success()),
        Ok(true)
    );

    if git_available {
        let sha = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .expect("git rev-parse")
            .stdout;
        let sha = String::from_utf8(sha).unwrap();
        let sha = sha.trim();

        let dirty = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .expect("git status")
            .stdout;
        let dirty = String::from_utf8(dirty).unwrap();
        let dirty = if dirty.trim().is_empty() {
            ""
        } else {
            "-dirty"
        };

        version.push_str(&format!(" ({sha}{dirty})"));
    }

    println!("cargo:rustc-env=BBL2CSV_VERSION={version}");
}
