pub mod app;
pub mod hardware;
pub mod input;
pub mod presentation;
pub mod render;
pub mod transport;

#[cfg(test)]
mod architecture_tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn raw_lvgl_ffi_imports_stay_inside_render_lvgl_module() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut violations = Vec::new();
        for path in rust_files(&src_dir) {
            let relative = path.strip_prefix(&src_dir).unwrap_or(&path);
            let components = relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            let inside_render_lvgl = components
                .first()
                .is_some_and(|component| component == "render")
                && components
                    .get(1)
                    .is_some_and(|component| component == "lvgl");
            if inside_render_lvgl {
                continue;
            }

            let contents = fs::read_to_string(&path).expect("reading Rust source");
            let old_module_path = ["lvgl", "sys"].join("::");
            let old_crate_module_path = ["crate", "lvgl", "sys"].join("::");
            let module_path = ["lvgl", "ffi"].join("::");
            let crate_module_path = ["crate", "lvgl", "ffi"].join("::");
            let render_module_path = ["render", "lvgl", "ffi"].join("::");
            let render_crate_module_path = ["crate", "render", "lvgl", "ffi"].join("::");
            if contents.contains(&old_module_path)
                || contents.contains(&old_crate_module_path)
                || contents.contains(&module_path)
                || contents.contains(&crate_module_path)
                || contents.contains(&render_module_path)
                || contents.contains(&render_crate_module_path)
            {
                violations.push(relative.display().to_string());
            }
        }

        assert!(
            violations.is_empty(),
            "raw LVGL FFI imports outside src/render/lvgl: {violations:?}"
        );
    }

    #[test]
    fn no_old_top_level_lvgl_module_remains() {
        let old_lvgl_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lvgl");
        assert!(
            !old_lvgl_dir.exists(),
            "src/lvgl module must not remain; LVGL implementation belongs in src/render/lvgl"
        );
    }

    #[test]
    fn transport_layer_does_not_import_render_layer() {
        let transport_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transport");
        let mut violations = Vec::new();
        for path in rust_files(&transport_dir) {
            let contents = fs::read_to_string(&path).expect("reading Rust source");
            if contents.contains("crate::render") {
                violations.push(path.display().to_string());
            }
        }

        assert!(
            violations.is_empty(),
            "transport must not import render-layer APIs: {violations:?}"
        );
    }

    #[test]
    fn removed_root_modules_do_not_return() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for filename in [
            "worker.rs",
            "input.rs",
            "whisplay_panel.rs",
            "framebuffer.rs",
        ] {
            let path = src_dir.join(filename);
            assert!(
                !path.exists(),
                "{filename} must stay in its audit-aligned module directory"
            );
        }
    }

    #[test]
    fn native_scene_identity_and_controller_dispatch_stay_registry_owned() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut violations = Vec::new();
        let forbidden = [
            ["Native", "Scene", "Key"].join(""),
            ["pub trait ", "Screen", "Controller"].join(""),
            ["Box<dyn ", "Screen", "Controller"].join(""),
            ["Controller", "Adapter"].join(""),
        ];
        for path in rust_files(&src_dir) {
            let relative = path.strip_prefix(&src_dir).unwrap_or(&path);
            let contents = fs::read_to_string(&path).expect("reading Rust source");
            for forbidden in &forbidden {
                if contents.contains(forbidden) {
                    violations.push(format!("{} contains {forbidden}", relative.display()));
                }
            }
        }

        assert!(
            violations.is_empty(),
            "legacy scene/controller dispatch names returned: {violations:?}"
        );
    }

    fn rust_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        collect_rust_files(root, &mut files);
        files
    }

    fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(path).expect("reading source directory");
        for entry in entries {
            let path = entry.expect("reading source directory entry").path();
            if path.is_dir() {
                collect_rust_files(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
}
