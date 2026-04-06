#include "liblinphone_shim.h"

#include <ctype.h>
#include <pthread.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#include <linphone/api/c-account-cbs.h>
#include <linphone/api/c-account-params.h>
#include <linphone/api/c-account.h>
#include <linphone/api/c-address.h>
#include <linphone/api/c-auth-info.h>
#include <linphone/api/c-call.h>
#include <linphone/api/c-chat-message-cbs.h>
#include <linphone/api/c-chat-message.h>
#include <linphone/api/c-chat-room.h>
#include <linphone/api/c-chat-room-cbs.h>
#include <linphone/api/c-chat-room-params.h>
#include <linphone/api/c-content.h>
#include <linphone/api/c-event.h>
#include <linphone/api/c-event-log.h>
#include <linphone/api/c-factory.h>
#include <linphone/api/c-nat-policy.h>
#include <linphone/api/c-recorder.h>
#include <bctoolbox/list.h>
#include <linphone/buffer.h>
#include <linphone/core.h>
#include <linphone/error_info.h>
#include <linphone/im_notif_policy.h>
#include <linphone/misc.h>

#define YOYOPY_EVENT_QUEUE_CAPACITY 128

enum {
    YOYOPY_EVENT_NONE = 0,
    YOYOPY_EVENT_REGISTRATION = 1,
    YOYOPY_EVENT_CALL_STATE = 2,
    YOYOPY_EVENT_INCOMING_CALL = 3,
    YOYOPY_EVENT_BACKEND_STOPPED = 4,
    YOYOPY_EVENT_MESSAGE_RECEIVED = 5,
    YOYOPY_EVENT_MESSAGE_DELIVERY_CHANGED = 6,
    YOYOPY_EVENT_MESSAGE_DOWNLOAD_COMPLETED = 7,
    YOYOPY_EVENT_MESSAGE_FAILED = 8
};

enum {
    YOYOPY_REGISTRATION_NONE = 0,
    YOYOPY_REGISTRATION_PROGRESS = 1,
    YOYOPY_REGISTRATION_OK = 2,
    YOYOPY_REGISTRATION_CLEARED = 3,
    YOYOPY_REGISTRATION_FAILED = 4
};

enum {
    YOYOPY_CALL_IDLE = 0,
    YOYOPY_CALL_INCOMING = 1,
    YOYOPY_CALL_OUTGOING_INIT = 2,
    YOYOPY_CALL_OUTGOING_PROGRESS = 3,
    YOYOPY_CALL_OUTGOING_RINGING = 4,
    YOYOPY_CALL_OUTGOING_EARLY_MEDIA = 5,
    YOYOPY_CALL_CONNECTED = 6,
    YOYOPY_CALL_STREAMS_RUNNING = 7,
    YOYOPY_CALL_PAUSED = 8,
    YOYOPY_CALL_PAUSED_BY_REMOTE = 9,
    YOYOPY_CALL_UPDATED_BY_REMOTE = 10,
    YOYOPY_CALL_RELEASED = 11,
    YOYOPY_CALL_ERROR = 12,
    YOYOPY_CALL_END = 13
};

enum {
    YOYOPY_MESSAGE_KIND_TEXT = 1,
    YOYOPY_MESSAGE_KIND_VOICE_NOTE = 2
};

enum {
    YOYOPY_MESSAGE_DIRECTION_INCOMING = 1,
    YOYOPY_MESSAGE_DIRECTION_OUTGOING = 2
};

enum {
    YOYOPY_MESSAGE_DELIVERY_QUEUED = 1,
    YOYOPY_MESSAGE_DELIVERY_SENDING = 2,
    YOYOPY_MESSAGE_DELIVERY_SENT = 3,
    YOYOPY_MESSAGE_DELIVERY_DELIVERED = 4,
    YOYOPY_MESSAGE_DELIVERY_FAILED = 5
};

typedef struct {
    bool initialized;
    bool started;
    LinphoneFactory *factory;
    LinphoneCore *core;
    LinphoneAccount *account;
    LinphoneAccountCbs *account_cbs;
    LinphoneCoreCbs *core_cbs;
    LinphoneChatMessageCbs *message_cbs;
    LinphoneChatRoomCbs *chat_room_cbs;
    LinphoneCall *current_call;
    LinphoneRecorder *current_recorder;
    bool recorder_running;
    bool auto_download_incoming_voice_recordings;
    char voice_note_store_dir[512];
    char current_recording_path[512];
    LinphoneChatRoom *attached_chat_rooms[64];
    size_t attached_chat_room_count;
    pthread_mutex_t queue_lock;
    yoyopy_liblinphone_event_t queue[YOYOPY_EVENT_QUEUE_CAPACITY];
    size_t queue_head;
    size_t queue_tail;
    unsigned long long message_counter;
} yoyopy_liblinphone_state_t;

static yoyopy_liblinphone_state_t g_state = {0};
static pthread_mutex_t g_error_lock = PTHREAD_MUTEX_INITIALIZER;
static char g_last_error[512] = "";

static void yoyopy_build_chat_room_peer(
    LinphoneChatRoom *chat_room,
    char *buffer,
    size_t buffer_size
);

static void yoyopy_build_specs_string(char *buffer, size_t buffer_size);

static void yoyopy_set_error(const char *format, ...) {
    va_list args;
    pthread_mutex_lock(&g_error_lock);
    va_start(args, format);
    vsnprintf(g_last_error, sizeof(g_last_error), format, args);
    va_end(args);
    pthread_mutex_unlock(&g_error_lock);
}

static void yoyopy_clear_error(void) {
    pthread_mutex_lock(&g_error_lock);
    g_last_error[0] = '\0';
    pthread_mutex_unlock(&g_error_lock);
}

static void yoyopy_debug_log(const char *format, ...) {
    va_list args;
    va_start(args, format);
    fprintf(stderr, "YOYOPY-LIBLINPHONE: ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    fflush(stderr);
    va_end(args);
}

static void yoyopy_copy_string(char *destination, size_t destination_size, const char *source) {
    if (destination_size == 0) {
        return;
    }
    if (source == NULL) {
        destination[0] = '\0';
        return;
    }
    snprintf(destination, destination_size, "%s", source);
}

static const char *yoyopy_safe_string(const char *value) {
    return value != NULL ? value : "";
}

static int yoyopy_map_registration_state(LinphoneRegistrationState state) {
    switch (state) {
        case LinphoneRegistrationProgress:
            return YOYOPY_REGISTRATION_PROGRESS;
        case LinphoneRegistrationOk:
            return YOYOPY_REGISTRATION_OK;
        case LinphoneRegistrationCleared:
            return YOYOPY_REGISTRATION_CLEARED;
        case LinphoneRegistrationFailed:
            return YOYOPY_REGISTRATION_FAILED;
        case LinphoneRegistrationNone:
        default:
            return YOYOPY_REGISTRATION_NONE;
    }
}

static int yoyopy_map_call_state(LinphoneCallState state) {
    switch (state) {
        case LinphoneCallIncomingReceived:
        case LinphoneCallIncomingEarlyMedia:
            return YOYOPY_CALL_INCOMING;
        case LinphoneCallOutgoingInit:
            return YOYOPY_CALL_OUTGOING_INIT;
        case LinphoneCallOutgoingProgress:
            return YOYOPY_CALL_OUTGOING_PROGRESS;
        case LinphoneCallOutgoingRinging:
            return YOYOPY_CALL_OUTGOING_RINGING;
        case LinphoneCallOutgoingEarlyMedia:
            return YOYOPY_CALL_OUTGOING_EARLY_MEDIA;
        case LinphoneCallConnected:
            return YOYOPY_CALL_CONNECTED;
        case LinphoneCallStreamsRunning:
            return YOYOPY_CALL_STREAMS_RUNNING;
        case LinphoneCallPaused:
            return YOYOPY_CALL_PAUSED;
        case LinphoneCallPausedByRemote:
            return YOYOPY_CALL_PAUSED_BY_REMOTE;
        case LinphoneCallUpdatedByRemote:
        case LinphoneCallUpdating:
        case LinphoneCallEarlyUpdatedByRemote:
        case LinphoneCallEarlyUpdating:
            return YOYOPY_CALL_UPDATED_BY_REMOTE;
        case LinphoneCallReleased:
            return YOYOPY_CALL_RELEASED;
        case LinphoneCallError:
            return YOYOPY_CALL_ERROR;
        case LinphoneCallEnd:
            return YOYOPY_CALL_END;
        case LinphoneCallIdle:
        default:
            return YOYOPY_CALL_IDLE;
    }
}

static int yoyopy_map_message_delivery_state(LinphoneChatMessageState state) {
    switch (state) {
        case LinphoneChatMessageStateIdle:
            return YOYOPY_MESSAGE_DELIVERY_QUEUED;
        case LinphoneChatMessageStateInProgress:
        case LinphoneChatMessageStateFileTransferInProgress:
            return YOYOPY_MESSAGE_DELIVERY_SENDING;
        case LinphoneChatMessageStateDelivered:
        case LinphoneChatMessageStateFileTransferDone:
            return YOYOPY_MESSAGE_DELIVERY_SENT;
        case LinphoneChatMessageStateDeliveredToUser:
        case LinphoneChatMessageStateDisplayed:
            return YOYOPY_MESSAGE_DELIVERY_DELIVERED;
        case LinphoneChatMessageStateNotDelivered:
        case LinphoneChatMessageStateFileTransferError:
        default:
            return YOYOPY_MESSAGE_DELIVERY_FAILED;
    }
}

static int yoyopy_path_exists(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return 0;
    }
    return access(path, F_OK) == 0;
}

