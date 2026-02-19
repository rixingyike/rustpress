---
title: 恩言 - 大字有声圣经
layout: project
version: 1.0.0
description: 一款大字有声圣经朗读应用，支持新约旧约全书、繁体中文显示、离线语音朗读，适配 macOS 与 Android 多平台。
createTime: 2026-02-19 14:00:00
icon: /assets/enyan/icon.png
screenshots:
  - /assets/enyan/screenshot1.png
  - /assets/enyan/screenshot2.png
  - /assets/enyan/screenshot3.png
  - /assets/enyan/screenshot4.png
  - /assets/enyan/screenshot5.png
  - /assets/enyan/screenshot6.png
downloads:
  - platform: macOS (Apple Silicon)
    url: https://gitee.com/rxyk/enyan/releases/download/v1.0.0/enyan-macos-arm64.app.zip
    icon: apple
    size: M1/M2/M3 芯片 Mac
  - platform: macOS (Intel)
    url: https://gitee.com/rxyk/enyan/releases/download/v1.0.0/enyan-macos-x64.app.zip
    icon: apple
    size: Intel 芯片 Mac
  - platform: Android (ARM64)
    url: https://gitee.com/rxyk/enyan/releases/download/v1.0.0/enyan-android-arm64.apk
    icon: android
    size: 主流现代安卓手机
  - platform: Android (ARMv7)
    url: https://gitee.com/rxyk/enyan/releases/download/v1.0.0/enyan-android-armv7.apk
    icon: android
    size: 老旧安卓手机
  - platform: Android (x64)
    url: https://gitee.com/rxyk/enyan/releases/download/v1.0.0/enyan-android-x64.apk
    icon: android
    size: 电脑模拟器 / Chromebook
tags: [app, flutter, rust, tts, 圣经]
comments: false
toc: false
---

## 功能特性

- 📖 **新约旧约全书** — 涵盖圣经全部 66 卷书，繁体中文和合本
- 🔤 **大字显示** — 专为阅读优化的大字排版，清晰易读
- 🔊 **有声朗读** — 内置离线语音引擎，支持逐章逐节朗读
- 🎯 **逐节跟读** — 播放时自动高亮当前朗读的经节
- 📴 **完全离线** — 全部内容与语音离线可用，无需网络
- 📱 **多平台支持** — 同时支持 macOS 和 Android 平台

## 技术架构

本应用使用 **Flutter** 作为跨平台 UI 框架，**Rust** 处理核心 TTS 引擎：

- TTS 语音引擎基于 Rust 实现，通过 FFI 桥接 Flutter
- 语音资源包支持按需下载，首次使用自动解压
- 支持 6kHz 与 8kHz 两种语音质量可选

## 资源包说明

应用内置默认内容可直接使用。在有网络的情况下，App 会自动引导下载以下可选资源包以获得更佳体验：

| 资源包 | 说明 |
|--------|------|
| 繁体中文语言包 | 繁体中文经文显示 |
| 6kHz 语音包 | 基础语音朗读（体积较小） |
| 8kHz 语音包 | 高质量语音朗读（推荐） |

> 不下载资源包也可正常使用 App 的默认内容。
