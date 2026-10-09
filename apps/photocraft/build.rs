//! Windows only: embed the app icon and version info (VERSIONINFO) into `photocraft.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PHOTOCRAFT_REQUIRE_WINRES=1` (set by the
//! release workflow) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/photocraft.ico");
    println!("cargo:rerun-if-env-changed=PHOTOCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=RC_PATH");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    if let Ok(path) = std::env::var("RC_PATH")
        && let Some(parent) = std::path::Path::new(&path).parent().and_then(|p| p.to_str())
    {
        res.set_toolkit_path(parent);
    }
    res.set_icon("../../assets/app-icon/photocraft.ico")
        .set("ProductName", "Starframe FITS")
        .set("FileDescription", "Starframe FITS image editor")
        .set("CompanyName", "Starframe FITS contributors")
        .set("LegalCopyright", "Copyright (c) the Starframe FITS authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "StarframeFITS.exe")
        .set("InternalName", "starframe-fits");
    if let Err(e) = res.compile() {
        if std::env::var_os("PHOTOCRAFT_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=photocraft.exe built without icon/version resources: {e}");
    }
}