static void yoyopy_ensure_directory(const char *path) {
    char buffer[512];
    size_t length;
    size_t index;

    if (path == NULL || path[0] == '\0') {
        return;
    }

    yoyopy_copy_string(buffer, sizeof(buffer), path);
    length = strlen(buffer);
    if (length == 0) {
        return;
    }

    if (buffer[length - 1] == '/') {
        buffer[length - 1] = '\0';
    }

    for (index = 1; buffer[index] != '\0'; ++index) {
        if (buffer[index] == '/') {
            buffer[index] = '\0';
            mkdir(buffer, 0775);
            buffer[index] = '/';
        }
    }
    mkdir(buffer, 0775);
}

static void yoyopy_build_address_uri(const LinphoneAddress *address, char *buffer, size_t buffer_size) {
    const char *username;
    const char *domain;
    if (buffer_size == 0) {
        return;
    }
    buffer[0] = '\0';
    if (address == NULL) {
        return;
    }

    username = linphone_address_get_username(address);
    domain = linphone_address_get_domain(address);
    if (username != NULL && domain != NULL) {
        snprintf(buffer, buffer_size, "sip:%s@%s", username, domain);
        return;
    }
    if (domain != NULL) {
        snprintf(buffer, buffer_size, "sip:%s", domain);
    }
}

static void yoyopy_build_message_id(LinphoneChatMessage *message, char *buffer, size_t buffer_size) {
    const char *message_id;
    const char *user_data;
    struct timespec now;

    if (buffer_size == 0) {
        return;
    }
    buffer[0] = '\0';
    if (message == NULL) {
        return;
    }

    message_id = linphone_chat_message_get_message_id(message);
    if (message_id != NULL && message_id[0] != '\0') {
        yoyopy_copy_string(buffer, buffer_size, message_id);
        return;
    }

    user_data = (const char *)linphone_chat_message_get_user_data(message);
    if (user_data != NULL && user_data[0] != '\0') {
        yoyopy_copy_string(buffer, buffer_size, user_data);
        return;
    }

    clock_gettime(CLOCK_REALTIME, &now);
    g_state.message_counter += 1;
    snprintf(buffer, buffer_size, "local-%lld-%llu", (long long)now.tv_sec, g_state.message_counter);
    linphone_chat_message_set_user_data(message, strdup(buffer));
}

static void yoyopy_build_mime_type(const LinphoneContent *content, char *buffer, size_t buffer_size) {
    const char *type;
    const char *subtype;
    if (buffer_size == 0) {
        return;
    }
    buffer[0] = '\0';
    if (content == NULL) {
        return;
    }
    type = linphone_content_get_type(content);
    subtype = linphone_content_get_subtype(content);
    if (type != NULL && subtype != NULL) {
        snprintf(buffer, buffer_size, "%s/%s", type, subtype);
    } else if (type != NULL) {
        yoyopy_copy_string(buffer, buffer_size, type);
    }
}

static int yoyopy_string_contains(const char *value, const char *needle) {
    return value != NULL && needle != NULL && strstr(value, needle) != NULL;
}

static int yoyopy_extract_xml_tag_value(
    const char *xml,
    const char *open_tag,
    const char *close_tag,
    char *buffer,
    size_t buffer_size
) {
    const char *start;
    const char *end;
    size_t length;

    if (buffer_size == 0) {
        return 0;
    }
    buffer[0] = '\0';
    if (xml == NULL || open_tag == NULL || close_tag == NULL) {
        return 0;
    }

    start = strstr(xml, open_tag);
    if (start == NULL) {
        return 0;
    }
    start += strlen(open_tag);
    end = strstr(start, close_tag);
    if (end == NULL || end <= start) {
        return 0;
    }
    length = (size_t)(end - start);
    if (length >= buffer_size) {
        length = buffer_size - 1U;
    }
    memcpy(buffer, start, length);
    buffer[length] = '\0';
    return 1;
}

static int yoyopy_is_file_transfer_xml_content(const LinphoneContent *content) {
    const char *type;
    const char *subtype;

    if (content == NULL) {
        return 0;
    }
    type = linphone_content_get_type(content);
    subtype = linphone_content_get_subtype(content);
    return type != NULL && subtype != NULL
           && strcmp(type, "application") == 0
           && strcmp(subtype, "vnd.gsma.rcs-ft-http+xml") == 0;
}

static int yoyopy_is_voice_note_content(const LinphoneContent *content) {
    const char *type;
    if (content == NULL) {
        return 0;
    }
    type = linphone_content_get_type(content);
    return type != NULL && strcmp(type, "audio") == 0;
}

static int yoyopy_is_voice_note_xml_text(const char *text) {
    return yoyopy_string_contains(text, "voice-recording=yes");
}

static void yoyopy_extract_voice_note_payload_mime(
    LinphoneChatMessage *message,
    LinphoneContent *content,
    char *buffer,
    size_t buffer_size
) {
    char xml_value[256];
    const char *text;
    char *separator;

    if (buffer_size == 0) {
        return;
    }
    buffer[0] = '\0';

    if (yoyopy_is_voice_note_content(content)) {
        yoyopy_build_mime_type(content, buffer, buffer_size);
        return;
    }

    text = linphone_chat_message_get_utf8_text(message);
    if (!yoyopy_extract_xml_tag_value(
            text,
            "<content-type>",
            "</content-type>",
            xml_value,
            sizeof(xml_value)
        )) {
        return;
    }
    separator = strchr(xml_value, ';');
    if (separator != NULL) {
        *separator = '\0';
    }
    yoyopy_copy_string(buffer, buffer_size, xml_value);
}

static int yoyopy_extract_voice_note_duration_ms(LinphoneChatMessage *message) {
    char value[64];
    const char *text = linphone_chat_message_get_utf8_text(message);
    if (
        !yoyopy_extract_xml_tag_value(
            text,
            "<am:playing-length>",
            "</am:playing-length>",
            value,
            sizeof(value)
        )
    ) {
        return 0;
    }
    return atoi(value);
}

static void yoyopy_extract_voice_note_extension(
    LinphoneChatMessage *message,
    const char *mime_type,
    char *buffer,
    size_t buffer_size
) {
    char file_name[256];
    const char *text = linphone_chat_message_get_utf8_text(message);
    const char *dot;

    if (buffer_size == 0) {
        return;
    }
    buffer[0] = '\0';
    if (
        yoyopy_extract_xml_tag_value(
            text,
            "<file-name>",
            "</file-name>",
            file_name,
            sizeof(file_name)
        )
    ) {
        dot = strrchr(file_name, '.');
        if (dot != NULL && dot[1] != '\0') {
            yoyopy_copy_string(buffer, buffer_size, dot + 1);
            return;
        }
    }
    if (mime_type != NULL && strstr(mime_type, "/") != NULL) {
        const char *slash = strchr(mime_type, '/');
        if (slash != NULL && slash[1] != '\0') {
            yoyopy_copy_string(buffer, buffer_size, slash + 1);
            return;
        }
    }
    yoyopy_copy_string(buffer, buffer_size, "wav");
}

static int yoyopy_is_voice_note_message(LinphoneChatMessage *message) {
    LinphoneContent *content;
    const char *text;

    if (message == NULL) {
        return 0;
    }
    content = linphone_chat_message_get_file_transfer_information(message);
    if (yoyopy_is_voice_note_content(content)) {
        return 1;
    }
    if (!yoyopy_is_file_transfer_xml_content(content)) {
        return 0;
    }
    text = linphone_chat_message_get_utf8_text(message);
    return yoyopy_is_voice_note_xml_text(text);
}

static int yoyopy_message_kind_from_message(LinphoneChatMessage *message) {
    return yoyopy_is_voice_note_message(message) ? YOYOPY_MESSAGE_KIND_VOICE_NOTE : YOYOPY_MESSAGE_KIND_TEXT;
}

static int yoyopy_message_direction_from_message(const LinphoneChatMessage *message) {
    return linphone_chat_message_is_outgoing(message)
               ? YOYOPY_MESSAGE_DIRECTION_OUTGOING
               : YOYOPY_MESSAGE_DIRECTION_INCOMING;
}

static void yoyopy_enqueue_event(const yoyopy_liblinphone_event_t *event_value) {
    size_t next_tail;
    pthread_mutex_lock(&g_state.queue_lock);
    next_tail = (g_state.queue_tail + 1U) % YOYOPY_EVENT_QUEUE_CAPACITY;
    if (next_tail == g_state.queue_head) {
        g_state.queue_head = (g_state.queue_head + 1U) % YOYOPY_EVENT_QUEUE_CAPACITY;
    }
    g_state.queue[g_state.queue_tail] = *event_value;
    g_state.queue_tail = next_tail;
    pthread_mutex_unlock(&g_state.queue_lock);
}

static void yoyopy_queue_registration_event(LinphoneRegistrationState state, const char *reason) {
    yoyopy_liblinphone_event_t event_value;
    memset(&event_value, 0, sizeof(event_value));
    event_value.type = YOYOPY_EVENT_REGISTRATION;
    event_value.registration_state = yoyopy_map_registration_state(state);
    yoyopy_copy_string(event_value.reason, sizeof(event_value.reason), reason);
    yoyopy_enqueue_event(&event_value);
}

