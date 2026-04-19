"""People-domain models and YAML serialization helpers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass
class Contact:
    """One mutable address-book entry used by the Talk flow."""

    name: str
    sip_address: str = ""
    phone_number: str = ""
    favorite: bool = False
    notes: str = ""
    contact_id: str = ""
    sync_origin: str = "local"
    can_call: bool = True
    can_receive: bool = True

    @property
    def display_name(self) -> str:
        """Return the kid-facing label for the contact."""

        label = self.notes.strip()
        return label or self.name

    def __str__(self) -> str:
        return f"{self.name} ({self.sip_address or self.phone_number or 'no-address'})"

    def preferred_call_target(
        self,
        *,
        gsm_enabled: bool = False,
    ) -> tuple[str | None, str]:
        """Return the active call route and address for this contact."""

        return resolve_contact_call_target(self, gsm_enabled=gsm_enabled)

    def is_callable(self, *, gsm_enabled: bool = False) -> bool:
        """Return whether this contact can be called on the active device."""

        return contact_is_callable(self, gsm_enabled=gsm_enabled)


def resolve_contact_call_target(
    contact: object,
    *,
    gsm_enabled: bool = False,
) -> tuple[str | None, str]:
    """Return the active call route for any contact-like object."""

    if not bool(getattr(contact, "can_call", True)):
        return None, ""

    sip_address = str(getattr(contact, "sip_address", "")).strip()
    if sip_address:
        return "sip", sip_address

    phone_number = str(getattr(contact, "phone_number", "")).strip()
    if gsm_enabled and phone_number:
        return "gsm", phone_number

    return None, ""


def contact_is_callable(contact: object, *, gsm_enabled: bool = False) -> bool:
    """Return whether any contact-like object is callable on the active device."""

    route, _ = resolve_contact_call_target(contact, gsm_enabled=gsm_enabled)
    return bool(route)


def contacts_from_mapping(data: dict[str, Any]) -> tuple[list[Contact], dict[int, str]]:
    """Build contacts and speed-dial data from one YAML-compatible mapping."""

    contacts = [
        Contact(
            name=contact_data.get("name", ""),
            sip_address=contact_data.get("sip_address", ""),
            phone_number=contact_data.get("phone_number", ""),
            favorite=contact_data.get("favorite", False),
            notes=contact_data.get("notes", ""),
            contact_id=contact_data.get("contact_id", ""),
            sync_origin=contact_data.get("sync_origin", "local"),
            can_call=contact_data.get("can_call", True),
            can_receive=contact_data.get("can_receive", True),
        )
        for contact_data in data.get("contacts", [])
    ]
    raw_speed_dial = data.get("speed_dial", {})
    speed_dial = {
        int(slot): str(address)
        for slot, address in raw_speed_dial.items()
        if str(slot).isdigit() and str(address).strip()
    }
    return contacts, speed_dial


def contacts_to_mapping(
    contacts: list[Contact],
    speed_dial: dict[int, str],
) -> dict[str, Any]:
    """Serialize contacts and speed dial into a YAML-friendly mapping."""

    return {
        "contacts": [_contact_to_mapping(contact) for contact in contacts],
        "speed_dial": speed_dial,
    }


def _contact_to_mapping(contact: Contact) -> dict[str, Any]:
    """Serialize one contact without emitting default-only fields."""

    mapping: dict[str, Any] = {"name": contact.name}

    if contact.sip_address:
        mapping["sip_address"] = contact.sip_address
    if contact.phone_number:
        mapping["phone_number"] = contact.phone_number
    if contact.favorite:
        mapping["favorite"] = True
    if contact.notes:
        mapping["notes"] = contact.notes
    if contact.contact_id:
        mapping["contact_id"] = contact.contact_id
    if contact.sync_origin != "local":
        mapping["sync_origin"] = contact.sync_origin
    if not contact.can_call:
        mapping["can_call"] = False
    if not contact.can_receive:
        mapping["can_receive"] = False

    return mapping
