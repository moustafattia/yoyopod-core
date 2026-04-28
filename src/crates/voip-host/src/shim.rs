use crate::config::VoipConfig;
use crate::host::{BackendEvent, CallBackend};
use libloading::Library;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeEvent {
    pub event_type: i32,
    pub registration_state: i32,
    pub call_state: i32,
    pub message_kind: i32,
    pub message_direction: i32,
    pub message_delivery_state: i32,
    pub duration_ms: i32,
    pub unread: i32,
    pub message_id: [c_char; 128],
    pub peer_sip_address: [c_char; 256],
    pub sender_sip_address: [c_char; 256],
    pub recipient_sip_address: [c_char; 256],
    pub local_file_path: [c_char; 512],
    pub mime_type: [c_char; 128],
    pub text: [c_char; 1024],
    pub reason: [c_char; 256],
}

impl Default for NativeEvent {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Debug, Error)]
pub enum ShimError {
    #[error("liblinphone shim path could not be resolved")]
    NotFound,
    #[error("liblinphone shim load failed: {0}")]
    Load(String),
    #[error("liblinphone shim call failed: {0}")]
    Call(String),
    #[error("string contains interior NUL: {0}")]
    InvalidCString(#[from] std::ffi::NulError),
}

type InitFn = unsafe extern "C" fn() -> c_int;
type ShutdownFn = unsafe extern "C" fn();
type StopFn = unsafe extern "C" fn();
type IterateFn = unsafe extern "C" fn();
type PollEventFn = unsafe extern "C" fn(*mut NativeEvent) -> c_int;
type StartFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    i32,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    i32,
    i32,
    i32,
    *const c_char,
) -> c_int;
type SimpleCallFn = unsafe extern "C" fn() -> c_int;
type MakeCallFn = unsafe extern "C" fn(*const c_char) -> c_int;
type SetMutedFn = unsafe extern "C" fn(i32) -> c_int;
type LastErrorFn = unsafe extern "C" fn() -> *const c_char;

pub struct LiblinphoneShim {
    _library: Library,
    init: InitFn,
    shutdown: ShutdownFn,
    start: StartFn,
    stop: StopFn,
    iterate: IterateFn,
    poll_event: PollEventFn,
    make_call: MakeCallFn,
    answer_call: SimpleCallFn,
    reject_call: SimpleCallFn,
    hangup: SimpleCallFn,
    set_muted: SetMutedFn,
    last_error: LastErrorFn,
}

pub struct ShimBackend {
    shim: LiblinphoneShim,
    next_outgoing_call_id: u64,
}

struct StartAudioSettings {
    audio_enabled: c_int,
    mic_gain: c_int,
    output_volume: c_int,
}

impl StartAudioSettings {
    fn from_config(config: &VoipConfig) -> Self {
        Self {
            audio_enabled: 1,
            mic_gain: config.mic_gain,
            output_volume: config.output_volume,
        }
    }
}

pub fn resolve_shim_path(explicit_path: Option<&str>) -> Result<PathBuf, ShimError> {
    if let Some(path) = explicit_path.filter(|value| !value.trim().is_empty()) {
        return Ok(PathBuf::from(path));
    }
    if let Ok(path) = env::var("YOYOPOD_LIBLINPHONE_SHIM_PATH") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    let repo_candidate = Path::new("yoyopod")
        .join("backends")
        .join("voip")
        .join("shim_native")
        .join("build")
        .join(shim_file_name());
    if repo_candidate.exists() {
        return Ok(repo_candidate);
    }
    Err(ShimError::NotFound)
}

fn shim_file_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "yoyopod_liblinphone_shim.dll"
    } else if cfg!(target_os = "macos") {
        "libyoyopod_liblinphone_shim.dylib"
    } else {
        "libyoyopod_liblinphone_shim.so"
    }
}