static void yoyopy_queue_call_state_event(LinphoneCall *call, LinphoneCallState state, const char *reason) {
    yoyopy_liblinphone_event_t event_value;
    memset(&event_value, 0, sizeof(event_value));
    event_value.type = YOYOPY_EVENT_CALL_STATE;
    event_value.call_state = yoyopy_map_call_state(state);
    if (call != NULL) {
        yoyopy_build_address_uri(
            linphone_call_get_remote_address(call),
            event_value.peer_sip_address,
            sizeof(event_value.peer_sip_address)
        );
    }
    yoyopy_copy_string(event_value.reason, sizeof(event_value.reason), reason);
    yoyopy_enqueue_event(&event_value);
}

static void yoyopy_queue_incoming_call_event(LinphoneCall *call) {
    yoyopy_liblinphone_event_t event_value;
    memset(&event_value, 0, sizeof(event_value));
    event_value.type = YOYOPY_EVENT_INCOMING_CALL;
    if (call != NULL) {
        yoyopy_build_address_uri(
            linphone_call_get_remote_address(call),
            event_value.peer_sip_address,
            sizeof(event_value.peer_sip_address)
        );
    }
    yoyopy_enqueue_event(&event_value);
}

static void yoyopy_fill_message_event_common(
    yoyopy_liblinphone_event_t *event_value,
    LinphoneChatMessage *message
) {
    LinphoneContent *content;
    char voice_note_mime[128];

    content = linphone_chat_message_get_file_transfer_information(message);
    event_value->message_kind = yoyopy_message_kind_from_message(message);
    event_value->message_direction = yoyopy_message_direction_from_message(message);
    event_value->message_delivery_state = yoyopy_map_message_delivery_state(
        linphone_chat_message_get_state(message)
    );
    yoyopy_build_message_id(message, event_value->message_id, sizeof(event_value->message_id));
    yoyopy_build_address_uri(
        linphone_chat_message_get_peer_address(message),
        event_value->peer_sip_address,
        sizeof(event_value->peer_sip_address)
    );
    yoyopy_build_address_uri(
        linphone_chat_message_get_from_address(message),
        event_value->sender_sip_address,
        sizeof(event_value->sender_sip_address)
    );
    yoyopy_build_address_uri(
        linphone_chat_message_get_to_address(message),
        event_value->recipient_sip_address,
        sizeof(event_value->recipient_sip_address)
    );
    yoyopy_copy_string(
        event_value->text,
        sizeof(event_value->text),
        linphone_chat_message_get_utf8_text(message)
    );
    voice_note_mime[0] = '\0';
    if (event_value->message_kind == YOYOPY_MESSAGE_KIND_VOICE_NOTE) {
        yoyopy_extract_voice_note_payload_mime(
            message,
            content,
            voice_note_mime,
            sizeof(voice_note_mime)
        );
        yoyopy_copy_string(
            event_value->mime_type,
            sizeof(event_value->mime_type),
            voice_note_mime[0] != '\0' ? voice_note_mime : "audio/wav"
        );
        event_value->duration_ms = yoyopy_extract_voice_note_duration_ms(message);
        if (yoyopy_is_file_transfer_xml_content(content)) {
            event_value->text[0] = '\0';
        }
    } else {
        yoyopy_build_mime_type(content, event_value->mime_type, sizeof(event_value->mime_type));
    }
    if (content != NULL) {
        yoyopy_copy_string(
            event_value->local_file_path,
            sizeof(event_value->local_file_path),
            linphone_content_get_file_path(content)
        );
    }
}

static void yoyopy_attach_message_callbacks(LinphoneChatMessage *message) {
    if (message == NULL || g_state.message_cbs == NULL) {
        return;
    }
    linphone_chat_message_add_callbacks(message, g_state.message_cbs);
}

static void yoyopy_generate_voice_note_path(
    const char *message_id,
    const char *extension,
    char *buffer,
    size_t buffer_size
) {
    const char *selected_extension = extension;
    if (selected_extension == NULL || selected_extension[0] == '\0') {
        selected_extension = "wav";
    }
    snprintf(
        buffer,
        buffer_size,
        "%s/%s.%s",
        g_state.voice_note_store_dir,
        message_id,
        selected_extension
    );
}

static size_t yoyopy_list_size(const bctbx_list_t *list) {
    size_t count = 0U;
    const bctbx_list_t *item = list;
    while (item != NULL) {
        count += 1U;
        item = item->next;
    }
    return count;
}

static void yoyopy_log_room_snapshot(LinphoneChatRoom *chat_room, const char *phase) {
    const LinphoneChatRoomParams *room_params = NULL;
    char peer[256];
    peer[0] = '\0';
    yoyopy_build_chat_room_peer(chat_room, peer, sizeof(peer));
    if (chat_room != NULL) {
        room_params = linphone_chat_room_get_current_params(chat_room);
    }
    yoyopy_debug_log(
        "diag[%s] room peer=%s state=%d caps=%d backend=%d encryption_backend=%d group=%d encrypted=%d read_only=%d participants=%d",
        phase,
        peer,
        chat_room != NULL ? (int)linphone_chat_room_get_state(chat_room) : -1,
        chat_room != NULL ? (int)linphone_chat_room_get_capabilities(chat_room) : -1,
        room_params != NULL ? (int)linphone_chat_room_params_get_backend(room_params) : -1,
        room_params != NULL ? (int)linphone_chat_room_params_get_encryption_backend(room_params) : -1,
        room_params != NULL && linphone_chat_room_params_group_enabled(room_params) ? 1 : 0,
        room_params != NULL && linphone_chat_room_params_encryption_enabled(room_params) ? 1 : 0,
        (chat_room != NULL && linphone_chat_room_is_read_only(chat_room)) ? 1 : 0,
        chat_room != NULL ? linphone_chat_room_get_nb_participants(chat_room) : -1
    );
}

static void yoyopy_log_account_diagnostics(const char *phase) {
    const LinphoneAccountParams *params;
    const bctbx_list_t *core_rooms;
    bctbx_list_t *account_rooms;
    LinphoneChatRoomParams *default_params;
    const LinphoneAddress *conference_factory_address = NULL;
    const char *file_transfer_server = "";
    const char *lime_server = "";
    char core_specs[256];
    char conference_factory_uri[256];
    size_t core_room_count;
    size_t account_room_count;
    const bctbx_list_t *item;

    if (g_state.core == NULL || g_state.account == NULL) {
        return;
    }

    params = linphone_account_get_params(g_state.account);
    conference_factory_uri[0] = '\0';
    if (params != NULL) {
        conference_factory_address = linphone_account_params_get_conference_factory_address(params);
        yoyopy_build_address_uri(
            conference_factory_address,
            conference_factory_uri,
            sizeof(conference_factory_uri)
        );
        file_transfer_server = yoyopy_safe_string(
            linphone_account_params_get_file_transfer_server(params)
        );
        lime_server = yoyopy_safe_string(
            linphone_account_params_get_lime_server_url(params)
        );
    }

    core_rooms = linphone_core_get_chat_rooms(g_state.core);
    account_rooms = linphone_account_get_chat_rooms(g_state.account);
    core_room_count = yoyopy_list_size(core_rooms);
    account_room_count = yoyopy_list_size(account_rooms);

    yoyopy_debug_log(
        "diag[%s] account unread=%d core_rooms=%llu account_rooms=%llu cpim_basic=%d conference_factory=%s file_transfer=%s lime=%s",
        phase,
        linphone_account_get_unread_chat_message_count(g_state.account),
        (unsigned long long)core_room_count,
        (unsigned long long)account_room_count,
        params != NULL && linphone_account_params_cpim_in_basic_chat_room_enabled(params) ? 1 : 0,
        conference_factory_uri,
        file_transfer_server,
        lime_server
    );
    yoyopy_build_specs_string(core_specs, sizeof(core_specs));
    yoyopy_debug_log("diag[%s] core_specs=%s", phase, core_specs);

    default_params = linphone_core_create_default_chat_room_params(g_state.core);
    if (default_params != NULL) {
        yoyopy_debug_log(
            "diag[%s] default_chat_room backend=%d encryption_backend=%d group=%d encrypted=%d rtt=%d ephemeral_mode=%d ephemeral_lifetime=%ld",
            phase,
            (int)linphone_chat_room_params_get_backend(default_params),
            (int)linphone_chat_room_params_get_encryption_backend(default_params),
            linphone_chat_room_params_group_enabled(default_params) ? 1 : 0,
            linphone_chat_room_params_encryption_enabled(default_params) ? 1 : 0,
            linphone_chat_room_params_rtt_enabled(default_params) ? 1 : 0,
            (int)linphone_chat_room_params_get_ephemeral_mode(default_params),
            linphone_chat_room_params_get_ephemeral_lifetime(default_params)
        );
        linphone_chat_room_params_unref(default_params);
    }

    item = core_rooms;
    while (item != NULL) {
        LinphoneChatRoom *chat_room = (LinphoneChatRoom *)bctbx_list_get_data(item);
        yoyopy_log_room_snapshot(chat_room, phase);
        item = item->next;
    }

    if (account_rooms != NULL) {
        bctbx_list_free(account_rooms);
    }
}

