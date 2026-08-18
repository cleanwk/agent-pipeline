#!/usr/bin/env bash
# Runs the Agent Pipeline desktop frontend (Vite dev server) for the Cloud Agent.
#
# The native macOS Tauri shell cannot run on the Linux Cloud Agent host, but the
# Vue frontend has a built-in browser demo fallback (see apps/desktop/src/api.ts),
# so the full Mission Control UI is browsable at http://127.0.0.1:1420.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

# shellcheck disable=SC1091
. "$REPO_DIR/.cursor/activate-node.sh"

exec pnpm --dir apps/desktop dev
