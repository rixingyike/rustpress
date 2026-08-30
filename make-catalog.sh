#!/usr/bin/env bash
# 自动检查并更新专栏 catalog 目录索引 (代理调用 rustpress 原生指令)
set -euo pipefail

exec rustpress make-catalog "$@"