static void yoyopy_build_specs_string(char *buffer, size_t buffer_size) {
    bctbx_list_t *specs;
    bctbx_list_t *item;
    size_t used = 0;

    if (buffer_size == 0) {
        return;
    }
    buffer[0] = '\0';
    if (g_state.core == NULL) {
        return;
    }

    specs = linphone_core_get_linphone_specs_list(g_state.core);
    item = specs;
    while (item != NULL) {
        const char *spec = (const char *)bctbx_list_get_data(item);
        int written;
        if (spec == NULL) {
            item = item->next;
            continue;
        }
        written = snprintf(
            buffer + used,
            buffer_size - used,
            "%s%s",
            used == 0 ? "" : ",",
            spec
        );
        if (written < 0 || (size_t)written >= (buffer_size - used)) {
            break;
        }
        used += (size_t)written;
        item = item->next;
    }
    if (specs != NULL) {
        bctbx_list_free(specs);
    }
}

static void yoyopy_ensure_core_spec(const char *spec) {
    bctbx_list_t *specs;
    bctbx_list_t *item;

    if (g_state.core == NULL || spec == NULL || spec[0] == '\0') {
        return;
    }

    specs = linphone_core_get_linphone_specs_list(g_state.core);
    item = specs;
    while (item != NULL) {
        const char *current = (const char *)bctbx_list_get_data(item);
        if (current != NULL && strcmp(current, spec) == 0) {
            if (specs != NULL) {
                bctbx_list_free(specs);
            }
            return;
        }
        item = item->next;
    }
    if (specs != NULL) {
        bctbx_list_free(specs);
    }
    linphone_core_add_linphone_spec(g_state.core, spec);
}

static void yoyopy_prepare_auto_download(LinphoneChatMessage *message) {
    LinphoneContent *content;
    char message_id[128];
    char mime_type[128];
    char extension[32];
    char target_path[512];

    if (!g_state.auto_download_incoming_voice_recordings || message == NULL) {
        return;
    }

    content = linphone_chat_message_get_file_transfer_information(message);
    if (!yoyopy_is_voice_note_message(message) || content == NULL) {
        return;
    }

    yoyopy_build_message_id(message, message_id, sizeof(message_id));
    yoyopy_extract_voice_note_payload_mime(message, content, mime_type, sizeof(mime_type));
    yoyopy_extract_voice_note_extension(message, mime_type, extension, sizeof(extension));
    yoyopy_generate_voice_note_path(message_id, extension, target_path, sizeof(target_path));
    yoyopy_ensure_directory(g_state.voice_note_store_dir);
    linphone_content_set_file_path(content, target_path);
    yoyopy_debug_log(
        "auto-download voice note message_id=%s mime_type=%s target=%s",
        message_id,
        mime_type,
        target_path
    );
    linphone_chat_message_download_content(message, content);
}

static void yoyopy_build_chat_room_peer(
    LinphoneChatRoom *chat_room,
    char *buffer,
    size_t buffer_size
) {
    if (buffer_size == 0) {
        return;
    }
    buffer[0] = '\0';
    if (chat_room == NULL) {
        return;
    }
    yoyopy_build_address_uri(linphone_chat_room_get_peer_address(chat_room), buffer, buffer_size);
}

static int yoyopy_chat_room_already_attached(LinphoneChatRoom *chat_room) {
    size_t index;
    if (chat_room == NULL) {
        return 1;
    }
    for (index = 0; index < g_state.attached_chat_room_count; ++index) {
        if (g_state.attached_chat_rooms[index] == chat_room) {
            return 1;
        }
    }
    return 0;
}

static int yoyopy_apply_transports(LinphoneCore *core, const char *transport) {
    LinphoneTransports *transports = NULL;
    const char *selected = transport;
    LinphoneStatus status;

    if (core == NULL) {
        yoyopy_set_error("Cannot configure Liblinphone transports without a core");
        return -1;
    }

    if (selected == NULL || selected[0] == '\0' || strcmp(selected, "auto") == 0) {
        selected = "tcp";
    }

    transports = linphone_core_get_transports(core);
    if (transports == NULL) {
        yoyopy_set_error("Failed to allocate Linphone transports");
        return -1;
    }

    linphone_transports_set_udp_port(transports, 0);
    linphone_transports_set_tcp_port(transports, 0);
    linphone_transports_set_tls_port(transports, 0);
    linphone_transports_set_dtls_port(transports, 0);

    if (strcmp(selected, "udp") == 0) {
        linphone_transports_set_udp_port(transports, LC_SIP_TRANSPORT_RANDOM);
    } else if (strcmp(selected, "tls") == 0) {
        linphone_transports_set_tls_port(transports, LC_SIP_TRANSPORT_RANDOM);
    } else if (strcmp(selected, "dtls") == 0) {
        linphone_transports_set_dtls_port(transports, LC_SIP_TRANSPORT_RANDOM);
    } else {
        linphone_transports_set_tcp_port(transports, LC_SIP_TRANSPORT_RANDOM);
    }

    status = linphone_core_set_transports(core, transports);
    linphone_transports_unref(transports);
    if (status != 0) {
        yoyopy_set_error("Failed to configure Liblinphone transports for %s", selected);
        return -1;
    }
    return 0;
}

static int yoyopy_configure_media_policy(LinphoneCore *core, LinphoneFactory *factory) {
    LinphoneVideoActivationPolicy *policy = NULL;

    if (core == NULL || factory == NULL) {
        yoyopy_set_error("Cannot configure Liblinphone media policy without a core and factory");
        return -1;
    }

    linphone_core_enable_video_capture(core, FALSE);
    linphone_core_enable_video_display(core, FALSE);

    policy = linphone_factory_create_video_activation_policy(factory);
    if (policy == NULL) {
        yoyopy_set_error("Failed to create Liblinphone video activation policy");
        return -1;
    }

    linphone_video_activation_policy_set_automatically_accept(policy, FALSE);
    linphone_video_activation_policy_set_automatically_initiate(policy, FALSE);
    linphone_core_set_video_activation_policy(core, policy);
    linphone_video_activation_policy_unref(policy);
    return 0;
}

static int yoyopy_configure_network_media_defaults(LinphoneCore *core, const char *stun_server) {
    LinphoneNatPolicy *nat_policy = NULL;

    if (core == NULL) {
        yoyopy_set_error("Cannot configure Liblinphone network defaults without a core");
        return -1;
    }

    linphone_core_set_media_encryption(core, LinphoneMediaEncryptionSRTP);
    linphone_core_set_media_encryption_mandatory(core, FALSE);
    linphone_core_set_audio_port_range(core, 7076, 7100);
    linphone_core_set_video_port_range(core, 9076, 9100);

    nat_policy = linphone_core_create_nat_policy(core);
    if (nat_policy == NULL) {
        yoyopy_set_error("Failed to create Liblinphone NAT policy");
        return -1;
    }

    linphone_nat_policy_enable_stun(nat_policy, TRUE);
    linphone_nat_policy_enable_ice(nat_policy, TRUE);
    if (stun_server != NULL && stun_server[0] != '\0') {
        linphone_nat_policy_set_stun_server(nat_policy, stun_server);
    }
    linphone_core_set_nat_policy(core, nat_policy);
    linphone_nat_policy_unref(nat_policy);
    return 0;
}

static void yoyopy_queue_message_received_event(LinphoneChatMessage *message) {
    yoyopy_liblinphone_event_t event_value;
    memset(&event_value, 0, sizeof(event_value));
    event_value.type = YOYOPY_EVENT_MESSAGE_RECEIVED;
    event_value.unread = linphone_chat_message_is_read(message) ? 0 : 1;
    yoyopy_fill_message_event_common(&event_value, message);
    yoyopy_enqueue_event(&event_value);
}

static void yoyopy_queue_message_delivery_event(
    LinphoneChatMessage *message,
    LinphoneChatMessageState state
) {
    yoyopy_liblinphone_event_t event_value;
    memset(&event_value, 0, sizeof(event_value));
    event_value.type = YOYOPY_EVENT_MESSAGE_DELIVERY_CHANGED;
    yoyopy_fill_message_event_common(&event_value, message);
    event_value.message_delivery_state = yoyopy_map_message_delivery_state(state);
    if (state == LinphoneChatMessageStateNotDelivered || state == LinphoneChatMessageStateFileTransferError) {
        const char *state_text = linphone_chat_message_state_to_string(state);
        yoyopy_copy_string(event_value.reason, sizeof(event_value.reason), state_text);
    }
    yoyopy_enqueue_event(&event_value);
}

static void yoyopy_queue_download_completed_event(LinphoneChatMessage *message) {
    LinphoneContent *content;
    yoyopy_liblinphone_event_t event_value;
    memset(&event_value, 0, sizeof(event_value));
    content = linphone_chat_message_get_file_transfer_information(message);
    if (content == NULL) {
        return;
    }
    event_value.type = YOYOPY_EVENT_MESSAGE_DOWNLOAD_COMPLETED;
    yoyopy_fill_message_event_common(&event_value, message);
    yoyopy_copy_string(
        event_value.local_file_path,
        sizeof(event_value.local_file_path),
        linphone_content_get_file_path(content)
    );
    yoyopy_enqueue_event(&event_value);
}

static void yoyopy_on_registration_state_changed(
    LinphoneAccount *account,
    LinphoneRegistrationState state,
    const char *message
) {
    (void)account;
    yoyopy_queue_registration_event(state, message);
    if (state == LinphoneRegistrationOk) {
        yoyopy_log_account_diagnostics("registration_ok");
    }
}

