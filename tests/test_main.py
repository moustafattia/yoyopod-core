import sys

import yoyopy.main as main_module


def test_configure_logger_uses_shared_utility(monkeypatch) -> None:
    """Keep the app entrypoint aligned with the shared logging helper."""
    calls = []

    def fake_init_logger(**kwargs) -> None:
        calls.append(kwargs)

    monkeypatch.setattr(main_module, "init_logger", fake_init_logger)

    main_module.configure_logger()

    assert calls == [
        {
            "level": "INFO",
            "console": True,
            "file_logging": False,
            "console_stream": sys.stderr,
            "announce": False,
        }
    ]
