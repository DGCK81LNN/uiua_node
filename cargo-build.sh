#!/usr/bin/env bash
(cargo build --message-format=json-render-diagnostics "$@" > cargo.log) &&
(neon dist -n uiua_node < cargo.log)