static void yoyopy_on_call_state_changed(
    LinphoneCore *core,
    LinphoneCall *call,
    LinphoneCallState state,
    const char *message
) {
    (void)core;
    g_state.current_call = call;
    yoyopy_queue_call_state_event(call, state, message);
    if (state == LinphoneCallIncomingReceived) {
        yoyopy_queue_incoming_call_event(call);
    }
    if (state == LinphoneCallReleased || state == LinphoneCallEnd || state == LinphoneCallError) {
        g_state.current_call = NULL;
    }
}

static void yoyopy_on_message_received(
    LinphoneCore *core,
    LinphoneChatRoom *chat_room,
    LinphoneChatMessage *message
) {
    char peer[256];
    (void)core;
    (void)chat_room;
    peer[0] = '\0';
    if (message != NULL) {
        yoyopy_build_address_uri(
            linphone_chat_message_get_peer_address(message),
            peer,
            sizeof(peer)
        );
    }
    yoyopy_debug_log(
        "message_received peer=%s kind=%d delivery=%d",
        peer,
        yoyopy_message_kind_from_message(message),
        yoyopy_map_message_delivery_state(linphone_chat_message_get_state(message))
    );
    yoyopy_attach_message_callbacks(message);
    yoyopy_queue_message_received_event(message);
    yoyopy_prepare_auto_download(message);
}

static void yoyopy_on_chat_room_message_received(
    LinphoneChatRoom *chat_room,
    LinphoneChatMessage *message
) {
    char peer[256];
    peer[0] = '\0';
    yoyopy_build_chat_room_peer(chat_room, peer, sizeof(peer));
    yoyopy_debug_log(
        "chat_room.message_received peer=%s kind=%d delivery=%d",
        peer,
        yoyopy_message_kind_from_message(message),
        yoyopy_map_message_delivery_state(linphone_chat_message_get_state(message))
    );
    yoyopy_attach_message_callbacks(message);
    yoyopy_queue_message_received_event(message);
    yoyopy_prepare_auto_download(message);
}

static void yoyopy_on_chat_room_messages_received(
    LinphoneChatRoom *chat_room,
    const bctbx_list_t *messages
) {
    const bctbx_list_t *item = messages;
    char peer[256];
    peer[0] = '\0';
    yoyopy_build_chat_room_peer(chat_room, peer, sizeof(peer));
    yoyopy_debug_log("chat_room.messages_received peer=%s", peer);
    while (item != NULL) {
        LinphoneChatMessage *message = (LinphoneChatMessage *)bctbx_list_get_data(item);
        if (message != NULL) {
            yoyopy_on_chat_room_message_received(chat_room, message);
        }
        item = item->next;
    }
}

static void yoyopy_on_chat_room_chat_message_received(
    LinphoneChatRoom *chat_room,
    const LinphoneEventLog *event_log
) {
    LinphoneChatMessage *message = linphone_event_log_get_chat_message(event_log);
    char peer[256];
    peer[0] = '\0';
    yoyopy_build_chat_room_peer(chat_room, peer, sizeof(peer));
    yoyopy_debug_log("chat_room.chat_message_received peer=%s", peer);
    if (message != NULL) {
        yoyopy_on_chat_room_message_received(chat_room, message);
    }
}

static void yoyopy_on_chat_room_undecryptable_message_received(
    LinphoneChatRoom *chat_room,
    LinphoneChatMessage *message
) {
    char peer[256];
    peer[0] = '\0';
    yoyopy_build_chat_room_peer(chat_room, peer, sizeof(peer));
    yoyopy_debug_log(
        "chat_room.undecryptable_message_received peer=%s kind=%d",
        peer,
        message != NULL ? yoyopy_message_kind_from_message(message) : 0
    );
}

static void yoyopy_attach_chat_room_callbacks(LinphoneChatRoom *chat_room) {
    char peer[256];
    if (chat_room == NULL || g_state.chat_room_cbs == NULL || yoyopy_chat_room_already_attached(chat_room)) {
        return;
    }
    if (g_state.attached_chat_room_count >= (sizeof(g_state.attached_chat_rooms) / sizeof(g_state.attached_chat_rooms[0]))) {
        yoyopy_debug_log("chat_room callback registry is full; skipping room attachment");
        return;
    }
    linphone_chat_room_add_callbacks(chat_room, g_state.chat_room_cbs);
    g_state.attached_chat_rooms[g_state.attached_chat_room_count++] = chat_room;
    peer[0] = '\0';
    yoyopy_build_chat_room_peer(chat_room, peer, sizeof(peer));
    yoyopy_debug_log(
        "attached chat_room callbacks peer=%s state=%d read_only=%d",
        peer,
        (int)linphone_chat_room_get_state(chat_room),
        linphone_chat_room_is_read_only(chat_room) ? 1 : 0
    );
}

static void yoyopy_attach_all_chat_room_callbacks(void) {
    const bctbx_list_t *rooms;
    const bctbx_list_t *item;
    if (g_state.core == NULL) {
        return;
    }
    rooms = linphone_core_get_chat_rooms(g_state.core);
    item = rooms;
    while (item != NULL) {
        LinphoneChatRoom *chat_room = (LinphoneChatRoom *)bctbx_list_get_data(item);
        yoyopy_attach_chat_room_callbacks(chat_room);
        item = item->next;
    }
}

static LinphoneChatRoomParams *yoyopy_create_preferred_chat_room_params(void) {
    LinphoneChatRoomParams *params;
    const LinphoneAccountParams *account_params;
    const char *file_transfer_server = "";
    const char *lime_server = "";

    if (g_state.core == NULL) {
        return NULL;
    }

    params = linphone_core_create_default_chat_room_params(g_state.core);
    if (params == NULL) {
        return NULL;
    }

    linphone_chat_room_params_enable_group(params, FALSE);
    linphone_chat_room_params_enable_rtt(params, FALSE);

    account_params = g_state.account != NULL ? linphone_account_get_params(g_state.account) : NULL;
    if (account_params != NULL) {
        file_transfer_server = yoyopy_safe_string(
            linphone_account_params_get_file_transfer_server(account_params)
        );
        lime_server = yoyopy_safe_string(
            linphone_account_params_get_lime_server_url(account_params)
        );
    }

    if (file_transfer_server[0] != '\0' || lime_server[0] != '\0') {
        linphone_chat_room_params_set_backend(params, LinphoneChatRoomBackendFlexisipChat);
        linphone_chat_room_params_set_subject(params, "YoyoPod");
    }
    if (lime_server[0] != '\0') {
        linphone_chat_room_params_set_encryption_backend(params, LinphoneChatRoomEncryptionBackendLime);
        linphone_chat_room_params_enable_encryption(params, TRUE);
    }

    return params;
}

static LinphoneChatRoomParams *yoyopy_create_direct_chat_room_params(void) {
    LinphoneChatRoomParams *params;

    if (g_state.core == NULL) {
        return NULL;
    }

    params = linphone_core_create_default_chat_room_params(g_state.core);
    if (params == NULL) {
        return NULL;
    }

    linphone_chat_room_params_set_backend(params, LinphoneChatRoomBackendBasic);
    linphone_chat_room_params_enable_group(params, FALSE);
    linphone_chat_room_params_enable_rtt(params, FALSE);
    linphone_chat_room_params_enable_encryption(params, FALSE);
    return params;
}

static int yoyopy_should_prefer_hosted_chat_rooms(void) {
    const LinphoneAccountParams *account_params;
    const char *file_transfer_server = "";
    const char *lime_server = "";
    const LinphoneAddress *conference_factory_address = NULL;

    if (g_state.account == NULL) {
        return 0;
    }

    account_params = linphone_account_get_params(g_state.account);
    if (account_params == NULL) {
        return 0;
    }

    conference_factory_address = linphone_account_params_get_conference_factory_address(account_params);
    file_transfer_server = yoyopy_safe_string(
        linphone_account_params_get_file_transfer_server(account_params)
    );
    lime_server = yoyopy_safe_string(
        linphone_account_params_get_lime_server_url(account_params)
    );

    return conference_factory_address != NULL || file_transfer_server[0] != '\0' || lime_server[0] != '\0';
}

static void yoyopy_prune_stale_basic_chat_rooms(void) {
    const bctbx_list_t *rooms;
    const bctbx_list_t *item;
    int removed_any = 0;

    if (
        g_state.core == NULL ||
        !yoyopy_should_prefer_hosted_chat_rooms() ||
        g_state.auto_download_incoming_voice_recordings
    ) {
        return;
    }

    rooms = linphone_core_get_chat_rooms(g_state.core);
    item = rooms;
    while (item != NULL) {
        const bctbx_list_t *next = item->next;
        LinphoneChatRoom *chat_room = (LinphoneChatRoom *)bctbx_list_get_data(item);
        const LinphoneChatRoomParams *room_params = NULL;

        if (chat_room != NULL) {
            room_params = linphone_chat_room_get_current_params(chat_room);
            if (
                room_params != NULL &&
                linphone_chat_room_params_get_backend(room_params) == LinphoneChatRoomBackendBasic
            ) {
                char peer[256];
                peer[0] = '\0';
                yoyopy_build_chat_room_peer(chat_room, peer, sizeof(peer));
                yoyopy_debug_log("pruning stale basic chat room peer=%s", peer);
                linphone_core_delete_chat_room(g_state.core, chat_room);
                removed_any = 1;
            }
        }

        item = next;
    }

    if (removed_any) {
        g_state.attached_chat_room_count = 0U;
    }
}

