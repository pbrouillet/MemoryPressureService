use std::path::Path;
use std::process::Command;

fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("docs/appicon.ico");
        res.compile().expect("Failed to compile Windows resources");
    }

    build_ui();
}

fn npm(ui_dir: &Path, args: &[&str]) -> std::process::ExitStatus {
    Command::new("cmd")
        .args(["/C", "npm"])
        .args(args)
        .current_dir(ui_dir)
        .status()
        .expect("Failed to run npm. Is Node.js installed and on PATH?")
}

fn build_ui() {
    let ui_dir = Path::new("ui");

    // Rerun when UI sources change or when dist outputs are missing
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/stats.html");
    println!("cargo:rerun-if-changed=ui/settings.html");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-changed=ui/vite.config.ts");
    println!("cargo:rerun-if-changed=ui/dist/stats.html");
    println!("cargo:rerun-if-changed=ui/dist/settings.html");

    // Install dependencies if needed
    if !ui_dir.join("node_modules").exists() {
        let status = npm(ui_dir, &["install"]);
        assert!(status.success(), "npm install failed");
    }

    // Build the UI
    let status = npm(ui_dir, &["run", "build"]);
    assert!(status.success(), "UI build failed");

    // Verify outputs exist
    let stats_html = ui_dir.join("dist").join("stats.html");
    let settings_html = ui_dir.join("dist").join("settings.html");
    assert!(
        stats_html.exists(),
        "UI build did not produce dist/stats.html"
    );
    assert!(
        settings_html.exists(),
        "UI build did not produce dist/settings.html"
    );
}
