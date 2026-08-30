#!/usr/bin/env bash
# 一键创建专栏连载文章并自动同步 catalog (代理调用 rustpress 原生指令)
set -euo pipefail

exec rustpress new-article "$@"
