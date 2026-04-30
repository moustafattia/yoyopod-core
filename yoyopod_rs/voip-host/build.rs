use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    if env::var_os("CARGO_FEATURE_NATIVE_LIBLINPHONE").is_none() {
        return;
    }

    match pkg_config::Config::new().probe("linphone") {
        Ok(_library) => {
            // Some distro pkg-config files expose Liblinphone's dependency set
            // without emitting the main shared library. Direct FFI requires the
            // host binary to link liblinphone itself.
            println!("cargo:rustc-link-lib=linphone");
        }
        Err(error) => {
            panic!(
                "native-liblinphone builds require Liblinphone development libraries discoverable by pkg-config: {error}"
            );
        }
    }
}
