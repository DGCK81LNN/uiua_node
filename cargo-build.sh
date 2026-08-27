#!/usr/bin/env bash
cargo fetch &&
./vendor-uiua-assets.sh &&
(cargo build --message-format=json-render-diagnostics "$@" > cargo.log) &&
(neon dist -n uiua_node < cargo.log)
