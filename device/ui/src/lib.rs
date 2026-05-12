pub mod app;
pub mod framebuffer;
pub mod hardware;
pub mod input;
pub mod lvgl;
pub mod presentation;
pub mod protocol;
pub mod render;
pub mod runtime;
pub mod screens;
pub mod whisplay_panel;
pub mod worker;

#[cfg(test)]
mod architecture_tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn raw_lvgl_ffi_imports_stay_inside_lvgl_module() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut violations = Vec::new();
        for path in rust_files(&src_dir) {
            let relative = path.strip_prefix(&src_dir).unwrap_or(&path);
            let inside_lvgl = relative
                .components()
                .next()
                .is_some_and(|component| component.as_os_str().to_string_lossy() == "lvgl");
            if inside_lvgl {
                continue;
            }

            let contents = fs::read_to_string(&path).expect("reading Rust source");
            let legacy_module_path = ["lvgl", "sys"].join("::");
            let legacy_crate_module_path = ["crate", "lvgl", "sys"].join("::");
            let module_path = ["lvgl", "ffi"].join("::");
            let crate_module_path = ["crate", "lvgl", "ffi"].join("::");
            if contents.contains(&legacy_module_path)
                || contents.contains(&legacy_crate_module_path)
                || contents.contains(&module_path)
                || contents.contains(&crate_module_path)
            {
                violations.push(relative.display().to_string());
            }
        }

        assert!(
            violations.is_empty(),
            "raw LVGL FFI imports outside src/lvgl: {violations:?}"
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
