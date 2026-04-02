"""
Configuration Manager for YoyoPod.

Manages VoIP settings and contacts from YAML configuration files.
"""

import os
import yaml
from pathlib import Path
from typing import List, Optional, Dict, Any
from dataclasses import dataclass
from loguru import logger


@dataclass
class Contact:
    """Represents a VoIP contact."""
    name: str
    sip_address: str
    favorite: bool = False
    notes: str = ""

    def __str__(self) -> str:
        return f"{self.name} ({self.sip_address})"


class ConfigManager:
    """
    Manages application configuration.

    Loads VoIP settings and contacts from YAML files.
    """

    def __init__(self, config_dir: str = "config") -> None:
        """
        Initialize configuration manager.

        Args:
            config_dir: Directory containing configuration files
        """
        self.config_dir = Path(config_dir)
        self.voip_config_file = self.config_dir / "voip_config.yaml"
        self.contacts_file = self.config_dir / "contacts.yaml"

        self.voip_config: Dict[str, Any] = {}
        self.contacts: List[Contact] = []
        self.speed_dial: Dict[int, str] = {}

        logger.info(f"ConfigManager initialized (config_dir: {config_dir})")

        # Load configurations
        self.load_voip_config()
        self.load_contacts()

    def load_voip_config(self) -> bool:
        """
        Load VoIP configuration from file.

        Returns:
            True if loaded successfully, False otherwise
        """
        if not self.voip_config_file.exists():
            logger.warning(f"VoIP config file not found: {self.voip_config_file}")
            self._create_default_voip_config()
            return False

        try:
            with open(self.voip_config_file, 'r') as f:
                self.voip_config = yaml.safe_load(f) or {}

            logger.info("VoIP configuration loaded successfully")
            logger.debug(f"SIP Server: {self.get_sip_server()}")
            logger.debug(f"SIP Identity: {self.get_sip_identity()}")
            return True

        except Exception as e:
            logger.error(f"Error loading VoIP config: {e}")
            return False

    def load_contacts(self) -> bool:
        """
        Load contacts from file.

        Returns:
            True if loaded successfully, False otherwise
        """
        if not self.contacts_file.exists():
            logger.warning(f"Contacts file not found: {self.contacts_file}")
            self._create_default_contacts()
            return False

        try:
            with open(self.contacts_file, 'r') as f:
                data = yaml.safe_load(f) or {}

            # Load contacts
            self.contacts = []
            for contact_data in data.get('contacts', []):
                contact = Contact(
                    name=contact_data.get('name', ''),
                    sip_address=contact_data.get('sip_address', ''),
                    favorite=contact_data.get('favorite', False),
                    notes=contact_data.get('notes', '')
                )
                self.contacts.append(contact)

            # Load speed dial
            self.speed_dial = data.get('speed_dial', {})

            logger.info(f"Loaded {len(self.contacts)} contacts")
            return True

        except Exception as e:
            logger.error(f"Error loading contacts: {e}")
            return False

    def save_contacts(self) -> bool:
        """
        Save contacts to file.

        Returns:
            True if saved successfully, False otherwise
        """
        try:
            data = {
                'contacts': [
                    {
                        'name': c.name,
                        'sip_address': c.sip_address,
                        'favorite': c.favorite,
                        'notes': c.notes
                    }
                    for c in self.contacts
                ],
                'speed_dial': self.speed_dial
            }

            with open(self.contacts_file, 'w') as f:
                yaml.dump(data, f, default_flow_style=False, sort_keys=False)

            logger.info("Contacts saved successfully")
            return True

        except Exception as e:
            logger.error(f"Error saving contacts: {e}")
            return False

    def _create_default_voip_config(self) -> None:
        """Create default VoIP configuration file."""
        logger.info("Creating default VoIP configuration...")
        self.config_dir.mkdir(parents=True, exist_ok=True)

        default_config = {
            'account': {
                'sip_server': 'sip.linphone.org',
                'sip_username': '',
                'sip_password': '',
                'sip_identity': '',
                'transport': 'tcp',
                'display_name': 'YoyoPod'
            },
            'network': {
                'stun_server': 'stun.linphone.org',
                'enable_ice': True
            },
            'audio': {
                'preferred_codec': 'opus',
                'echo_cancellation': True,
                'mic_gain': 80,
                'speaker_volume': 80,
                'playback_device_id': 'ALSA: plughw:1',
                'ringer_device_id': 'ALSA: plughw:1',
                'capture_device_id': 'ALSA: plughw:1',
                'media_device_id': 'ALSA: plughw:1',
                'ring_output_device': 'plughw:1'
            },
            'linphonec_path': '/usr/bin/linphonec',
            'auto_answer': False,
            'call_timeout': 60
        }

        with open(self.voip_config_file, 'w') as f:
            yaml.dump(default_config, f, default_flow_style=False, sort_keys=False)

        self.voip_config = default_config

    def _create_default_contacts(self) -> None:
        """Create default contacts file."""
        logger.info("Creating default contacts file...")
        self.config_dir.mkdir(parents=True, exist_ok=True)

        default_contacts = {
            'contacts': [
                {
                    'name': 'Echo Test Service',
                    'sip_address': 'sip:echo@sip.linphone.org',
                    'favorite': True,
                    'notes': 'Linphone echo test'
                }
            ],
            'speed_dial': {
                1: 'sip:echo@sip.linphone.org'
            }
        }

        with open(self.contacts_file, 'w') as f:
            yaml.dump(default_contacts, f, default_flow_style=False, sort_keys=False)

        self.load_contacts()

    # VoIP Configuration Getters

    def get_sip_server(self) -> str:
        """Get SIP server address."""
        return self.voip_config.get('account', {}).get('sip_server', '')

    def get_sip_username(self) -> str:
        """Get SIP username."""
        return self.voip_config.get('account', {}).get('sip_username', '')

    def get_sip_password(self) -> str:
        """Get SIP password."""
        return self.voip_config.get('account', {}).get('sip_password', '')

    def get_sip_password_ha1(self) -> str:
        """Get SIP password HA1 hash."""
        return self.voip_config.get('account', {}).get('sip_password_ha1', '')

    def get_sip_identity(self) -> str:
        """Get SIP identity."""
        return self.voip_config.get('account', {}).get('sip_identity', '')

    def get_transport(self) -> str:
        """Get transport protocol."""
        return self.voip_config.get('account', {}).get('transport', 'tcp')

    def get_display_name(self) -> str:
        """Get display name."""
        return self.voip_config.get('account', {}).get('display_name', 'YoyoPod')

    def get_stun_server(self) -> str:
        """Get STUN server address."""
        return self.voip_config.get('network', {}).get('stun_server', '')

    def get_linphonec_path(self) -> str:
        """Get linphonec executable path."""
        return self.voip_config.get('linphonec_path', '/usr/bin/linphonec')

    def get_auto_answer(self) -> bool:
        """Get auto-answer setting."""
        return self.voip_config.get('auto_answer', False)

    def get_call_timeout(self) -> int:
        """Get call timeout in seconds."""
        return self.voip_config.get('call_timeout', 60)

    def _get_audio_setting(self, key: str, env_var: str, default: str = "") -> str:
        """Get an audio setting from env override, config, or default."""
        env_value = os.getenv(env_var)
        if env_value:
            return env_value
        return self.voip_config.get('audio', {}).get(key, default)

    def get_playback_device_id(self) -> str:
        """Get the ALSA playback device id for Linphone."""
        return self._get_audio_setting(
            'playback_device_id',
            'YOYOPOD_PLAYBACK_DEVICE',
            'ALSA: plughw:1',
        )

    def get_ringer_device_id(self) -> str:
        """Get the ALSA ringer device id for Linphone."""
        return self._get_audio_setting(
            'ringer_device_id',
            'YOYOPOD_RINGER_DEVICE',
            self.get_playback_device_id(),
        )

    def get_capture_device_id(self) -> str:
        """Get the ALSA capture device id for Linphone."""
        return self._get_audio_setting(
            'capture_device_id',
            'YOYOPOD_CAPTURE_DEVICE',
            'ALSA: plughw:1',
        )

    def get_media_device_id(self) -> str:
        """Get the ALSA media device id for Linphone."""
        return self._get_audio_setting(
            'media_device_id',
            'YOYOPOD_MEDIA_DEVICE',
            self.get_playback_device_id(),
        )

    def get_ring_output_device(self) -> str:
        """Get the output device for the speaker-test ring tone helper."""
        ring_output = self._get_audio_setting(
            'ring_output_device',
            'YOYOPOD_RING_OUTPUT_DEVICE',
            '',
        )
        if ring_output:
            return ring_output

        playback_device = self.get_playback_device_id()
        if playback_device.startswith("ALSA:"):
            return playback_device.split(":", 1)[1].strip()
        return playback_device or "default"

    # Contact Management

    def get_contacts(self, favorites_only: bool = False) -> List[Contact]:
        """
        Get list of contacts.

        Args:
            favorites_only: If True, return only favorite contacts

        Returns:
            List of contacts
        """
        if favorites_only:
            return [c for c in self.contacts if c.favorite]
        return self.contacts

    def get_contact_by_name(self, name: str) -> Optional[Contact]:
        """
        Get contact by name.

        Args:
            name: Contact name

        Returns:
            Contact if found, None otherwise
        """
        for contact in self.contacts:
            if contact.name.lower() == name.lower():
                return contact
        return None

    def get_contact_by_address(self, sip_address: str) -> Optional[Contact]:
        """
        Get contact by SIP address.

        Args:
            sip_address: SIP address

        Returns:
            Contact if found, None otherwise
        """
        for contact in self.contacts:
            if contact.sip_address == sip_address:
                return contact
        return None

    def add_contact(self, name: str, sip_address: str,
                   favorite: bool = False, notes: str = "") -> Contact:
        """
        Add a new contact.

        Args:
            name: Contact name
            sip_address: SIP address
            favorite: Mark as favorite
            notes: Additional notes

        Returns:
            Created contact
        """
        contact = Contact(
            name=name,
            sip_address=sip_address,
            favorite=favorite,
            notes=notes
        )
        self.contacts.append(contact)
        self.save_contacts()
        logger.info(f"Added contact: {contact}")
        return contact

    def remove_contact(self, name: str) -> bool:
        """
        Remove a contact by name.

        Args:
            name: Contact name

        Returns:
            True if removed, False if not found
        """
        contact = self.get_contact_by_name(name)
        if contact:
            self.contacts.remove(contact)
            self.save_contacts()
            logger.info(f"Removed contact: {contact}")
            return True
        return False

    def update_contact(self, name: str, **kwargs) -> bool:
        """
        Update contact properties.

        Args:
            name: Contact name
            **kwargs: Properties to update

        Returns:
            True if updated, False if not found
        """
        contact = self.get_contact_by_name(name)
        if contact:
            for key, value in kwargs.items():
                if hasattr(contact, key):
                    setattr(contact, key, value)
            self.save_contacts()
            logger.info(f"Updated contact: {contact}")
            return True
        return False

    def get_speed_dial_address(self, number: int) -> Optional[str]:
        """
        Get SIP address for speed dial number.

        Args:
            number: Speed dial number

        Returns:
            SIP address if found, None otherwise
        """
        return self.speed_dial.get(number)

    def set_speed_dial(self, number: int, sip_address: str) -> None:
        """
        Set speed dial number.

        Args:
            number: Speed dial number
            sip_address: SIP address to assign
        """
        self.speed_dial[number] = sip_address
        self.save_contacts()
        logger.info(f"Set speed dial {number} to {sip_address}")

    def reload(self) -> None:
        """Reload all configuration from files."""
        logger.info("Reloading configuration...")
        self.load_voip_config()
        self.load_contacts()
