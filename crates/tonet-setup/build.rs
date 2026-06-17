fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        let manifest = std::path::Path::new("windows/app.manifest");
        if manifest.exists() {
            res.set_manifest_file(manifest.to_str().expect("manifest path must be UTF-8"));
        }
        let icon = std::path::Path::new("../tonet/windows/app.ico");
        if icon.exists() {
            res.set_icon(icon.to_str().expect("icon path must be UTF-8"));
        }
        if let Err(e) = res.compile() {
            println!("cargo:warning=winres: {e}");
        }
    }
}
