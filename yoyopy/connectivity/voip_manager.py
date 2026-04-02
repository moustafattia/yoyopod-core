"""
VoIP Manager for YoyoPod using Linphone.

Provides SIP/VoIP calling capability by interfacing with linphonec CLI.
Handles registration, call management, and status monitoring.
"""

import subprocess
import threading
import time
from enum import Enum
from typing import Optional, Callable, List
from dataclasses import dataclass
from loguru import logger


class RegistrationState(Enum):
    """SIP registration states."""
    NONE = "none"
    PROGRESS = "progress"
    OK = "ok"
    CLEARED = "cleared"
    FAILED = "failed"


class CallState(Enum):
    """Call states."""
    IDLE = "idle"
    INCOMING = "incoming"
    OUTGOING = "outgoing_init"
    OUTGOING_PROGRESS = "outgoing_progress"
    OUTGOING_RINGING = "outgoing_ringing"
    OUTGOING_EARLY_MEDIA = "outgoing_early_media"
    CONNECTED = "connected"
    STREAMS_RUNNING = "streams_running"
    PAUSED = "paused"
    PAUSED_BY_REMOTE = "paused_by_remote"
    UPDATED_BY_REMOTE = "updated_by_remote"
    RELEASED = "released"
    ERROR = "error"
    END = "end"


@dataclass
class VoIPConfig:
    """VoIP configuration."""
    sip_server: str = "sip.linphone.org"
    sip_username: str = ""
    sip_password: str = ""
    sip_password_ha1: str = ""  # HA1 hash (SHA-256) for SIP authentication
    sip_identity: str = ""  # sip:username@server
    transport: str = "tcp"  # tcp, udp, tls
    stun_server: str = ""
    linphonec_path: str = "/usr/bin/linphonec"
    playback_dev_id: str = "ALSA: plughw:1"
    ringer_dev_id: str = "ALSA: plughw:1"
    capture_dev_id: str = "ALSA: plughw:1"
    media_dev_id: str = "ALSA: plughw:1"

    @staticmethod
    def from_config_manager(config_manager) -> 'VoIPConfig':
        """
        Create VoIPConfig from ConfigManager.

        Args:
            config_manager: ConfigManager instance

        Returns:
            VoIPConfig instance
        """
        return VoIPConfig(
            sip_server=config_manager.get_sip_server(),
            sip_username=config_manager.get_sip_username(),
            sip_password=config_manager.get_sip_password(),
            sip_password_ha1=config_manager.get_sip_password_ha1(),
            sip_identity=config_manager.get_sip_identity(),
            transport=config_manager.get_transport(),
            stun_server=config_manager.get_stun_server(),
            linphonec_path=config_manager.get_linphonec_path(),
            playback_dev_id=config_manager.get_playback_device_id(),
            ringer_dev_id=config_manager.get_ringer_device_id(),
            capture_dev_id=config_manager.get_capture_device_id(),
            media_dev_id=config_manager.get_media_device_id(),
        )


