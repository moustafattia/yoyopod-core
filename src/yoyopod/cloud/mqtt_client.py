"""Compatibility shim for the relocated cloud MQTT client."""

from yoyopod.backends.cloud.mqtt import DeviceMqttClient

__all__ = ["DeviceMqttClient"]
