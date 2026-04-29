use libloading::Library;
use std::ffi::OsString;
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::sync::Arc;

#[repr(C)]
pub struct LinphoneFactory {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneCore {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneCoreCbs {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneAccount {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneAccountCbs {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneAccountParams {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneAuthInfo {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneAddress {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneCall {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneCallParams {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneChatRoom {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneChatRoomCbs {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneChatMessage {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneChatMessageCbs {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneContent {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneEventLog {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneImNotifPolicy {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneNatPolicy {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneRecorder {
    _private: [u8; 0],
}
#[repr(C)]
pub struct LinphoneRecorderParams {
    _private: [u8; 0],
}

pub type CoreCallStateChangedCb =
    Option<unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneCall, c_int, *const c_char)>;
pub type CoreMessageReceivedCb = Option<
    unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneChatRoom, *mut LinphoneChatMessage),
>;
pub type CoreMessageUnableDecryptCb = Option<
    unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneChatRoom, *mut LinphoneChatMessage),
>;
pub type AccountRegistrationChangedCb =
    Option<unsafe extern "C" fn(*mut LinphoneAccount, c_int, *const c_char)>;
pub type ChatMessageStateChangedCb = Option<unsafe extern "C" fn(*mut LinphoneChatMessage, c_int)>;
pub type ChatRoomMessageReceivedCb =
    Option<unsafe extern "C" fn(*mut LinphoneChatRoom, *mut LinphoneChatMessage)>;
pub type ChatRoomMessagesReceivedCb =
    Option<unsafe extern "C" fn(*mut LinphoneChatRoom, *const c_void)>;
pub type ChatRoomEventLogReceivedCb =
    Option<unsafe extern "C" fn(*mut LinphoneChatRoom, *mut LinphoneEventLog)>;

pub struct LinphoneApi {
    _library: Library,
    pub factory_get: unsafe extern "C" fn() -> *mut LinphoneFactory,
    pub factory_create_core_3: unsafe extern "C" fn(
        *mut LinphoneFactory,
        *const c_char,
        *const c_char,
        *mut c_void,
    ) -> *mut LinphoneCore,
    pub factory_create_core_cbs: unsafe extern "C" fn(*mut LinphoneFactory) -> *mut LinphoneCoreCbs,
    pub factory_create_chat_room_cbs:
        unsafe extern "C" fn(*mut LinphoneFactory) -> *mut LinphoneChatRoomCbs,
    pub factory_create_address:
        unsafe extern "C" fn(*mut LinphoneFactory, *const c_char) -> *mut LinphoneAddress,
    pub factory_create_auth_info_2: unsafe extern "C" fn(
        *mut LinphoneFactory,
        *const c_char,
        *const c_char,
        *const c_char,
        *const c_char,
        *const c_char,
        *const c_char,
        *const c_char,
    ) -> *mut LinphoneAuthInfo,
    pub core_cbs_set_call_state_changed:
        unsafe extern "C" fn(*mut LinphoneCoreCbs, CoreCallStateChangedCb),
    pub core_cbs_set_message_received:
        unsafe extern "C" fn(*mut LinphoneCoreCbs, CoreMessageReceivedCb),
    pub core_cbs_set_message_received_unable_decrypt:
        Option<unsafe extern "C" fn(*mut LinphoneCoreCbs, CoreMessageUnableDecryptCb)>,
    pub core_add_callbacks: unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneCoreCbs),
    pub core_start: unsafe extern "C" fn(*mut LinphoneCore) -> c_int,
    pub core_stop: unsafe extern "C" fn(*mut LinphoneCore),
    pub core_unref: unsafe extern "C" fn(*mut LinphoneCore),
    pub core_iterate: unsafe extern "C" fn(*mut LinphoneCore),
    pub core_enable_chat: unsafe extern "C" fn(*mut LinphoneCore),
    pub core_set_playback_device: unsafe extern "C" fn(*mut LinphoneCore, *const c_char),
    pub core_set_ringer_device: unsafe extern "C" fn(*mut LinphoneCore, *const c_char),
    pub core_set_capture_device: unsafe extern "C" fn(*mut LinphoneCore, *const c_char),
    pub core_set_media_device: unsafe extern "C" fn(*mut LinphoneCore, *const c_char),
    pub core_enable_echo_cancellation: unsafe extern "C" fn(*mut LinphoneCore, c_int),
    pub core_set_mic_gain_db: unsafe extern "C" fn(*mut LinphoneCore, c_float),
    pub core_set_playback_gain_db: unsafe extern "C" fn(*mut LinphoneCore, c_float),
    pub core_set_audio_port_range: unsafe extern "C" fn(*mut LinphoneCore, c_int, c_int),
    pub core_set_video_port_range: unsafe extern "C" fn(*mut LinphoneCore, c_int, c_int),
    pub core_create_nat_policy: unsafe extern "C" fn(*mut LinphoneCore) -> *mut LinphoneNatPolicy,
    pub core_set_nat_policy: unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneNatPolicy),
    pub core_set_stun_server: unsafe extern "C" fn(*mut LinphoneCore, *const c_char),
    pub core_set_file_transfer_server: unsafe extern "C" fn(*mut LinphoneCore, *const c_char),
    pub core_enable_lime_x3dh: Option<unsafe extern "C" fn(*mut LinphoneCore, c_int)>,
    pub core_get_im_notif_policy:
        Option<unsafe extern "C" fn(*mut LinphoneCore) -> *mut LinphoneImNotifPolicy>,
    pub core_add_linphone_spec: Option<unsafe extern "C" fn(*mut LinphoneCore, *const c_char)>,
    pub core_set_chat_messages_aggregation_enabled:
        Option<unsafe extern "C" fn(*mut LinphoneCore, c_int)>,
    pub core_enable_auto_download_voice_recordings:
        Option<unsafe extern "C" fn(*mut LinphoneCore, c_int)>,
    pub core_create_account_params:
        unsafe extern "C" fn(*mut LinphoneCore) -> *mut LinphoneAccountParams,
    pub core_create_account:
        unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneAccountParams) -> *mut LinphoneAccount,
    pub core_add_account: unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneAccount) -> c_int,
    pub core_set_default_account: unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneAccount),
    pub core_add_auth_info: unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneAuthInfo),
    pub core_create_call_params:
        unsafe extern "C" fn(*mut LinphoneCore, *mut LinphoneCall) -> *mut LinphoneCallParams,
    pub core_invite_address_with_params: unsafe extern "C" fn(
        *mut LinphoneCore,
        *mut LinphoneAddress,
        *mut LinphoneCallParams,
    ) -> *mut LinphoneCall,
    pub core_get_chat_room_from_uri:
        unsafe extern "C" fn(*mut LinphoneCore, *const c_char) -> *mut LinphoneChatRoom,
    pub core_create_recorder_params:
        Option<unsafe extern "C" fn(*mut LinphoneCore) -> *mut LinphoneRecorderParams>,
    pub core_create_recorder: Option<
        unsafe extern "C" fn(
            *mut LinphoneCore,
            *mut LinphoneRecorderParams,
        ) -> *mut LinphoneRecorder,
    >,
    pub account_params_set_server_address:
        unsafe extern "C" fn(*mut LinphoneAccountParams, *mut LinphoneAddress) -> c_int,
    pub account_params_set_identity_address:
        unsafe extern "C" fn(*mut LinphoneAccountParams, *mut LinphoneAddress) -> c_int,
    pub account_params_enable_register: unsafe extern "C" fn(*mut LinphoneAccountParams, c_int),
    pub account_params_enable_cpim_in_basic_chat_room:
        Option<unsafe extern "C" fn(*mut LinphoneAccountParams, c_int)>,
    pub account_params_set_conference_factory_address:
        Option<unsafe extern "C" fn(*mut LinphoneAccountParams, *mut LinphoneAddress)>,
    pub account_params_set_audio_video_conference_factory_address:
        Option<unsafe extern "C" fn(*mut LinphoneAccountParams, *mut LinphoneAddress)>,
    pub account_params_set_file_transfer_server:
        Option<unsafe extern "C" fn(*mut LinphoneAccountParams, *const c_char)>,
    pub account_params_set_lime_server_url:
        Option<unsafe extern "C" fn(*mut LinphoneAccountParams, *const c_char)>,
    pub account_cbs_new: unsafe extern "C" fn() -> *mut LinphoneAccountCbs,
    pub account_cbs_set_registration_state_changed:
        unsafe extern "C" fn(*mut LinphoneAccountCbs, AccountRegistrationChangedCb),
    pub account_add_callbacks: unsafe extern "C" fn(*mut LinphoneAccount, *mut LinphoneAccountCbs),
    pub account_unref: unsafe extern "C" fn(*mut LinphoneAccount),
    pub account_cbs_unref: unsafe extern "C" fn(*mut LinphoneAccountCbs),
    pub account_params_unref: unsafe extern "C" fn(*mut LinphoneAccountParams),
    pub address_get_username: unsafe extern "C" fn(*const LinphoneAddress) -> *const c_char,
    pub address_get_domain: unsafe extern "C" fn(*const LinphoneAddress) -> *const c_char,
    pub address_unref: unsafe extern "C" fn(*mut LinphoneAddress),
    pub auth_info_unref: unsafe extern "C" fn(*mut LinphoneAuthInfo),
    pub call_params_unref: unsafe extern "C" fn(*mut LinphoneCallParams),
    pub call_get_remote_address: unsafe extern "C" fn(*mut LinphoneCall) -> *const LinphoneAddress,
    pub call_accept: unsafe extern "C" fn(*mut LinphoneCall) -> c_int,
    pub call_decline: unsafe extern "C" fn(*mut LinphoneCall, c_int) -> c_int,
    pub call_terminate: unsafe extern "C" fn(*mut LinphoneCall) -> c_int,
    pub call_set_microphone_muted: unsafe extern "C" fn(*mut LinphoneCall, c_int),
    pub chat_room_add_callbacks:
        unsafe extern "C" fn(*mut LinphoneChatRoom, *mut LinphoneChatRoomCbs),
    pub chat_room_create_message_from_utf8:
        unsafe extern "C" fn(*mut LinphoneChatRoom, *const c_char) -> *mut LinphoneChatMessage,
    pub chat_room_create_voice_recording_message: Option<
        unsafe extern "C" fn(
            *mut LinphoneChatRoom,
            *mut LinphoneRecorder,
        ) -> *mut LinphoneChatMessage,
    >,
    pub chat_room_cbs_set_message_received:
        unsafe extern "C" fn(*mut LinphoneChatRoomCbs, ChatRoomMessageReceivedCb),
    pub chat_room_cbs_set_messages_received:
        Option<unsafe extern "C" fn(*mut LinphoneChatRoomCbs, ChatRoomMessagesReceivedCb)>,
    pub chat_room_cbs_set_chat_message_received:
        Option<unsafe extern "C" fn(*mut LinphoneChatRoomCbs, ChatRoomEventLogReceivedCb)>,
    pub chat_message_cbs_new: unsafe extern "C" fn() -> *mut LinphoneChatMessageCbs,
    pub chat_message_cbs_unref: unsafe extern "C" fn(*mut LinphoneChatMessageCbs),
    pub chat_message_cbs_set_msg_state_changed:
        unsafe extern "C" fn(*mut LinphoneChatMessageCbs, ChatMessageStateChangedCb),
    pub chat_message_add_callbacks:
        unsafe extern "C" fn(*mut LinphoneChatMessage, *mut LinphoneChatMessageCbs),
    pub chat_message_send: unsafe extern "C" fn(*mut LinphoneChatMessage),
    pub chat_message_get_message_id:
        unsafe extern "C" fn(*mut LinphoneChatMessage) -> *const c_char,
    pub chat_message_get_user_data: unsafe extern "C" fn(*mut LinphoneChatMessage) -> *mut c_void,
    pub chat_message_set_user_data: unsafe extern "C" fn(*mut LinphoneChatMessage, *mut c_void),
    pub chat_message_get_utf8_text:
        Option<unsafe extern "C" fn(*const LinphoneChatMessage) -> *const c_char>,
    pub chat_message_get_text:
        Option<unsafe extern "C" fn(*const LinphoneChatMessage) -> *const c_char>,
    pub chat_message_get_file_transfer_information:
        unsafe extern "C" fn(*mut LinphoneChatMessage) -> *mut LinphoneContent,
    pub chat_message_get_state: unsafe extern "C" fn(*mut LinphoneChatMessage) -> c_int,
    pub chat_message_state_to_string: unsafe extern "C" fn(c_int) -> *const c_char,
    pub chat_message_is_outgoing: unsafe extern "C" fn(*const LinphoneChatMessage) -> c_int,
    pub chat_message_is_read: unsafe extern "C" fn(*mut LinphoneChatMessage) -> c_int,
    pub chat_message_get_peer_address:
        unsafe extern "C" fn(*mut LinphoneChatMessage) -> *const LinphoneAddress,
    pub chat_message_get_from_address:
        unsafe extern "C" fn(*mut LinphoneChatMessage) -> *const LinphoneAddress,
    pub chat_message_get_to_address:
        unsafe extern "C" fn(*mut LinphoneChatMessage) -> *const LinphoneAddress,
    pub chat_message_download_content:
        unsafe extern "C" fn(*mut LinphoneChatMessage, *mut LinphoneContent),
    pub content_get_type: unsafe extern "C" fn(*const LinphoneContent) -> *const c_char,
    pub content_get_subtype: unsafe extern "C" fn(*const LinphoneContent) -> *const c_char,
    pub content_get_file_path: unsafe extern "C" fn(*const LinphoneContent) -> *const c_char,
    pub content_set_file_path: unsafe extern "C" fn(*mut LinphoneContent, *const c_char),
    pub core_cbs_unref: unsafe extern "C" fn(*mut LinphoneCoreCbs),
    pub chat_room_cbs_unref: unsafe extern "C" fn(*mut LinphoneChatRoomCbs),
    pub event_log_get_chat_message:
        Option<unsafe extern "C" fn(*mut LinphoneEventLog) -> *mut LinphoneChatMessage>,
    pub im_notif_policy_enable_all: Option<unsafe extern "C" fn(*mut LinphoneImNotifPolicy)>,
    pub nat_policy_enable_stun: unsafe extern "C" fn(*mut LinphoneNatPolicy, c_int),
    pub nat_policy_enable_ice: unsafe extern "C" fn(*mut LinphoneNatPolicy, c_int),
    pub nat_policy_set_stun_server: unsafe extern "C" fn(*mut LinphoneNatPolicy, *const c_char),
    pub nat_policy_unref: unsafe extern "C" fn(*mut LinphoneNatPolicy),
    pub recorder_params_set_file_format:
        Option<unsafe extern "C" fn(*mut LinphoneRecorderParams, c_int)>,
    pub recorder_params_unref: Option<unsafe extern "C" fn(*mut LinphoneRecorderParams)>,
    pub recorder_open: Option<unsafe extern "C" fn(*mut LinphoneRecorder, *const c_char) -> c_int>,
    pub recorder_start: Option<unsafe extern "C" fn(*mut LinphoneRecorder) -> c_int>,
    pub recorder_pause: Option<unsafe extern "C" fn(*mut LinphoneRecorder) -> c_int>,
    pub recorder_get_duration: Option<unsafe extern "C" fn(*mut LinphoneRecorder) -> c_int>,
    pub recorder_close: Option<unsafe extern "C" fn(*mut LinphoneRecorder) -> c_int>,
    pub recorder_unref: Option<unsafe extern "C" fn(*mut LinphoneRecorder)>,
    pub core_get_version: unsafe extern "C" fn() -> *const c_char,
    pub registration_state_to_string: Option<unsafe extern "C" fn(c_int) -> *const c_char>,
    pub call_state_to_string: Option<unsafe extern "C" fn(c_int) -> *const c_char>,
}

impl LinphoneApi {
    pub unsafe fn load() -> Result<Arc<Self>, String> {
        let library = load_library()?;
        unsafe fn symbol<T: Copy>(library: &Library, name: &[u8]) -> Result<T, String> {
            let symbol = unsafe { library.get::<T>(name) }
                .map_err(|error| format!("missing {}: {error}", symbol_name(name)))?;
            Ok(*symbol)
        }
        unsafe fn optional_symbol<T: Copy>(library: &Library, name: &[u8]) -> Option<T> {
            unsafe { library.get::<T>(name) }.ok().map(|symbol| *symbol)
        }

        Ok(Arc::new(Self {
            factory_get: unsafe { symbol(&library, b"linphone_factory_get\0") }?,
            factory_create_core_3: unsafe {
                symbol(&library, b"linphone_factory_create_core_3\0")
            }?,
            factory_create_core_cbs: unsafe {
                symbol(&library, b"linphone_factory_create_core_cbs\0")
            }?,
            factory_create_chat_room_cbs: unsafe {
                symbol(&library, b"linphone_factory_create_chat_room_cbs\0")
            }?,
            factory_create_address: unsafe {
                symbol(&library, b"linphone_factory_create_address\0")
            }?,
            factory_create_auth_info_2: unsafe {
                symbol(&library, b"linphone_factory_create_auth_info_2\0")
            }?,
            core_cbs_set_call_state_changed: unsafe {
                symbol(&library, b"linphone_core_cbs_set_call_state_changed\0")
            }?,
            core_cbs_set_message_received: unsafe {
                symbol(&library, b"linphone_core_cbs_set_message_received\0")
            }?,
            core_cbs_set_message_received_unable_decrypt: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_core_cbs_set_message_received_unable_decrypt\0",
                )
            },
            core_add_callbacks: unsafe { symbol(&library, b"linphone_core_add_callbacks\0") }?,
            core_start: unsafe { symbol(&library, b"linphone_core_start\0") }?,
            core_stop: unsafe { symbol(&library, b"linphone_core_stop\0") }?,
            core_unref: unsafe { symbol(&library, b"linphone_core_unref\0") }?,
            core_iterate: unsafe { symbol(&library, b"linphone_core_iterate\0") }?,
            core_enable_chat: unsafe { symbol(&library, b"linphone_core_enable_chat\0") }?,
            core_set_playback_device: unsafe {
                symbol(&library, b"linphone_core_set_playback_device\0")
            }?,
            core_set_ringer_device: unsafe {
                symbol(&library, b"linphone_core_set_ringer_device\0")
            }?,
            core_set_capture_device: unsafe {
                symbol(&library, b"linphone_core_set_capture_device\0")
            }?,
            core_set_media_device: unsafe {
                symbol(&library, b"linphone_core_set_media_device\0")
            }?,
            core_enable_echo_cancellation: unsafe {
                symbol(&library, b"linphone_core_enable_echo_cancellation\0")
            }?,
            core_set_mic_gain_db: unsafe { symbol(&library, b"linphone_core_set_mic_gain_db\0") }?,
            core_set_playback_gain_db: unsafe {
                symbol(&library, b"linphone_core_set_playback_gain_db\0")
            }?,
            core_set_audio_port_range: unsafe {
                symbol(&library, b"linphone_core_set_audio_port_range\0")
            }?,
            core_set_video_port_range: unsafe {
                symbol(&library, b"linphone_core_set_video_port_range\0")
            }?,
            core_create_nat_policy: unsafe {
                symbol(&library, b"linphone_core_create_nat_policy\0")
            }?,
            core_set_nat_policy: unsafe { symbol(&library, b"linphone_core_set_nat_policy\0") }?,
            core_set_stun_server: unsafe { symbol(&library, b"linphone_core_set_stun_server\0") }?,
            core_set_file_transfer_server: unsafe {
                symbol(&library, b"linphone_core_set_file_transfer_server\0")
            }?,
            core_enable_lime_x3dh: unsafe {
                optional_symbol(&library, b"linphone_core_enable_lime_x3dh\0")
            },
            core_get_im_notif_policy: unsafe {
                optional_symbol(&library, b"linphone_core_get_im_notif_policy\0")
            },
            core_add_linphone_spec: unsafe {
                optional_symbol(&library, b"linphone_core_add_linphone_spec\0")
            },
            core_set_chat_messages_aggregation_enabled: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_core_set_chat_messages_aggregation_enabled\0",
                )
            },
            core_enable_auto_download_voice_recordings: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_core_enable_auto_download_voice_recordings\0",
                )
            },
            core_create_account_params: unsafe {
                symbol(&library, b"linphone_core_create_account_params\0")
            }?,
            core_create_account: unsafe { symbol(&library, b"linphone_core_create_account\0") }?,
            core_add_account: unsafe { symbol(&library, b"linphone_core_add_account\0") }?,
            core_set_default_account: unsafe {
                symbol(&library, b"linphone_core_set_default_account\0")
            }?,
            core_add_auth_info: unsafe { symbol(&library, b"linphone_core_add_auth_info\0") }?,
            core_create_call_params: unsafe {
                symbol(&library, b"linphone_core_create_call_params\0")
            }?,
            core_invite_address_with_params: unsafe {
                symbol(&library, b"linphone_core_invite_address_with_params\0")
            }?,
            core_get_chat_room_from_uri: unsafe {
                symbol(&library, b"linphone_core_get_chat_room_from_uri\0")
            }?,
            core_create_recorder_params: unsafe {
                optional_symbol(&library, b"linphone_core_create_recorder_params\0")
            },
            core_create_recorder: unsafe {
                optional_symbol(&library, b"linphone_core_create_recorder\0")
            },
            account_params_set_server_address: unsafe {
                symbol(&library, b"linphone_account_params_set_server_address\0")
            }?,
            account_params_set_identity_address: unsafe {
                symbol(&library, b"linphone_account_params_set_identity_address\0")
            }?,
            account_params_enable_register: unsafe {
                symbol(&library, b"linphone_account_params_enable_register\0")
            }?,
            account_params_enable_cpim_in_basic_chat_room: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_account_params_enable_cpim_in_basic_chat_room\0",
                )
            },
            account_params_set_conference_factory_address: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_account_params_set_conference_factory_address\0",
                )
            },
            account_params_set_audio_video_conference_factory_address: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_account_params_set_audio_video_conference_factory_address\0",
                )
            },
            account_params_set_file_transfer_server: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_account_params_set_file_transfer_server\0",
                )
            },
            account_params_set_lime_server_url: unsafe {
                optional_symbol(&library, b"linphone_account_params_set_lime_server_url\0")
            },
            account_cbs_new: unsafe { symbol(&library, b"linphone_account_cbs_new\0") }?,
            account_cbs_set_registration_state_changed: unsafe {
                symbol(
                    &library,
                    b"linphone_account_cbs_set_registration_state_changed\0",
                )
            }?,
            account_add_callbacks: unsafe {
                symbol(&library, b"linphone_account_add_callbacks\0")
            }?,
            account_unref: unsafe { symbol(&library, b"linphone_account_unref\0") }?,
            account_cbs_unref: unsafe { symbol(&library, b"linphone_account_cbs_unref\0") }?,
            account_params_unref: unsafe { symbol(&library, b"linphone_account_params_unref\0") }?,
            address_get_username: unsafe { symbol(&library, b"linphone_address_get_username\0") }?,
            address_get_domain: unsafe { symbol(&library, b"linphone_address_get_domain\0") }?,
            address_unref: unsafe { symbol(&library, b"linphone_address_unref\0") }?,
            auth_info_unref: unsafe { symbol(&library, b"linphone_auth_info_unref\0") }?,
            call_params_unref: unsafe { symbol(&library, b"linphone_call_params_unref\0") }?,
            call_get_remote_address: unsafe {
                symbol(&library, b"linphone_call_get_remote_address\0")
            }?,
            call_accept: unsafe { symbol(&library, b"linphone_call_accept\0") }?,
            call_decline: unsafe { symbol(&library, b"linphone_call_decline\0") }?,
            call_terminate: unsafe { symbol(&library, b"linphone_call_terminate\0") }?,
            call_set_microphone_muted: unsafe {
                symbol(&library, b"linphone_call_set_microphone_muted\0")
            }?,
            chat_room_add_callbacks: unsafe {
                symbol(&library, b"linphone_chat_room_add_callbacks\0")
            }?,
            chat_room_create_message_from_utf8: unsafe {
                symbol(&library, b"linphone_chat_room_create_message_from_utf8\0")
            }?,
            chat_room_create_voice_recording_message: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_chat_room_create_voice_recording_message\0",
                )
            },
            chat_room_cbs_set_message_received: unsafe {
                symbol(&library, b"linphone_chat_room_cbs_set_message_received\0")
            }?,
            chat_room_cbs_set_messages_received: unsafe {
                optional_symbol(&library, b"linphone_chat_room_cbs_set_messages_received\0")
            },
            chat_room_cbs_set_chat_message_received: unsafe {
                optional_symbol(
                    &library,
                    b"linphone_chat_room_cbs_set_chat_message_received\0",
                )
            },
            chat_message_cbs_new: unsafe { symbol(&library, b"linphone_chat_message_cbs_new\0") }?,
            chat_message_cbs_unref: unsafe {
                symbol(&library, b"linphone_chat_message_cbs_unref\0")
            }?,
            chat_message_cbs_set_msg_state_changed: unsafe {
                symbol(
                    &library,
                    b"linphone_chat_message_cbs_set_msg_state_changed\0",
                )
            }?,
            chat_message_add_callbacks: unsafe {
                symbol(&library, b"linphone_chat_message_add_callbacks\0")
            }?,
            chat_message_send: unsafe { symbol(&library, b"linphone_chat_message_send\0") }?,
            chat_message_get_message_id: unsafe {
                symbol(&library, b"linphone_chat_message_get_message_id\0")
            }?,
            chat_message_get_user_data: unsafe {
                symbol(&library, b"linphone_chat_message_get_user_data\0")
            }?,
            chat_message_set_user_data: unsafe {
                symbol(&library, b"linphone_chat_message_set_user_data\0")
            }?,
            chat_message_get_utf8_text: unsafe {
                optional_symbol(&library, b"linphone_chat_message_get_utf8_text\0")
            },
            chat_message_get_text: unsafe {
                optional_symbol(&library, b"linphone_chat_message_get_text\0")
            },
            chat_message_get_file_transfer_information: unsafe {
                symbol(
                    &library,
                    b"linphone_chat_message_get_file_transfer_information\0",
                )
            }?,
            chat_message_get_state: unsafe {
                symbol(&library, b"linphone_chat_message_get_state\0")
            }?,
            chat_message_state_to_string: unsafe {
                symbol(&library, b"linphone_chat_message_state_to_string\0")
            }?,
            chat_message_is_outgoing: unsafe {
                symbol(&library, b"linphone_chat_message_is_outgoing\0")
            }?,
            chat_message_is_read: unsafe { symbol(&library, b"linphone_chat_message_is_read\0") }?,
            chat_message_get_peer_address: unsafe {
                symbol(&library, b"linphone_chat_message_get_peer_address\0")
            }?,
            chat_message_get_from_address: unsafe {
                symbol(&library, b"linphone_chat_message_get_from_address\0")
            }?,
            chat_message_get_to_address: unsafe {
                symbol(&library, b"linphone_chat_message_get_to_address\0")
            }?,
            chat_message_download_content: unsafe {
                symbol(&library, b"linphone_chat_message_download_content\0")
            }?,
            content_get_type: unsafe { symbol(&library, b"linphone_content_get_type\0") }?,
            content_get_subtype: unsafe { symbol(&library, b"linphone_content_get_subtype\0") }?,
            content_get_file_path: unsafe {
                symbol(&library, b"linphone_content_get_file_path\0")
            }?,
            content_set_file_path: unsafe {
                symbol(&library, b"linphone_content_set_file_path\0")
            }?,
            core_cbs_unref: unsafe { symbol(&library, b"linphone_core_cbs_unref\0") }?,
            chat_room_cbs_unref: unsafe { symbol(&library, b"linphone_chat_room_cbs_unref\0") }?,
            event_log_get_chat_message: unsafe {
                optional_symbol(&library, b"linphone_event_log_get_chat_message\0")
            },
            im_notif_policy_enable_all: unsafe {
                optional_symbol(&library, b"linphone_im_notif_policy_enable_all\0")
            },
            nat_policy_enable_stun: unsafe {
                symbol(&library, b"linphone_nat_policy_enable_stun\0")
            }?,
            nat_policy_enable_ice: unsafe {
                symbol(&library, b"linphone_nat_policy_enable_ice\0")
            }?,
            nat_policy_set_stun_server: unsafe {
                symbol(&library, b"linphone_nat_policy_set_stun_server\0")
            }?,
            nat_policy_unref: unsafe { symbol(&library, b"linphone_nat_policy_unref\0") }?,
            recorder_params_set_file_format: unsafe {
                optional_symbol(&library, b"linphone_recorder_params_set_file_format\0")
            },
            recorder_params_unref: unsafe {
                optional_symbol(&library, b"linphone_recorder_params_unref\0")
            },
            recorder_open: unsafe { optional_symbol(&library, b"linphone_recorder_open\0") },
            recorder_start: unsafe { optional_symbol(&library, b"linphone_recorder_start\0") },
            recorder_pause: unsafe { optional_symbol(&library, b"linphone_recorder_pause\0") },
            recorder_get_duration: unsafe {
                optional_symbol(&library, b"linphone_recorder_get_duration\0")
            },
            recorder_close: unsafe { optional_symbol(&library, b"linphone_recorder_close\0") },
            recorder_unref: unsafe { optional_symbol(&library, b"linphone_recorder_unref\0") },
            core_get_version: unsafe { symbol(&library, b"linphone_core_get_version\0") }?,
            registration_state_to_string: unsafe {
                optional_symbol(&library, b"linphone_registration_state_to_string\0")
            },
            call_state_to_string: unsafe {
                optional_symbol(&library, b"linphone_call_state_to_string\0")
            },
            _library: library,
        }))
    }
}

fn load_library() -> Result<Library, String> {
    if let Some(path) =
        std::env::var_os("YOYOPOD_LIBLINPHONE_LIBRARY_PATH").filter(|value| !value.is_empty())
    {
        return unsafe { Library::new(path) }
            .map_err(|error| format!("failed to load liblinphone: {error}"));
    }

    let mut errors = Vec::new();
    for candidate in library_candidates() {
        match unsafe { Library::new(&candidate) } {
            Ok(library) => return Ok(library),
            Err(error) => errors.push(format!("{}: {error}", candidate.to_string_lossy())),
        }
    }
    Err(format!(
        "failed to load liblinphone ({})",
        errors.join("; ")
    ))
}

fn library_candidates() -> Vec<OsString> {
    if cfg!(target_os = "windows") {
        return vec!["linphone.dll".into(), "liblinphone.dll".into()];
    }
    if cfg!(target_os = "macos") {
        return vec!["liblinphone.dylib".into()];
    }
    vec![
        "liblinphone.so.12".into(),
        "liblinphone.so.11".into(),
        "liblinphone.so.10".into(),
        "liblinphone.so".into(),
    ]
}

fn symbol_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name.strip_suffix(&[0]).unwrap_or(name)).into_owned()
}
