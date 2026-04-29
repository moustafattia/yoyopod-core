use std::ffi::CStr;

#[test]
fn rust_shim_exports_version_and_last_error_strings() {
    let version = unsafe { yoyopod_liblinphone_shim::yoyopod_liblinphone_version() };
    assert!(!version.is_null());
    let version = unsafe { CStr::from_ptr(version) }.to_string_lossy();
    assert!(version.contains("rust-liblinphone-shim"));

    let error = unsafe { yoyopod_liblinphone_shim::yoyopod_liblinphone_last_error() };
    assert!(!error.is_null());
}

#[test]
fn rust_shim_exports_current_c_shim_abi_surface() {
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_init as unsafe extern "C" fn() -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_shutdown as unsafe extern "C" fn();
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_start
        as unsafe extern "C" fn(
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            i32,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            i32,
            i32,
            i32,
            *const std::os::raw::c_char,
        ) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_stop as unsafe extern "C" fn();
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_iterate as unsafe extern "C" fn();
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_poll_event
        as unsafe extern "C" fn(*mut yoyopod_liblinphone_shim::YoyopodLiblinphoneEvent) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_make_call
        as unsafe extern "C" fn(*const std::os::raw::c_char) -> i32;
    let _ =
        yoyopod_liblinphone_shim::yoyopod_liblinphone_answer_call as unsafe extern "C" fn() -> i32;
    let _ =
        yoyopod_liblinphone_shim::yoyopod_liblinphone_reject_call as unsafe extern "C" fn() -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_hangup as unsafe extern "C" fn() -> i32;
    let _ =
        yoyopod_liblinphone_shim::yoyopod_liblinphone_set_muted as unsafe extern "C" fn(i32) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_send_text_message
        as unsafe extern "C" fn(
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            *mut std::os::raw::c_char,
            u32,
        ) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_start_voice_recording
        as unsafe extern "C" fn(*const std::os::raw::c_char) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_stop_voice_recording
        as unsafe extern "C" fn(*mut i32) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_cancel_voice_recording
        as unsafe extern "C" fn() -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_send_voice_note
        as unsafe extern "C" fn(
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
            i32,
            *const std::os::raw::c_char,
            *mut std::os::raw::c_char,
            u32,
        ) -> i32;
}
