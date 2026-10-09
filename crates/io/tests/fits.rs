//! FITS follows the same import path as desktop Open, CLI and Web.
use photocraft_color::{ColorMode, SampleType};
use photocraft_io::{ExportOptions, export, import};

fn fits() -> Vec<u8> {
    let mut b = Vec::new();
    for c in [
        "SIMPLE  =                    T",
        "BITPIX  =                  -32",
        "NAXIS   =                    2",
        "NAXIS1  =                    2",
        "NAXIS2  =                    2",
        "END",
    ] {
        b.extend_from_slice(format!("{c:80}").as_bytes());
    }
    b.resize(2880, b' ');
    for v in [-0.5f32, 0.25, 0.75, 2.0] {
        b.extend_from_slice(&v.to_be_bytes());
    }
    b.resize(5760, 0);
    b
}

#[test]
fn import_gray_f32_retains_hdr_samples_and_warns() {
    for name in ["nebula.fits", "nebula.fit", "nebula.FTS", "extensionless"] {
        let r = import(name, &fits()).unwrap();
        let d = r.document;
        assert_eq!(d.name, name);
        assert_eq!(d.mode, ColorMode::Grayscale);
        assert_eq!(d.depth, SampleType::F32);
        assert_eq!((d.size.width, d.size.height), (2, 2));
        assert_eq!(d.layers.len(), 2);
        let s = d.layers[0].surface().unwrap();
        assert_eq!(s.pixel(0, 0), [0.75, 1.0]);
        assert_eq!(s.pixel(1, 0), [2.0, 1.0]);
        assert_eq!(s.pixel(0, 1), [-0.5, 1.0]);
        assert!(r.warnings.is_empty());
        for ext in ["pcraft", "tif", "psd"] {
            let saved = export(&d, ext, &ExportOptions::default()).unwrap();
            let back = import(&format!("saved.{ext}"), &saved.bytes).unwrap().document;
            assert_eq!(back.mode, ColorMode::Grayscale);
            assert_eq!(back.depth, SampleType::F32);
            assert_eq!(back.layers[0].surface().unwrap().read_region(back.bounds()), s.read_region(d.bounds()), "{ext}");
        }
        assert!(export(&d, "fits", &ExportOptions::default()).is_err());
    }
}

#[test]
fn fits_display_scaling_is_visible_removable_and_preserves_detector_counts() {
    let mut bytes = fits();
    bytes[80..160].copy_from_slice(format!("{:80}", "BITPIX  =                   16").as_bytes());
    bytes[400..480].copy_from_slice(format!("{:80}", "BZERO   =                32768").as_bytes());
    bytes[480..560].copy_from_slice(format!("{:80}", "END").as_bytes());
    for (i, v) in [934i32, 1185, 1331, 16383].into_iter().enumerate() {
        bytes[2880 + i * 2..2882 + i * 2].copy_from_slice(&((v - 32768) as i16).to_be_bytes());
    }
    let mut doc = import("detector.fit", &bytes).unwrap().document;
    assert_eq!(doc.layers[0].surface().unwrap().pixel(0, 0), [1331.0, 1.0]);
    assert_eq!(doc.layers[1].name, "FITS display scaling");
    assert!(matches!(doc.layers[1].content, photocraft_doc::LayerContent::Adjustment(photocraft_doc::Adjustment::Exposure { exposure: -14.0, .. })));
    let shown = photocraft_compose::render(&doc, photocraft_geom::Rect::from_xywh(0, 0, 1, 1)).px[0][0];
    assert!((shown - 1331.0 / 16384.0).abs() < 1e-6, "{shown}");
    doc.layers[1].visible = false;
    assert_eq!(doc.layers[0].surface().unwrap().pixel(0, 0), [1331.0, 1.0]);
}

#[test]
fn corrupt_fits_returns_an_error_without_a_document() {
    let bytes = fits();
    assert!(import("broken.fits", &bytes[..3000]).is_err());
    assert!(import("broken.fits", &[]).is_err());
}

#[test]
fn import_rgb_planes_retains_colors_and_float_depth_through_save() {
    let mut bytes = fits();
    bytes[160..240].copy_from_slice(format!("{:80}", "NAXIS   =                    3").as_bytes());
    bytes[400..480].copy_from_slice(format!("{:80}", "NAXIS3  =                    3").as_bytes());
    bytes[480..560].copy_from_slice(format!("{:80}", "END").as_bytes());
    for (i, v) in [0.1f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2].into_iter().enumerate() {
        bytes[2880 + i * 4..2884 + i * 4].copy_from_slice(&v.to_be_bytes());
    }
    let doc = import("color.fit", &bytes).unwrap().document;
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.depth, SampleType::F32);
    let surface = doc.layers[0].surface().unwrap();
    assert_eq!(surface.pixel(0, 0), [0.3, 0.7, 1.1, 1.0]);
    assert_eq!(surface.pixel(1, 1), [0.2, 0.6, 1.0, 1.0]);
    for ext in ["pcraft", "tif", "psd"] {
        let saved = export(&doc, ext, &ExportOptions::default()).unwrap();
        let back = import(&format!("color.{ext}"), &saved.bytes).unwrap().document;
        assert_eq!(back.mode, ColorMode::Rgb);
        assert_eq!(back.depth, SampleType::F32);
        assert_eq!(back.layers[0].surface().unwrap().read_region(back.bounds()), surface.read_region(doc.bounds()));
    }
}