impl LiblinphoneShim {
    pub unsafe fn load(path: &Path) -> Result<Self, ShimError> {
        let library =
            unsafe { Library::new(path) }.map_err(|error| ShimError::Load(error.to_string()))?;
        unsafe fn symbol<T: Copy>(library: &Library, name: &[u8]) -> Result<T, ShimError> {
            let symbol = unsafe { library.get::<T>(name) }
                .map_err(|error| ShimError::Load(error.to_string()))?;
            Ok(*symbol)
        }
        Ok(Self {
            init: unsafe { symbol(&library, b"yoyopod_liblinphone_init\0") }?,
            shutdown: unsafe { symbol(&library, b"yoyopod_liblinphone_shutdown\0") }?,
            start: unsafe { symbol(&library, b"yoyopod_liblinphone_start\0") }?,
            stop: unsafe { symbol(&library, b"yoyopod_liblinphone_stop\0") }?,
            iterate: unsafe { symbol(&library, b"yoyopod_liblinphone_iterate\0") }?,
            poll_event: unsafe { symbol(&library, b"yoyopod_liblinphone_poll_event\0") }?,
            make_call: unsafe { symbol(&library, b"yoyopod_liblinphone_make_call\0") }?,
            answer_call: unsafe { symbol(&library, b"yoyopod_liblinphone_answer_call\0") }?,
            reject_call: unsafe { symbol(&library, b"yoyopod_liblinphone_reject_call\0") }?,
            hangup: unsafe { symbol(&library, b"yoyopod_liblinphone_hangup\0") }?,
            set_muted: unsafe { symbol(&library, b"yoyopod_liblinphone_set_muted\0") }?,
            last_error: unsafe { symbol(&library, b"yoyopod_liblinphone_last_error\0") }?,
            _library: library,
        })
    }

    pub fn init(&self) -> Result<(), ShimError> {
        self.check(unsafe { (self.init)() })
    }

    pub fn start(&self, config: &VoipConfig) -> Result<(), ShimError> {
        let sip_server = CString::new(config.sip_server.as_str())?;
        let sip_username = CString::new(config.sip_username.as_str())?;
        let sip_password = CString::new(config.sip_password.as_str())?;
        let sip_password_ha1 = CString::new(config.sip_password_ha1.as_str())?;
        let sip_identity = CString::new(config.sip_identity.as_str())?;
        let factory_config_path = CString::new(config.factory_config_path.as_str())?;
        let transport = CString::new(config.transport.as_str())?;
        let stun_server = CString::new(config.stun_server.as_str())?;
        let conference_factory_uri = CString::new(config.conference_factory_uri.as_str())?;
        let file_transfer_server_url = CString::new(config.file_transfer_server_url.as_str())?;
        let lime_server_url = CString::new(config.lime_server_url.as_str())?;
        let playback_dev_id = CString::new(config.playback_dev_id.as_str())?;
        let ringer_dev_id = CString::new(config.ringer_dev_id.as_str())?;
        let capture_dev_id = CString::new(config.capture_dev_id.as_str())?;
        let media_dev_id = CString::new(config.media_dev_id.as_str())?;
        let voice_note_store_dir = CString::new(config.voice_note_store_dir.as_str())?;
        let audio_settings = StartAudioSettings::from_config(config);

        self.check(unsafe {
            (self.start)(
                sip_server.as_ptr(),
                sip_username.as_ptr(),
                sip_password.as_ptr(),
                sip_password_ha1.as_ptr(),
                sip_identity.as_ptr(),
                factory_config_path.as_ptr(),
                transport.as_ptr(),
                stun_server.as_ptr(),
                conference_factory_uri.as_ptr(),
                file_transfer_server_url.as_ptr(),
                lime_server_url.as_ptr(),
                if config.auto_download_incoming_voice_recordings {
                    1
                } else {
                    0
                },
                playback_dev_id.as_ptr(),
                ringer_dev_id.as_ptr(),
                capture_dev_id.as_ptr(),
                media_dev_id.as_ptr(),
                audio_settings.audio_enabled,
                audio_settings.mic_gain,
                audio_settings.output_volume,
                voice_note_store_dir.as_ptr(),
            )
        })
    }

    pub fn shutdown(&self) {
        unsafe { (self.shutdown)() }
    }

    pub fn stop(&self) {
        unsafe { (self.stop)() }
    }

    pub fn iterate(&self) {
        unsafe { (self.iterate)() }
    }

    pub fn make_call(&self, sip_address: &str) -> Result<(), ShimError> {
        let sip_address = CString::new(sip_address)?;
        self.check(unsafe { (self.make_call)(sip_address.as_ptr()) })
    }

    pub fn answer_call(&self) -> Result<(), ShimError> {
        self.check(unsafe { (self.answer_call)() })
    }

    pub fn reject_call(&self) -> Result<(), ShimError> {
        self.check(unsafe { (self.reject_call)() })
    }

