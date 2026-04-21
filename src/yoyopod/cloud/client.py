"""Compatibility shim for the relocated cloud HTTP client."""

from yoyopod.backends.cloud.http import CloudClientError, CloudDeviceClient, CloudHttpClient

__all__ = ["CloudClientError", "CloudDeviceClient", "CloudHttpClient"]
