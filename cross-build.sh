#!/usr/bin/env bash
(cross build --message-format=json-render-diagnostics "$@" > cross.log) &&
(neon dist -n uiua_node -m /target < cross.log)
