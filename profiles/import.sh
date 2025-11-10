#!/bin/sh
set -e
PROFILES_ROOT="$(dirname "$(realpath "$0")")"
cd "${PROFILES_ROOT}/.."

# Ensure the release is built

if [ ! -f ./target/release/pmrctrl ]; then
    cargo build --release --all-features
fi

if [ ! -f "${PROFILES_ROOT}/env" ]; then
    echo \'"${PROFILES_ROOT}/env"\' is missing\; >&2
    echo please copy and modify from \'"${PROFILES_ROOT}/env.example"\'. >&2
    exit 1
fi

source "${PROFILES_ROOT}/env"

run_pmrctrl () {
    ./target/release/pmrctrl --format=toml profile import "${FILE}" 2> /dev/null || return $?
}

for FILE in ${PROFILES_ROOT}/*.toml; do
    if ! run_pmrctrl ; then
        echo "${FILE} already imported"
    fi
done
