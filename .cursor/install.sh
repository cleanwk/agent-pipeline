#!/usr/bin/env bash
# Idempotent Cloud Agent bootstrap for Agent Pipeline.
#
# Installs the Linux system libraries required to compile the Tauri crate,
# provisions Node.js 24 (via nvm) + pnpm, prepares the Rust 1.93 toolchain
# pinned by rust-toolchain.toml, and installs workspace dependencies.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

echo "==> Installing Tauri v2 Linux system libraries"
export DEBIAN_FRONTEND=noninteractive
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libxdo-dev \
  libsoup-3.0-dev \
  pkg-config \
  build-essential \
  curl \
  wget \
  file

echo "==> Provisioning Node.js 24 via nvm"
export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ ! -s "$NVM_DIR/nvm.sh" ]; then
  echo "nvm not found at $NVM_DIR; installing nvm" >&2
  curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
fi
# shellcheck disable=SC1091
. "$NVM_DIR/nvm.sh"
nvm install 24
nvm alias default 24 >/dev/null

# Make the freshly installed Node 24 win over the base image's Node.
# shellcheck disable=SC1091
. "$REPO_DIR/.cursor/activate-node.sh"
corepack enable
corepack prepare pnpm@10.29.3 --activate
echo "Using node $(node -v) / pnpm $(pnpm -v)"

echo "==> Preparing Rust toolchain (from rust-toolchain.toml)"
rustup show >/dev/null

echo "==> Installing workspace dependencies"
pnpm install --frozen-lockfile

echo "==> Pre-fetching Rust dependencies"
cargo fetch --locked

echo "==> Environment setup complete"
