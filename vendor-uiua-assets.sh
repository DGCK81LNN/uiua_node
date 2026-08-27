#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
destination="$repo_root/vendor/uiua-assets"

if [[ -n "${UIUA_SOURCE:-}" ]]; then
    source_dir=$UIUA_SOURCE
else
    source_dir=$(find "${CARGO_HOME:-$HOME/.cargo}/registry/src" \
        -type d -path '*/uiua-*/src/assets' -print 2>/dev/null | sort -V | tail -n 1)
fi

if [[ -z "${source_dir:-}" || ! -d "$source_dir" ]]; then
    printf 'Could not find Uiua source assets. Set UIUA_SOURCE to the Uiua crate directory.\n' >&2
    exit 1
fi

assets=(
    Uiua386.ttf
    elevation.webp
    bad-apple.gif
    amen-break.wav
)

for asset in "${assets[@]}"; do
    if [[ ! -f "$source_dir/$asset" ]]; then
        printf 'Missing Uiua asset: %s/%s\n' "$source_dir" "$asset" >&2
        exit 1
    fi
done

mkdir -p "$destination"
cp -- "${assets[@]/#/$source_dir/}" "$destination/"
printf 'Copied Uiua assets to %s\n' "$destination"
