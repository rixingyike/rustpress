#!/usr/bin/env bash
# 一键创建日常博客文章 (代理调用 rustpress 原生指令)
set -euo pipefail

exec rustpress new-blog "$@"
