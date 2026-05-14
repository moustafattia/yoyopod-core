# DEPRECATED 2026-05-13 — superseded by Round 3 of the CLI rebuild.
#
# This Dockerfile invoked yoyopod_cli.build and scripts/build_release.py,
# both of which are deleted. CI's slot-arm64 job is disabled until Round 3
# reintroduces a Rust slot builder.
#
# Do not invoke this file. It is kept in-tree as a reference for the Round 3
# rewrite. See docs/ROADMAP.md.
#
# The historical contents are preserved below for context but commented out
# so any accidental `docker build` fails loud rather than producing a broken
# image.

# FROM python:3.12-bookworm AS build
#
# ARG CHANNEL=dev
# ARG VERSION=
#
# ENV DEBIAN_FRONTEND=noninteractive
# ENV PYTHONPATH=/src
#
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends \
#         build-essential \
#         cmake \
#         ffmpeg \
#         git \
#         libasound2-dev \
#         liblinphone-dev \
#         pkg-config && \
#     rm -rf /var/lib/apt/lists/*
#
# WORKDIR /src
# COPY . /src
#
# RUN python -m pip install --no-cache-dir --upgrade pip setuptools wheel && \
#     python -m pip install --no-cache-dir loguru typer
#
# RUN python -c "import sys; sys.path.insert(0, '/src'); from yoyopod_cli.build import app; app()" ensure-native
#
# RUN if [ -n "$VERSION" ]; then \
#         python /src/scripts/build_release.py --output /out --channel "$CHANNEL" --version "$VERSION" --with-venv --python-version 3.12; \
#     else \
#         python /src/scripts/build_release.py --output /out --channel "$CHANNEL" --with-venv --python-version 3.12; \
#     fi
#
# FROM scratch AS export
# COPY --from=build /out/ /

FROM scratch
LABEL deprecated="true"
LABEL replacement="Round 3 of CLI rebuild"
LABEL see="docs/ROADMAP.md"
