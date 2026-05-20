#!/bin/sh
set -eu

if [ -n "${AETHER_PROXY_AETHER_URL:-}" ] && [ -z "${AETHER_TUNNEL_AETHER_URL:-}" ]; then
  export AETHER_TUNNEL_AETHER_URL="${AETHER_PROXY_AETHER_URL}"
fi
if [ -n "${AETHER_PROXY_MANAGEMENT_TOKEN:-}" ] && [ -z "${AETHER_TUNNEL_MANAGEMENT_TOKEN:-}" ]; then
  export AETHER_TUNNEL_MANAGEMENT_TOKEN="${AETHER_PROXY_MANAGEMENT_TOKEN}"
fi
if [ -n "${AETHER_PROXY_NODE_NAME:-}" ] && [ -z "${AETHER_TUNNEL_NODE_NAME:-}" ]; then
  export AETHER_TUNNEL_NODE_NAME="${AETHER_PROXY_NODE_NAME}"
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL 'https://raw.githubusercontent.com/fawney19/Aether/main/apps/aether-tunnel/install.sh' | sh
elif command -v wget >/dev/null 2>&1; then
  wget -qO- 'https://raw.githubusercontent.com/fawney19/Aether/main/apps/aether-tunnel/install.sh' | sh
else
  printf '%s\n' "[Aether Tunnel] 需要 curl 或 wget 下载安装脚本" >&2
  exit 1
fi
