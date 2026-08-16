# Publish notarized releases through GitHub and Homebrew

## Status

Accepted

## Context

Agent Pipeline 面向团队成员分发，只有源码仓库或本机构建不能形成可信、可重复的安装体验。用户已有公开的 `cleanwk/homebrew-tap`，其中现有 Graft 项目已经验证了 Tag、GitHub Release、DMG 和自动更新 Cask 的基本流水线，但当前可见的仓库 Secrets 只有 Tauri updater 与 Tap 写入凭据，没有 Developer ID Application 和 Apple notarization 凭据。Tauri updater 签名只验证更新包来源，不能让 macOS Gatekeeper 信任应用。

## Decision

Host App 与七阶段示例 Package 分别发布到公开仓库 `cleanwk/agent-pipeline` 和 `cleanwk/agent-pipeline-example`，均使用 Apache-2.0。Host 的版本 Tag 触发 Apple Silicon 构建，生成 Developer ID 签名且经 Apple notarization/stapling 的 DMG，上传到 immutable GitHub Release；流水线随后计算 DMG SHA-256，并更新 `cleanwk/homebrew-tap` 的 `Casks/agent-pipeline.rb`。公开安装入口固定为 `brew install --cask cleanwk/tap/agent-pipeline`。

Release Gate 必须验证版本一致性、签名、公证、Gatekeeper、Cask style/audit、全新安装、从 `/Applications/Agent Pipeline.app` 启动、升级和卸载。缺少 Developer ID 或 notarization 凭据时可以继续本机构建和产品验收，但不得把该构建标记为正式公开 Release，也不得要求用户绕过 quarantine。仓库与 Release 仅在本机纵向链路和视觉验收完成后创建并发布。

## Consequences

可以直接复用 Graft 的 GitHub Actions 与 Tap 更新结构，但需要为新仓库配置独立或组织级 Apple 签名、公证、Tauri updater 和 Tap 写入 Secrets。首发只构建 `aarch64-apple-darwin`，Cask 明确限制 Apple Silicon 和支持的最低 macOS。发布流水线成为交付的一部分，而不是开发完成后的手工附件。