static void yoyopy_on_messages_received(
    LinphoneCore *core,
    LinphoneChatRoom *chat_room,
    const bctbx_list_t *messages
) {
    const bctbx_list_t *item = messages;
    (void)core;
    (void)chat_room;
    yoyopy_debug_log("messages_received aggregated callback triggered");
    while (item != NULL) {
        LinphoneChatMessage *message = (LinphoneChatMessage *)bctbx_list_get_data(item);
        if (message != NULL) {
            yoyopy_on_message_received(core, chat_room, message);
        }
        item = item->next;
    }
}

static void yoyopy_on_message_received_unable_decrypt(
    LinphoneCore *core,
    LinphoneChatRoom *chat_room,
    LinphoneChatMessage *message
) {
    char peer[256];
    (void)core;
    (void)chat_room;
    peer[0] = '\0';
    if (message != NULL) {
        yoyopy_build_address_uri(
            linphone_chat_message_get_peer_address(message),
            peer,
            sizeof(peer)
        );
    }
    yoyopy_debug_log(
        "message_received_unable_decrypt peer=%s kind=%d",
        peer,
        message != NULL ? yoyopy_message_kind_from_message(message) : 0
    );
}

static void yoyopy_on_subscription_state_changed(
    LinphoneCore *core,
    LinphoneEvent *linphone_event,
    LinphoneSubscriptionState state
) {
    const LinphoneErrorInfo *error_info;
    char from[256];
    char to[256];
    char resource[256];

    (void)core;
    if (linphone_event == NULL) {
        return;
    }

    error_info = linphone_event_get_error_info(linphone_event);
    yoyopy_build_address_uri(linphone_event_get_from(linphone_event), from, sizeof(from));
    yoyopy_build_address_uri(linphone_event_get_to(linphone_event), to, sizeof(to));
    yoyopy_build_address_uri(linphone_event_get_resource(linphone_event), resource, sizeof(resource));
    yoyopy_debug_log(
        "subscription_state_changed name=%s state=%s reason=%s protocol_code=%d phrase=%s from=%s to=%s resource=%s",
        yoyopy_safe_string(linphone_event_get_name(linphone_event)),
        yoyopy_safe_string(linphone_subscription_state_to_string(state)),
        yoyopy_safe_string(linphone_reason_to_string(linphone_event_get_reason(linphone_event))),
        error_info != NULL ? linphone_error_info_get_protocol_code(error_info) : -1,
        error_info != NULL ? yoyopy_safe_string(linphone_error_info_get_phrase(error_info)) : "",
        from,
        to,
        resource
    );
}

static void yoyopy_on_message_state_changed(
    LinphoneChatMessage *message,
    LinphoneChatMessageState state
) {
    yoyopy_queue_message_delivery_event(message, state);
    if (state == LinphoneChatMessageStateFileTransferDone) {
        yoyopy_queue_download_completed_event(message);
    }
}

static int yoyopy_configure_account(
    const char *sip_server,
    const char *sip_username,
    const char *sip_password,
    const char *sip_password_ha1,
    const char *sip_identity,
    const char *transport,
    const char *conference_factory_uri,
    const char *file_transfer_server_url,
    const char *lime_server_url
) {
    LinphoneAddress *server_address = NULL;
    LinphoneAddress *identity_address = NULL;
    LinphoneAddress *conference_factory_address = NULL;
    LinphoneAccountParams *params = NULL;
    LinphoneAccount *account = NULL;
    LinphoneAuthInfo *auth_info = NULL;
    char server_uri[256];

    snprintf(
        server_uri,
        sizeof(server_uri),
        "sip:%s;transport=%s",
        yoyopy_safe_string(sip_server),
        transport != NULL && transport[0] != '\0' ? transport : "tcp"
    );

    params = linphone_core_create_account_params(g_state.core);
    if (params == NULL) {
        yoyopy_set_error("Failed to create Linphone account params");
        return -1;
    }

    server_address = linphone_factory_create_address(g_state.factory, server_uri);
    identity_address = linphone_factory_create_address(g_state.factory, sip_identity);
    if (server_address == NULL || identity_address == NULL) {
        yoyopy_set_error("Failed to create Linphone account addresses");
        goto fail;
    }

    if (linphone_account_params_set_server_address(params, server_address) != 0) {
        yoyopy_set_error("Failed to set Linphone server address");
        goto fail;
    }
    if (linphone_account_params_set_identity_address(params, identity_address) != 0) {
        yoyopy_set_error("Failed to set Linphone identity address");
        goto fail;
    }
    linphone_account_params_enable_register(params, TRUE);
    linphone_account_params_enable_cpim_in_basic_chat_room(params, TRUE);
    if (conference_factory_uri != NULL && conference_factory_uri[0] != '\0') {
        conference_factory_address = linphone_factory_create_address(
            g_state.factory,
            conference_factory_uri
        );
        if (conference_factory_address == NULL) {
            yoyopy_set_error("Failed to create conference factory address");
            goto fail;
        }
        linphone_account_params_set_conference_factory_address(params, conference_factory_address);
        linphone_account_params_set_audio_video_conference_factory_address(
            params,
            conference_factory_address
        );
    }
    if (file_transfer_server_url != NULL && file_transfer_server_url[0] != '\0') {
        linphone_account_params_set_file_transfer_server(params, file_transfer_server_url);
        linphone_core_set_file_transfer_server(g_state.core, file_transfer_server_url);
    }
    if (lime_server_url != NULL && lime_server_url[0] != '\0') {
        linphone_core_enable_lime_x3dh(g_state.core, TRUE);
        linphone_account_params_set_lime_server_url(params, lime_server_url);
    } else {
        linphone_core_enable_lime_x3dh(g_state.core, FALSE);
    }

    account = linphone_core_create_account(g_state.core, params);
    if (account == NULL) {
        yoyopy_set_error("Failed to create Linphone account");
        goto fail;
    }

    g_state.account_cbs = linphone_account_cbs_new();
    if (g_state.account_cbs == NULL) {
        yoyopy_set_error("Failed to create Linphone account callbacks");
        goto fail;
    }
    linphone_account_cbs_set_registration_state_changed(
        g_state.account_cbs,
        yoyopy_on_registration_state_changed
    );
    linphone_account_add_callbacks(account, g_state.account_cbs);

    auth_info = linphone_factory_create_auth_info_2(
        g_state.factory,
        sip_username,
        sip_username,
        (sip_password != NULL && sip_password[0] != '\0') ? sip_password : NULL,
        (sip_password_ha1 != NULL && sip_password_ha1[0] != '\0') ? sip_password_ha1 : NULL,
        sip_server,
        sip_server,
        "SHA-256"
    );
    if (auth_info != NULL) {
        linphone_core_add_auth_info(g_state.core, auth_info);
        linphone_auth_info_unref(auth_info);
    }

    if (linphone_core_add_account(g_state.core, account) != 0) {
        yoyopy_set_error("Failed to add Linphone account to core");
        goto fail;
    }
    linphone_core_set_default_account(g_state.core, account);
    g_state.account = account;

    linphone_address_unref(server_address);
    linphone_address_unref(identity_address);
    if (conference_factory_address != NULL) {
        linphone_address_unref(conference_factory_address);
    }
    linphone_account_params_unref(params);
    return 0;

fail:
    if (account != NULL) {
        linphone_account_unref(account);
    }
    if (conference_factory_address != NULL) {
        linphone_address_unref(conference_factory_address);
    }
    if (identity_address != NULL) {
        linphone_address_unref(identity_address);
    }
    if (server_address != NULL) {
        linphone_address_unref(server_address);
    }
    if (params != NULL) {
        linphone_account_params_unref(params);
    }
    return -1;
}

static void yoyopy_cleanup_recorder(void) {
    if (g_state.current_recorder != NULL) {
        if (g_state.recorder_running) {
            linphone_recorder_pause(g_state.current_recorder);
        }
        linphone_recorder_close(g_state.current_recorder);
        linphone_recorder_unref(g_state.current_recorder);
        g_state.current_recorder = NULL;
    }
    g_state.recorder_running = false;
    g_state.current_recording_path[0] = '\0';
}

int yoyopy_liblinphone_init(void) {
    if (g_state.initialized) {
        return 0;
    }
    memset(&g_state, 0, sizeof(g_state));
    if (pthread_mutex_init(&g_state.queue_lock, NULL) != 0) {
        yoyopy_set_error("Failed to initialize Liblinphone event queue mutex");
        return -1;
    }
    g_state.factory = linphone_factory_get();
    if (g_state.factory == NULL) {
        yoyopy_set_error("Failed to get Liblinphone factory");
        pthread_mutex_destroy(&g_state.queue_lock);
        return -1;
    }
    g_state.initialized = true;
    yoyopy_clear_error();
    return 0;
}

void yoyopy_liblinphone_shutdown(void) {
    yoyopy_liblinphone_stop();
    if (g_state.initialized) {
        pthread_mutex_destroy(&g_state.queue_lock);
        memset(&g_state, 0, sizeof(g_state));
    }
}

