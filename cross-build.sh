#!/usr/bin/env bash
cargo fetch &&
./vendor-uiua-assets.sh &&
(cross build --message-format=json-render-diagnostics "$@" > cross.log) &&
(neon dist -n uiua_node -m /target < cross.log)
