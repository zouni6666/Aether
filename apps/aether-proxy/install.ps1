$ErrorActionPreference = 'Stop'

if ($env:AETHER_PROXY_AETHER_URL -and -not $env:AETHER_TUNNEL_AETHER_URL) {
    $env:AETHER_TUNNEL_AETHER_URL = $env:AETHER_PROXY_AETHER_URL
}
if ($env:AETHER_PROXY_MANAGEMENT_TOKEN -and -not $env:AETHER_TUNNEL_MANAGEMENT_TOKEN) {
    $env:AETHER_TUNNEL_MANAGEMENT_TOKEN = $env:AETHER_PROXY_MANAGEMENT_TOKEN
}
if ($env:AETHER_PROXY_NODE_NAME -and -not $env:AETHER_TUNNEL_NODE_NAME) {
    $env:AETHER_TUNNEL_NODE_NAME = $env:AETHER_PROXY_NODE_NAME
}

irm 'https://raw.githubusercontent.com/fawney19/Aether/main/apps/aether-tunnel/install.ps1' | iex