int yoyopy_liblinphone_start(
    const char *sip_server,
    const char *sip_username,
    const char *sip_password,
    const char *sip_password_ha1,
    const char *sip_identity,
    const char *factory_config_path,
    const char *transport,
    const char *stun_server,
    const char *conference_factory_uri,
    const char *file_transfer_server_url,
    const char *lime_server_url,
    int32_t auto_download_incoming_voice_recordings,
    const char *playback_device_id,
    const char *ringer_device_id,
    const char *capture_device_id,
    const char *media_device_id,
    int32_t echo_cancellation,
    int32_t mic_gain,
    int32_t speaker_volume,
    const char *voice_note_store_dir
) {
    if (!g_state.initialized && yoyopy_liblinphone_init() != 0) {
        return -1;
    }

    if (g_state.started) {
        return 0;
    }

    if (sip_server == NULL || sip_server[0] == '\0' || sip_identity == NULL || sip_identity[0] == '\0') {
        yoyopy_set_error("Missing SIP identity or SIP server for Liblinphone startup");
        return -1;
    }

    linphone_logging_service_set_log_level_mask(
        linphone_logging_service_get(),
        LinphoneLogLevelDebug
            | LinphoneLogLevelTrace
            | LinphoneLogLevelMessage
            | LinphoneLogLevelWarning
            | LinphoneLogLevelError
            | LinphoneLogLevelFatal
    );

    g_state.core = linphone_factory_create_core_3(
        g_state.factory,
        NULL,
        (factory_config_path != NULL && factory_config_path[0] != '\0') ? factory_config_path : NULL,
        NULL
    );
    if (g_state.core == NULL) {
        yoyopy_set_error("Failed to create Liblinphone core");
        return -1;
    }

    g_state.core_cbs = linphone_factory_create_core_cbs(g_state.factory);
    if (g_state.core_cbs == NULL) {
        yoyopy_set_error("Failed to create Liblinphone core callbacks");
        yoyopy_liblinphone_stop();
        return -1;
    }

    g_state.message_cbs = linphone_chat_message_cbs_new();
    if (g_state.message_cbs == NULL) {
        yoyopy_set_error("Failed to create Liblinphone chat message callbacks");
        yoyopy_liblinphone_stop();
        return -1;
    }
    g_state.chat_room_cbs = linphone_factory_create_chat_room_cbs(g_state.factory);
    if (g_state.chat_room_cbs == NULL) {
        yoyopy_set_error("Failed to create Liblinphone chat room callbacks");
        yoyopy_liblinphone_stop();
        return -1;
    }

    g_state.auto_download_incoming_voice_recordings = auto_download_incoming_voice_recordings != 0;
    yoyopy_copy_string(g_state.voice_note_store_dir, sizeof(g_state.voice_note_store_dir), voice_note_store_dir);
    yoyopy_ensure_directory(g_state.voice_note_store_dir);

    linphone_chat_message_cbs_set_msg_state_changed(g_state.message_cbs, yoyopy_on_message_state_changed);
    linphone_chat_room_cbs_set_message_received(g_state.chat_room_cbs, yoyopy_on_chat_room_message_received);
    linphone_chat_room_cbs_set_messages_received(g_state.chat_room_cbs, yoyopy_on_chat_room_messages_received);
    linphone_chat_room_cbs_set_chat_message_received(
        g_state.chat_room_cbs,
        yoyopy_on_chat_room_chat_message_received
    );
    linphone_chat_room_cbs_set_undecryptable_message_received(
        g_state.chat_room_cbs,
        yoyopy_on_chat_room_undecryptable_message_received
    );
    linphone_core_cbs_set_call_state_changed(g_state.core_cbs, yoyopy_on_call_state_changed);
    linphone_core_cbs_set_subscription_state_changed(
        g_state.core_cbs,
        yoyopy_on_subscription_state_changed
    );
    linphone_core_cbs_set_messages_received(g_state.core_cbs, yoyopy_on_messages_received);
    linphone_core_cbs_set_message_received(g_state.core_cbs, yoyopy_on_message_received);
    linphone_core_cbs_set_message_received_unable_decrypt(
        g_state.core_cbs,
        yoyopy_on_message_received_unable_decrypt
    );
    linphone_core_add_callbacks(g_state.core, g_state.core_cbs);

    linphone_core_set_playback_device(g_state.core, playback_device_id);
    linphone_core_set_ringer_device(g_state.core, ringer_device_id);
    linphone_core_set_capture_device(g_state.core, capture_device_id);
    linphone_core_set_media_device(g_state.core, media_device_id);
    linphone_core_enable_chat(g_state.core);
    if (linphone_core_get_im_notif_policy(g_state.core) != NULL) {
        linphone_im_notif_policy_enable_all(linphone_core_get_im_notif_policy(g_state.core));
    }
    yoyopy_ensure_core_spec("conference/2.0");
    linphone_core_enable_echo_cancellation(g_state.core, echo_cancellation != 0);
    linphone_core_set_chat_messages_aggregation_enabled(g_state.core, FALSE);
    linphone_core_set_mic_gain_db(g_state.core, ((float)mic_gain * 0.3f));
    linphone_core_set_playback_gain_db(g_state.core, ((float)speaker_volume * 0.12f) - 6.0f);
    if (yoyopy_configure_media_policy(g_state.core, g_state.factory) != 0) {
        yoyopy_liblinphone_stop();
        return -1;
    }
    if (yoyopy_configure_network_media_defaults(g_state.core, stun_server) != 0) {
        yoyopy_liblinphone_stop();
        return -1;
    }
    if (yoyopy_apply_transports(g_state.core, transport) != 0) {
        yoyopy_liblinphone_stop();
        return -1;
    }
    if (stun_server != NULL && stun_server[0] != '\0') {
        linphone_core_set_stun_server(g_state.core, stun_server);
    }
    if (file_transfer_server_url != NULL && file_transfer_server_url[0] != '\0') {
        linphone_core_set_file_transfer_server(g_state.core, file_transfer_server_url);
    }
    linphone_core_enable_auto_download_voice_recordings(g_state.core, FALSE);

    if (
        yoyopy_configure_account(
            sip_server,
            sip_username,
            sip_password,
            sip_password_ha1,
            sip_identity,
            transport,
            conference_factory_uri,
            file_transfer_server_url,
            lime_server_url
        ) != 0
    ) {
        yoyopy_liblinphone_stop();
        return -1;
    }

    if (linphone_core_start(g_state.core) != 0) {
        yoyopy_set_error("Liblinphone core failed to start");
        yoyopy_liblinphone_stop();
        return -1;
    }
    yoyopy_prune_stale_basic_chat_rooms();
    yoyopy_attach_all_chat_room_callbacks();
    yoyopy_log_account_diagnostics("startup");

    g_state.started = true;
    yoyopy_clear_error();
    return 0;
}

void yoyopy_liblinphone_stop(void) {
    if (g_state.core != NULL) {
        linphone_core_stop(g_state.core);
    }

    yoyopy_cleanup_recorder();
    g_state.current_call = NULL;

    if (g_state.account_cbs != NULL) {
        linphone_account_cbs_unref(g_state.account_cbs);
        g_state.account_cbs = NULL;
    }
    if (g_state.message_cbs != NULL) {
        linphone_chat_message_cbs_unref(g_state.message_cbs);
        g_state.message_cbs = NULL;
    }
    if (g_state.chat_room_cbs != NULL) {
        linphone_chat_room_cbs_unref(g_state.chat_room_cbs);
        g_state.chat_room_cbs = NULL;
    }
    if (g_state.core_cbs != NULL) {
        linphone_core_cbs_unref(g_state.core_cbs);
        g_state.core_cbs = NULL;
    }
    if (g_state.account != NULL) {
        linphone_account_unref(g_state.account);
        g_state.account = NULL;
    }
    if (g_state.core != NULL) {
        linphone_core_unref(g_state.core);
        g_state.core = NULL;
    }

    pthread_mutex_lock(&g_state.queue_lock);
    g_state.queue_head = 0;
    g_state.queue_tail = 0;
    pthread_mutex_unlock(&g_state.queue_lock);

    g_state.started = false;
}

void yoyopy_liblinphone_iterate(void) {
    if (g_state.started && g_state.core != NULL) {
        linphone_core_iterate(g_state.core);
        yoyopy_prune_stale_basic_chat_rooms();
        yoyopy_attach_all_chat_room_callbacks();
    }
}

int yoyopy_liblinphone_poll_event(yoyopy_liblinphone_event_t *event_out) {
    if (event_out == NULL || !g_state.initialized) {
        return 0;
    }

    pthread_mutex_lock(&g_state.queue_lock);
    if (g_state.queue_head == g_state.queue_tail) {
        pthread_mutex_unlock(&g_state.queue_lock);
        return 0;
    }

    *event_out = g_state.queue[g_state.queue_head];
    g_state.queue_head = (g_state.queue_head + 1U) % YOYOPY_EVENT_QUEUE_CAPACITY;
    pthread_mutex_unlock(&g_state.queue_lock);
    return 1;
}

int yoyopy_liblinphone_make_call(const char *sip_address) {
    LinphoneAddress *address = NULL;
    LinphoneCallParams *params = NULL;
    LinphoneCall *call = NULL;

    if (!g_state.started || g_state.core == NULL || sip_address == NULL || sip_address[0] == '\0') {
        yoyopy_set_error("Liblinphone core is not ready to place a call");
        return -1;
    }

    address = linphone_factory_create_address(g_state.factory, sip_address);
    if (address == NULL) {
        yoyopy_set_error("Invalid SIP address for outgoing call");
        return -1;
    }

    params = linphone_core_create_call_params(g_state.core, NULL);
    if (params == NULL) {
        linphone_address_unref(address);
        yoyopy_set_error("Failed to create Liblinphone call params");
        return -1;
    }

    call = linphone_core_invite_address_with_params(g_state.core, address, params);
    linphone_call_params_unref(params);
    linphone_address_unref(address);

    if (call == NULL) {
        yoyopy_set_error("Liblinphone failed to initiate outgoing call");
        return -1;
    }

    g_state.current_call = call;
    return 0;
}

