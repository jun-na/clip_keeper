fn main() {
    // Windows exe にアイコンを埋め込む
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app-icon.ico");
        res.compile().expect("Failed to compile Windows resource");
    }

    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}
