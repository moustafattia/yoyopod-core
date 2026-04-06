import sys
from types import SimpleNamespace

import yoyopy.main as main_module


def test_configure_logger_uses_shared_utility(monkeypatch) -> None:
    """Keep the app entrypoint aligned with the shared logging helper."""
    fake_settings = SimpleNamespace(logging=object())
    fake_runtime = object()
    calls = []

    def fake_load_app_settings(config_dir: str) -> object:
        assert config_dir == "config"
        return fake_settings

    def fake_build_logging_runtime_config(settings, *, base_dir):
        assert settings is fake_settings.logging
        assert base_dir.name.startswith("yoyo-py")
        return fake_runtime

    def fake_init_logger(**kwargs) -> None:
        calls.append(kwargs)
        return fake_runtime

    monkeypatch.setattr(main_module, "load_app_settings", fake_load_app_settings)
    monkeypatch.setattr(
        main_module,
        "build_logging_runtime_config",
        fake_build_logging_runtime_config,
    )
    monkeypatch.setattr(main_module, "init_logger", fake_init_logger)

    runtime = main_module.configure_logger()

    assert runtime is fake_runtime
    assert calls == [
        {
            "config": fake_runtime,
            "console": True,
            "file_logging": True,
            "console_stream": sys.stderr,
            "announce": False,
        }
    ]
