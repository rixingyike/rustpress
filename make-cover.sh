#!/usr/bin/env bash
# 自动生成专栏高清封面图片 (代理调用 rustpress 原生指令)
set -euo pipefail

exec rustpress make-cover "$@"
