fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("docs/appicon.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
