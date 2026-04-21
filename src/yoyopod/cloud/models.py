"""Compatibility shim for the relocated cloud models."""

from yoyopod.integrations.cloud.models import CloudAccessToken, CloudStatusSnapshot

__all__ = ["CloudAccessToken", "CloudStatusSnapshot"]