int yoyopy_liblinphone_answer_call(void) {
    if (!g_state.started || g_state.current_call == NULL) {
        yoyopy_set_error("No incoming call is available to answer");
        return -1;
    }
    return linphone_call_accept(g_state.current_call) == 0 ? 0 : -1;
}

int yoyopy_liblinphone_reject_call(void) {
    if (!g_state.started || g_state.current_call == NULL) {
        yoyopy_set_error("No incoming call is available to reject");
        return -1;
    }
    return linphone_call_decline(g_state.current_call, LinphoneReasonDeclined) == 0 ? 0 : -1;
}

int yoyopy_liblinphone_hangup(void) {
    if (!g_state.started || g_state.current_call == NULL) {
        yoyopy_set_error("No active call is available to hang up");
        return -1;
    }
    return linphone_call_terminate(g_state.current_call) == 0 ? 0 : -1;
}

int yoyopy_liblinphone_set_muted(int32_t muted) {
    if (!g_state.started || g_state.current_call == NULL) {
        yoyopy_set_error("No active call is available to mute");
        return -1;
    }
    linphone_call_set_microphone_muted(g_state.current_call, muted ? TRUE : FALSE);
    return 0;
}

static LinphoneChatRoom *yoyopy_get_chat_room_for_params(
    const char *sip_address,
    LinphoneChatRoomParams *params,
    const char *phase
) {
    LinphoneChatRoom *chat_room = NULL;
    LinphoneAddress *remote_address = NULL;
    const LinphoneAddress *local_address = NULL;
    const LinphoneAccountParams *account_params = NULL;
    bctbx_list_t *participants = NULL;

    if (!g_state.started || g_state.core == NULL) {
        if (params != NULL) {
            linphone_chat_room_params_unref(params);
        }
        return NULL;
    }

    remote_address = linphone_factory_create_address(g_state.factory, sip_address);
    account_params = g_state.account != NULL ? linphone_account_get_params(g_state.account) : NULL;
    local_address = account_params != NULL ? linphone_account_params_get_identity_address(account_params) : NULL;

    if (params != NULL && remote_address != NULL) {
        chat_room = linphone_core_search_chat_room(
            g_state.core,
            params,
            local_address,
            remote_address,
            NULL
        );
        if (chat_room == NULL) {
            participants = bctbx_list_append(participants, remote_address);
            chat_room = linphone_core_create_chat_room_6(
                g_state.core,
                params,
                local_address,
                participants
            );
        }
    }

    if (chat_room == NULL) {
        chat_room = linphone_core_get_chat_room_from_uri(g_state.core, sip_address);
    }

    yoyopy_attach_chat_room_callbacks(chat_room);
    if (chat_room != NULL) {
        yoyopy_log_room_snapshot(chat_room, phase != NULL ? phase : "lookup");
    }

    if (participants != NULL) {
        bctbx_list_free(participants);
    }
    if (remote_address != NULL) {
        linphone_address_unref(remote_address);
    }
    if (params != NULL) {
        linphone_chat_room_params_unref(params);
    }
    return chat_room;
}

static LinphoneChatRoom *yoyopy_get_chat_room(const char *sip_address) {
    return yoyopy_get_chat_room_for_params(
        sip_address,
        yoyopy_create_preferred_chat_room_params(),
        "lookup"
    );
}

static LinphoneChatRoom *yoyopy_get_direct_chat_room(const char *sip_address) {
    return yoyopy_get_chat_room_for_params(
        sip_address,
        yoyopy_create_direct_chat_room_params(),
        "direct_lookup"
    );
}

static void yoyopy_fill_message_id_out(
    LinphoneChatMessage *message,
    char *message_id_out,
    uint32_t message_id_out_size
) {
    char message_id[128];
    yoyopy_build_message_id(message, message_id, sizeof(message_id));
    if (message_id_out != NULL && message_id_out_size > 0) {
        snprintf(message_id_out, message_id_out_size, "%s", message_id);
    }
}

int yoyopy_liblinphone_send_text_message(
    const char *sip_address,
    const char *text,
    char *message_id_out,
    uint32_t message_id_out_size
) {
    LinphoneChatRoom *chat_room;
    LinphoneChatMessage *message;

    if (!g_state.started || sip_address == NULL || sip_address[0] == '\0' || text == NULL) {
        yoyopy_set_error("Liblinphone text message send is missing peer or payload");
        return -1;
    }

    chat_room = yoyopy_get_direct_chat_room(sip_address);
    if (chat_room == NULL) {
        yoyopy_set_error("Liblinphone could not resolve a chat room for %s", sip_address);
        return -1;
    }

    message = linphone_chat_room_create_message_from_utf8(chat_room, text);
    if (message == NULL) {
        yoyopy_set_error("Liblinphone failed to create a text chat message");
        return -1;
    }

    yoyopy_attach_message_callbacks(message);
    yoyopy_fill_message_id_out(message, message_id_out, message_id_out_size);
    linphone_chat_message_send(message);
    return 0;
}

int yoyopy_liblinphone_start_voice_recording(const char *file_path) {
    LinphoneRecorderParams *params;
    if (!g_state.started || g_state.core == NULL || file_path == NULL || file_path[0] == '\0') {
        yoyopy_set_error("Liblinphone voice-note recording requires an active core and target path");
        return -1;
    }

    yoyopy_cleanup_recorder();
    params = linphone_core_create_recorder_params(g_state.core);
    if (params == NULL) {
        yoyopy_set_error("Failed to create Liblinphone recorder params");
        return -1;
    }
    linphone_recorder_params_set_file_format(params, LinphoneRecorderFileFormatWav);

    g_state.current_recorder = linphone_core_create_recorder(g_state.core, params);
    linphone_recorder_params_unref(params);
    if (g_state.current_recorder == NULL) {
        yoyopy_set_error("Failed to create Liblinphone recorder");
        return -1;
    }

    yoyopy_copy_string(g_state.current_recording_path, sizeof(g_state.current_recording_path), file_path);
    yoyopy_ensure_directory(g_state.voice_note_store_dir);
    if (linphone_recorder_open(g_state.current_recorder, file_path) != 0) {
        yoyopy_set_error("Failed to open voice-note file for recording");
        yoyopy_cleanup_recorder();
        return -1;
    }
    if (linphone_recorder_start(g_state.current_recorder) != 0) {
        yoyopy_set_error("Failed to start voice-note recording");
        yoyopy_cleanup_recorder();
        return -1;
    }

    g_state.recorder_running = true;
    return 0;
}

int yoyopy_liblinphone_stop_voice_recording(int32_t *duration_ms_out) {
    int duration_ms;
    if (!g_state.started || g_state.current_recorder == NULL || !g_state.recorder_running) {
        yoyopy_set_error("No active Liblinphone voice-note recording is running");
        return -1;
    }

    linphone_recorder_pause(g_state.current_recorder);
    g_state.recorder_running = false;
    duration_ms = linphone_recorder_get_duration(g_state.current_recorder);
    linphone_recorder_close(g_state.current_recorder);
    if (duration_ms_out != NULL) {
        *duration_ms_out = duration_ms;
    }
    return 0;
}

int yoyopy_liblinphone_cancel_voice_recording(void) {
    if (g_state.current_recording_path[0] != '\0') {
        unlink(g_state.current_recording_path);
    }
    yoyopy_cleanup_recorder();
    return 0;
}

int yoyopy_liblinphone_send_voice_note(
    const char *sip_address,
    const char *file_path,
    int32_t duration_ms,
    const char *mime_type,
    char *message_id_out,
    uint32_t message_id_out_size
) {
    LinphoneChatRoom *chat_room;
    LinphoneChatMessage *message;

    (void)duration_ms;
    (void)mime_type;

    if (!g_state.started || g_state.current_recorder == NULL || sip_address == NULL || sip_address[0] == '\0') {
        yoyopy_set_error("Liblinphone voice-note send requires a closed recording and recipient");
        return -1;
    }
    if (g_state.recorder_running) {
        yoyopy_set_error("Voice-note recording must be stopped before sending");
        return -1;
    }
    if (file_path != NULL && file_path[0] != '\0' && strcmp(file_path, g_state.current_recording_path) != 0) {
        yoyopy_set_error("Voice-note send only supports the active recorder output in this build");
        return -1;
    }
    if (!yoyopy_path_exists(g_state.current_recording_path)) {
        yoyopy_set_error("Voice-note file does not exist at %s", g_state.current_recording_path);
        return -1;
    }

    chat_room = yoyopy_get_direct_chat_room(sip_address);
    if (chat_room == NULL) {
        yoyopy_set_error("Liblinphone could not resolve a chat room for %s", sip_address);
        return -1;
    }

    message = linphone_chat_room_create_voice_recording_message(chat_room, g_state.current_recorder);
    if (message == NULL) {
        yoyopy_set_error("Liblinphone failed to create a voice-note message");
        return -1;
    }

    yoyopy_attach_message_callbacks(message);
    yoyopy_fill_message_id_out(message, message_id_out, message_id_out_size);
    linphone_chat_message_send(message);
    return 0;
}

const char *yoyopy_liblinphone_last_error(void) {
    return g_last_error;
}

const char *yoyopy_liblinphone_version(void) {
    return linphone_core_get_version();
}
