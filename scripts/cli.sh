#!/usr/bin/env bash

export SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
export INK_DEV_IMAGE="brightinventions/disputes-ink-dev"
docker run --rm \
    -v "${SCRIPT_DIR}/../":/code \
    --network host \
    --entrypoint /bin/sh \
    "${INK_DEV_IMAGE}" \
    -c "cd /code/cli && ${1}"
