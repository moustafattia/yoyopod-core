#!/usr/bin/env bash
# deploy/scripts/launch.sh
#
# Slot launcher — run from systemd via /opt/yoyopod/current/bin/launch.
# Resolves the slot dir from $0, sets env vars the app needs, exec's python.
#
# This script MUST work when invoked through the `current` symlink. We
# dereference that symlink once to get the real slot directory so our
# $SLOT_DIR variable doesn't chase the symlink mid-run.

set -euo pipefail

# Resolve the real directory this script lives in (following the current symlink).
SCRIPT_PATH="$(readlink -f "$0")"
SLOT_DIR="$(dirname "$(dirname "$SCRIPT_PATH")")"

export YOYOPOD_RELEASE_MANIFEST="${SLOT_DIR}/manifest.json"
export YOYOPOD_STATE_DIR="${YOYOPOD_STATE_DIR:-/opt/yoyopod/state}"
export PYTHONPATH="${SLOT_DIR}/app"
export PYTHONUNBUFFERED=1

# Create the state dir if missing (first boot after bootstrap).
mkdir -p "${YOYOPOD_STATE_DIR}"

PYTHON="${SLOT_DIR}/venv/bin/python"
if [ ! -x "${PYTHON}" ]; then
    echo "slot runtime missing: ${PYTHON}" >&2
    exit 1
fi

cd "${SLOT_DIR}"
# yoyopod.main exposes main() but is not itself executable as a module.
exec "${PYTHON}" -c "from yoyopod.main import main; raise SystemExit(main())" "$@"