    pub fn hangup(&self) -> Result<(), ShimError> {
        self.check(unsafe { (self.hangup)() })
    }

    pub fn set_muted(&self, muted: bool) -> Result<(), ShimError> {
        self.check(unsafe { (self.set_muted)(if muted { 1 } else { 0 }) })
    }

    fn last_error(&self) -> String {
        unsafe {
            let raw = (self.last_error)();
            if raw.is_null() {
                return "unknown liblinphone shim error".to_string();
            }
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    }

    fn check(&self, code: c_int) -> Result<(), ShimError> {
        if code == 0 {
            Ok(())
        } else {
            Err(ShimError::Call(self.last_error()))
        }
    }

    pub fn drain_events(&self) -> Result<Vec<BackendEvent>, ShimError> {
        let mut events = Vec::new();
        loop {
            let mut event = NativeEvent::default();
            let has_event = unsafe { (self.poll_event)(&mut event as *mut NativeEvent) };
            if has_event == 0 {
                break;
            }
            match event.event_type {
                1 => events.push(BackendEvent::RegistrationChanged {
                    state: crate::events::RegistrationState::from_native(event.registration_state)
                        .as_protocol()
                        .to_string(),
                    reason: c_string(&event.reason),
                }),
                2 => events.push(BackendEvent::CallStateChanged {
                    call_id: c_string(&event.peer_sip_address),
                    state: crate::events::CallState::from_native(event.call_state)
                        .as_protocol()
                        .to_string(),
                }),
                3 => events.push(BackendEvent::IncomingCall {
                    call_id: c_string(&event.peer_sip_address),
                    from_uri: c_string(&event.peer_sip_address),
                }),
                4 => events.push(BackendEvent::BackendStopped {
                    reason: c_string(&event.reason),
                }),
                _ => {}
            }
        }
        Ok(events)
    }
}

impl ShimBackend {
    pub unsafe fn load(path: &Path) -> Result<Self, ShimError> {
        Ok(Self {
            shim: unsafe { LiblinphoneShim::load(path) }?,
            next_outgoing_call_id: 1,
        })
    }
}

impl CallBackend for ShimBackend {
    fn start(&mut self, config: &VoipConfig) -> Result<(), String> {
        self.shim.init().map_err(|error| error.to_string())?;
        self.shim.start(config).map_err(|error| error.to_string())
    }

    fn stop(&mut self) {
        self.shim.stop();
        self.shim.shutdown();
    }

    fn iterate(&mut self) -> Result<Vec<BackendEvent>, String> {
        self.shim.iterate();
        self.shim.drain_events().map_err(|error| error.to_string())
    }

    fn make_call(&mut self, sip_address: &str) -> Result<String, String> {
        self.shim
            .make_call(sip_address)
            .map_err(|error| error.to_string())?;
        let call_id = format!("outgoing-{}", self.next_outgoing_call_id);
        self.next_outgoing_call_id += 1;
        Ok(call_id)
    }

    fn answer_call(&mut self) -> Result<(), String> {
        self.shim.answer_call().map_err(|error| error.to_string())
    }

    fn reject_call(&mut self) -> Result<(), String> {
        self.shim.reject_call().map_err(|error| error.to_string())
    }

    fn hangup(&mut self) -> Result<(), String> {
        self.shim.hangup().map_err(|error| error.to_string())
    }

    fn set_muted(&mut self, muted: bool) -> Result<(), String> {
        self.shim
            .set_muted(muted)
            .map_err(|error| error.to_string())
    }
}

pub fn c_string<const N: usize>(buffer: &[c_char; N]) -> String {
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn explicit_path_wins() {
        let path = resolve_shim_path(Some("/tmp/libshim.so")).expect("path");
        assert_eq!(path, PathBuf::from("/tmp/libshim.so"));
    }

    #[test]
    fn start_audio_settings_forward_configured_gain_and_volume() {
        let config = VoipConfig::from_payload(&json!({
            "sip_server": "sip.example.com",
            "sip_identity": "sip:alice@example.com",
            "mic_gain": 42,
            "output_volume": 73
        }))
        .expect("config");

        let settings = StartAudioSettings::from_config(&config);

        assert_eq!(settings.audio_enabled, 1);
        assert_eq!(settings.mic_gain, 42);
        assert_eq!(settings.output_volume, 73);
    }
}