class VoIPManager:
    """
    VoIP Manager using Linphone CLI.

    Manages SIP registration and call handling through linphonec subprocess.
    """

    def __init__(self, config: VoIPConfig, config_manager=None) -> None:
        """
        Initialize VoIP manager.

        Args:
            config: VoIP configuration
            config_manager: Optional ConfigManager for contact name lookup
        """
        self.config = config
        self.config_manager = config_manager  # For contact lookup
        self.process: Optional[subprocess.Popen] = None
        self.running = False
        self.registered = False
        self.registration_state = RegistrationState.NONE
        self.call_state = CallState.IDLE
        self.current_call_id: Optional[str] = None
        self.caller_address: Optional[str] = None  # SIP address of caller/callee
        self.caller_name: Optional[str] = None  # Display name of caller/callee
        self.call_duration: int = 0  # Call duration in seconds
        self.call_start_time: Optional[float] = None  # Time when call became active
        self.is_muted: bool = False  # Microphone mute state

        # Callbacks
        self.registration_callbacks: List[Callable[[RegistrationState], None]] = []
        self.call_state_callbacks: List[Callable[[CallState], None]] = []
        self.incoming_call_callbacks: List[Callable[[str, str], None]] = []  # (address, name)

        # Monitor thread
        self.monitor_thread: Optional[threading.Thread] = None
        self.monitor_event = threading.Event()

        # Call duration tracking thread
        self.duration_thread: Optional[threading.Thread] = None
        self.duration_stop_event = threading.Event()

        logger.info(f"VoIPManager initialized (server: {config.sip_server})")

    def _generate_linphonerc(self) -> bool:
        """
        Generate .linphonerc configuration file from config.

        Returns:
            True if generated successfully, False otherwise
        """
        if not self.config.sip_password_ha1 or not self.config.sip_identity:
            logger.warning("Cannot generate .linphonerc: missing HA1 hash or identity")
            return False

        try:
            import os
            linphonerc_path = os.path.expanduser("~/.linphonerc")

            # Linphonerc configuration
            config_content = f"""# Generated by YoyoPod VoIPManager
[sip]
sip_port=-1
sip_tcp_port=-1
sip_tls_port=-1
default_proxy=0
register_only_when_network_is_up=1
register_only_when_upnp_is_ok=0
guess_hostname=1
inc_timeout=30
in_call_timeout=0
delayed_timeout=4
use_info=0
use_rfc2833=1
use_ipv6=0
record_aware=0
media_encryption=srtp

[auth_info_0]
username={self.config.sip_username}
userid={self.config.sip_username}
ha1={self.config.sip_password_ha1}
realm={self.config.sip_server}
domain={self.config.sip_server}
algorithm=SHA-256
available_algorithms=SHA-256

[proxy_0]
reg_proxy=<sip:{self.config.sip_server};transport={self.config.transport}>
reg_identity={self.config.sip_identity}
reg_expires=3600
reg_sendregister=1
publish=0
dial_escape_plus=0
nat_policy_ref=ice_nat_policy

[nat_policy_0]
ref=ice_nat_policy
stun_server={self.config.stun_server if self.config.stun_server else 'stun.linphone.org'}
ice_enabled=1
turn_enabled=0
upnp_enabled=0

[rtp]
audio_rtp_port=7076-7100
video_rtp_port=9076-9100
audio_jitt_comp=60
video_jitt_comp=60
nortp_timeout=30
audio_adaptive_jitt_comp_enabled=1
video_adaptive_jitt_comp_enabled=1

[net]
download_bw=380
upload_bw=60
adaptive_rate_control=1
mtu=1300

[sound]
playback_dev_id={self.config.playback_dev_id}
ringer_dev_id={self.config.ringer_dev_id}
capture_dev_id={self.config.capture_dev_id}
media_dev_id={self.config.media_dev_id}
echocancellation=1
mic_gain_db=10.0
"""

            with open(linphonerc_path, 'w') as f:
                f.write(config_content)

            logger.info(f"Generated .linphonerc at {linphonerc_path}")
            return True

        except Exception as e:
            logger.error(f"Failed to generate .linphonerc: {e}")
            return False

    def start(self) -> bool:
        """
        Start linphonec process and begin monitoring.

        Returns:
            True if started successfully, False otherwise
        """
        if self.running:
            logger.warning("VoIP manager already running")
            return True

        try:
            # Generate .linphonerc configuration file
            if not self._generate_linphonerc():
                logger.warning("Failed to generate .linphonerc, will try to use existing configuration")

            logger.info("Starting linphonec...")

            # Start linphonec in daemon mode with pipe interface
            self.process = subprocess.Popen(
                [self.config.linphonec_path, "-d", "6"],  # -d 6 = debug level
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1
            )

            self.running = True

            # Start monitor thread
            self.monitor_thread = threading.Thread(
                target=self._monitor_output,
                daemon=True
            )
            self.monitor_thread.start()

            logger.info("Linphonec started successfully")

            # Give it a moment to initialize
            time.sleep(2)

            # Check registration status
            self._send_command("status register")

            return True

        except Exception as e:
            logger.error(f"Failed to start linphonec: {e}")
            self.running = False
            return False

    def stop(self) -> None:
        """Stop linphonec process and monitoring."""
        if not self.running:
            return

        logger.info("Stopping VoIP manager...")

        self.running = False
        self.monitor_event.set()

        # Terminate linphonec
        if self.process:
            try:
                self._send_command("quit")
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.terminate()
                self.process.wait(timeout=1)
            except Exception as e:
                logger.error(f"Error stopping linphonec: {e}")
            finally:
                self.process = None

        # Wait for monitor thread
        if self.monitor_thread:
            self.monitor_thread.join(timeout=2)
            self.monitor_thread = None

        logger.info("VoIP manager stopped")

    def _send_command(self, command: str) -> bool:
        """
        Send command to linphonec.

        Args:
            command: Command to send

        Returns:
            True if command sent successfully
        """
        if not self.process or not self.process.stdin:
            logger.error("Cannot send command: linphonec not running")
            return False

        try:
            self.process.stdin.write(f"{command}\n")
            self.process.stdin.flush()
            logger.debug(f"Sent command: {command}")
            return True
        except Exception as e:
            logger.error(f"Failed to send command '{command}': {e}")
            return False

    def _monitor_output(self) -> None:
        """Monitor linphonec output for events."""
        logger.debug("Output monitor started")

        while self.running and self.process:
            try:
                if not self.process.stdout:
                    break

                line = self.process.stdout.readline()
                if not line:
                    if self.process.poll() is not None:
                        logger.warning("Linphonec process terminated")
                        self.running = False
                        break
                    continue

                line = line.strip()
                if line:
                    self._parse_output(line)

            except Exception as e:
                logger.error(f"Error monitoring output: {e}")
                break

        logger.debug("Output monitor stopped")

    def _parse_output(self, line: str) -> None:
        """
        Parse linphonec output line.

        Args:
            line: Output line to parse
        """
        logger.debug(f"Linphone: {line}")

        # Parse registration state
        if "Registration on" in line and "successful" in line:
            self._update_registration_state(RegistrationState.OK)
        elif "Registration on" in line and "failed" in line:
            self._update_registration_state(RegistrationState.FAILED)
        elif "Registration on" in line and "cleared" in line:
            self._update_registration_state(RegistrationState.CLEARED)
        elif "Refreshing" in line and "registration" in line:
            logger.debug("Registration refresh in progress")
        # New pattern for Linphone 5.x
        elif "LinphoneRegistrationOk" in line or ("Registration successful" in line and "reason" in line):
            self._update_registration_state(RegistrationState.OK)
        elif "LinphoneRegistrationProgress" in line:
            self._update_registration_state(RegistrationState.PROGRESS)
        elif "LinphoneRegistrationFailed" in line or ("Registration failed" in line and "reason" in line):
            self._update_registration_state(RegistrationState.FAILED)
        elif "LinphoneRegistrationCleared" in line:
            self._update_registration_state(RegistrationState.CLEARED)

        # Parse call state and extract caller info
        # Look for lines containing call information (case-insensitive)
        line_lower = line.lower()
        if "call" in line_lower or "callsession" in line_lower:
            # Try to extract SIP address from line
            # Linphone format examples:
            # - Linphone 5.x: "New incoming call from [sip:user@domain]"
            # - Linphone 4.x: "Receiving new incoming call from <sip:user@domain>"
            # - Linphone 4.x: "Call from <sip:user@domain>"
            if "from" in line_lower:
                # Try Linphone 5.x format with square brackets [sip:...]
                if "[sip:" in line:
                    start = line.find("[sip:")
                    end = line.find("]", start)
                    if start != -1 and end != -1:
                        self.caller_address = line[start+1:end]  # Remove [ ]
                        self.caller_name = self._lookup_contact_name(self.caller_address)
                        logger.debug(f"Extracted caller address: {self.caller_address}, name: {self.caller_name}")
                # Try Linphone 4.x format with angle brackets <sip:...>
                elif "<sip:" in line:
                    start = line.find("<sip:")
                    end = line.find(">", start)
                    if start != -1 and end != -1:
                        self.caller_address = line[start+1:end]  # Remove < >
                        self.caller_name = self._lookup_contact_name(self.caller_address)
                        logger.debug(f"Extracted caller address: {self.caller_address}, name: {self.caller_name}")
                # Fallback: try to extract plain sip: address
                elif "sip:" in line.lower():
                    # Extract from "from sip:user@domain" or similar
                    parts = line.lower().split("from")
                    if len(parts) > 1:
                        # Get everything after "from"
                        after_from = parts[1].strip()
                        # Extract sip address
                        if after_from.startswith("sip:"):
                            # Find end of SIP address (space, comma, or end of line)
                            end_chars = [' ', ',', '\t', '\n']
                            end_pos = len(after_from)
                            for char in end_chars:
                                pos = after_from.find(char)
                                if pos != -1 and pos < end_pos:
                                    end_pos = pos
                            self.caller_address = after_from[:end_pos]
                            self.caller_name = self._lookup_contact_name(self.caller_address)
                            logger.debug(f"Extracted caller address (fallback): {self.caller_address}, name: {self.caller_name}")

            # Linphone 5.x pattern: "LinphoneCallIncoming"
            if "LinphoneCallIncoming" in line or "incoming" in line.lower():
                self._update_call_state(CallState.INCOMING)
                # Fire incoming call callbacks
                if self.caller_address:
                    for callback in self.incoming_call_callbacks:
                        try:
                            callback(self.caller_address, self.caller_name or self._extract_username(self.caller_address))
                        except Exception as e:
                            logger.error(f"Error in incoming call callback: {e}")
            elif "outgoing" in line.lower():
                self._update_call_state(CallState.OUTGOING)
            elif "connected" in line.lower() or "streams running" in line.lower() or "LinphoneCallConnected" in line:
                self._update_call_state(CallState.CONNECTED)
                if not self.call_start_time:
                    self._start_call_timer()
            elif "released" in line.lower() or "ended" in line.lower() or "LinphoneCallReleased" in line:
                self._update_call_state(CallState.RELEASED)
                self._stop_call_timer()
                self.current_call_id = None
                self.caller_address = None
                self.caller_name = None

    def _update_registration_state(self, state: RegistrationState) -> None:
        """
        Update registration state and fire callbacks.

        Args:
            state: New registration state
        """
        if state != self.registration_state:
            old_state = self.registration_state
            self.registration_state = state
            self.registered = (state == RegistrationState.OK)

            logger.info(f"Registration state: {old_state.value} -> {state.value}")

            for callback in self.registration_callbacks:
                try:
                    callback(state)
                except Exception as e:
                    logger.error(f"Error in registration callback: {e}")

    def _update_call_state(self, state: CallState) -> None:
        """
        Update call state and fire callbacks.

        Args:
            state: New call state
        """
        if state != self.call_state:
            old_state = self.call_state
            self.call_state = state

            logger.info(f"Call state: {old_state.value} -> {state.value}")

            for callback in self.call_state_callbacks:
                try:
                    callback(state)
                except Exception as e:
                    logger.error(f"Error in call state callback: {e}")

    def make_call(self, sip_address: str, contact_name: str = None) -> bool:
        """
        Initiate outgoing call.

        Args:
            sip_address: SIP address to call (e.g., sip:user@domain)
            contact_name: Optional contact name (will be looked up if not provided)

        Returns:
            True if call initiated successfully
        """
        if not self.registered:
            logger.error("Cannot make call: not registered")
            return False

        # Store caller info for outgoing call
        self.caller_address = sip_address
        self.caller_name = contact_name or self._lookup_contact_name(sip_address)

        logger.info(f"Making call to: {self.caller_name} ({sip_address})")
        return self._send_command(f"call {sip_address}")

    def answer_call(self) -> bool:
        """
        Answer incoming call.

        Returns:
            True if answered successfully
        """
        logger.info("Answering call")
        return self._send_command("answer")

    def hangup(self) -> bool:
        """
        Hangup current call.

        Returns:
            True if hangup command sent successfully
        """
        logger.info("Hanging up call")
        return self._send_command("terminate")

    def get_status(self) -> dict:
        """
        Get current VoIP status.

        Returns:
            Status dictionary
        """
        return {
            "running": self.running,
            "registered": self.registered,
            "registration_state": self.registration_state.value,
            "call_state": self.call_state.value,
            "call_id": self.current_call_id,
            "sip_identity": self.config.sip_identity
        }

    def on_registration_change(self, callback: Callable[[RegistrationState], None]) -> None:
        """
        Register callback for registration state changes.

        Args:
            callback: Function to call on state change
        """
        self.registration_callbacks.append(callback)

    def on_call_state_change(self, callback: Callable[[CallState], None]) -> None:
        """
        Register callback for call state changes.

        Args:
            callback: Function to call on state change
        """
        self.call_state_callbacks.append(callback)

    def mute(self) -> bool:
        """
        Mute microphone.

        Returns:
            True if muted successfully
        """
        if not self.is_muted:
            logger.info("Muting microphone")
            if self._send_command("mute"):
                self.is_muted = True
                return True
        return False

    def unmute(self) -> bool:
        """
        Unmute microphone.

        Returns:
            True if unmuted successfully
        """
        if self.is_muted:
            logger.info("Unmuting microphone")
            if self._send_command("unmute"):
                self.is_muted = False
                return True
        return False

    def toggle_mute(self) -> bool:
        """
        Toggle microphone mute state.

        Returns:
            True if now muted, False if unmuted
        """
        if self.is_muted:
            self.unmute()
            return False
        else:
            self.mute()
            return True

    def reject_call(self) -> bool:
        """
        Reject incoming call.

        Returns:
            True if rejected successfully
        """
        logger.info("Rejecting call")
        return self._send_command("decline")

    def get_call_duration(self) -> int:
        """
        Get current call duration in seconds.

        Returns:
            Duration in seconds, or 0 if no active call
        """
        if self.call_start_time and self.call_state in [CallState.CONNECTED, CallState.STREAMS_RUNNING]:
            return int(time.time() - self.call_start_time)
        return 0

    def get_caller_info(self) -> dict:
        """
        Get information about current caller/callee.

        Returns:
            Dictionary with caller information
        """
        # If we have an address but no name, look it up
        if self.caller_address and not self.caller_name:
            self.caller_name = self._lookup_contact_name(self.caller_address)

        return {
            "address": self.caller_address,
            "name": self.caller_name or self.caller_address,
            "display_name": self.caller_name or self._lookup_contact_name(self.caller_address)
        }

    def _extract_username(self, sip_address: Optional[str]) -> str:
        """
        Extract username from SIP address.

        Args:
            sip_address: SIP URI (e.g., sip:user@domain)

        Returns:
            Username or full address if parsing fails
        """
        if not sip_address:
            return "Unknown"

        # Extract username from sip:username@domain
        if "@" in sip_address:
            username_part = sip_address.split("@")[0]
            if ":" in username_part:
                return username_part.split(":")[-1]
            return username_part
        return sip_address

    def _lookup_contact_name(self, sip_address: Optional[str]) -> str:
        """
        Look up contact name from SIP address.

        Args:
            sip_address: SIP URI to look up

        Returns:
            Contact name if found, otherwise extracted username
        """
        if not sip_address:
            return "Unknown"

        # Try to look up contact in config_manager
        if self.config_manager:
            contact = self.config_manager.get_contact_by_address(sip_address)
            if contact:
                logger.debug(f"Found contact: {contact.name} for {sip_address}")
                return contact.name

        # Fall back to extracting username from SIP address
        return self._extract_username(sip_address)

    def _start_call_timer(self) -> None:
        """Start tracking call duration."""
        self.call_start_time = time.time()
        self.call_duration = 0

        # Start duration tracking thread
        self.duration_stop_event.clear()
        self.duration_thread = threading.Thread(
            target=self._track_duration,
            daemon=True
        )
        self.duration_thread.start()
        logger.debug("Call duration timer started")

    def _stop_call_timer(self) -> None:
        """Stop tracking call duration."""
        self.duration_stop_event.set()
        if self.duration_thread:
            self.duration_thread.join(timeout=1)
            self.duration_thread = None
        self.call_start_time = None
        logger.debug("Call duration timer stopped")

    def _track_duration(self) -> None:
        """Background thread to track call duration."""
        while not self.duration_stop_event.is_set():
            if self.call_start_time:
                self.call_duration = int(time.time() - self.call_start_time)
            time.sleep(1)

    def on_incoming_call(self, callback: Callable[[str, str], None]) -> None:
        """
        Register callback for incoming calls.

        Args:
            callback: Function to call with (caller_address, caller_name)
        """
        self.incoming_call_callbacks.append(callback)

    def cleanup(self) -> None:
        """Clean up resources."""
        self._stop_call_timer()
        self.stop()
        logger.info("VoIP manager cleaned up")
