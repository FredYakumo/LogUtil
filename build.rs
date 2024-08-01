use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use chrono::{Local};

fn main() {
    // Get build profile
    let profile = std::env::var("PROFILE").unwrap();
    let is_release = profile == "release";
    // Get current time
    let now = Local::now().naive_local();
    let date = now.format("%Y-%m-%d").to_string();

    // Create source rust file to save build info
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version_info.rs");
    let mut f = File::create(dest_path).unwrap();

    // 获取当前的git commit id
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command")
        .stdout;

    let str = String::from_utf8(output).unwrap();

    let git_hash = str.trim();


    // Write version info
    f.write_all(format!("const VERSION: &str = \"Ver beta.{profile} commit id: {} build on {}\"; const IS_RELEASE: bool = {};", git_hash, date.trim(), is_release).as_bytes()).unwrap();
}