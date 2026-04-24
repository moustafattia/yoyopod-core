#!/usr/bin/env bash

set -euo pipefail

CHANNEL="dev"
DEST_DIR="dist"
VERSION=""
ARTIFACT_NAME=""
CACHE_ARGS=()

for arg in "$@"; do
    case "$arg" in
        --channel=*) CHANNEL="${arg#--channel=}" ;;
        --dest=*) DEST_DIR="${arg#--dest=}" ;;
        --version=*) VERSION="${arg#--version=}" ;;
        --artifact-name=*) ARTIFACT_NAME="${arg#--artifact-name=}" ;;
        --cache-from=*) CACHE_ARGS+=(--cache-from "${arg#--cache-from=}") ;;
        --cache-to=*) CACHE_ARGS+=(--cache-to "${arg#--cache-to=}") ;;
        --help|-h)
            cat <<'EOF'
Usage: build_slot_artifact_ci.sh --version=<slot-version>
                                [--channel=dev|beta|stable]
                                [--dest=dist]
                                [--artifact-name=yoyopod-slot-...tar.gz]
                                [--cache-from=<buildx-cache>]
                                [--cache-to=<buildx-cache>]
EOF
            exit 0
            ;;
        *)
            echo "build-slot-artifact-ci: unknown arg: $arg" >&2
            exit 2
            ;;
    esac
done

if [ -z "${VERSION}" ]; then
    echo "build-slot-artifact-ci: --version is required" >&2
    exit 2
fi

if [ -z "${ARTIFACT_NAME}" ]; then
    ARTIFACT_NAME="yoyopod-slot-${VERSION}-linux-arm64.tar.gz"
fi

RAW_OUT="$(mktemp -d)"
trap 'rm -rf "${RAW_OUT}"' EXIT

mkdir -p "${DEST_DIR}"

docker buildx build \
    --platform linux/arm64 \
    --build-arg "CHANNEL=${CHANNEL}" \
    --build-arg "VERSION=${VERSION}" \
    "${CACHE_ARGS[@]}" \
    --output "type=local,dest=${RAW_OUT}" \
    -f deploy/docker/slot-builder.Dockerfile \
    .

SOURCE_ARTIFACT="$(find "${RAW_OUT}" -maxdepth 1 -type f -name '*.tar.gz' | head -n 1)"
if [ -z "${SOURCE_ARTIFACT}" ]; then
    echo "build-slot-artifact-ci: no slot tarball was produced" >&2
    exit 1
fi

TARGET_ARTIFACT="${DEST_DIR}/${ARTIFACT_NAME}"
cp "${SOURCE_ARTIFACT}" "${TARGET_ARTIFACT}"

python3 - "${TARGET_ARTIFACT}" "${TARGET_ARTIFACT}.sha256" <<'PY'
import hashlib
import sys
from pathlib import Path

artifact = Path(sys.argv[1])
output = Path(sys.argv[2])
digest = hashlib.sha256()
with artifact.open("rb") as handle:
    for chunk in iter(lambda: handle.read(1024 * 1024), b""):
        digest.update(chunk)
output.write_text(f"{digest.hexdigest()}  {artifact.name}\n", encoding="utf-8")
PY

echo "${TARGET_ARTIFACT}"
