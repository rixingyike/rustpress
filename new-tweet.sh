#!/usr/bin/env bash
# 一键发布微动态/闲言 (代理调用 rustpress 原生指令)
set -euo pipefail

exec rustpress new-tweet "$@"
