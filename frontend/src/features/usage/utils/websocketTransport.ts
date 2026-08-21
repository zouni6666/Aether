export interface UsageWebSocketTransportLike {
  websocket_transport?: string | null
}

function normalizeWebSocketTransport(
  record: UsageWebSocketTransportLike,
): string {
  return record.websocket_transport?.trim().toLowerCase() ?? ''
}

export function formatUsageWebSocketTransportLabel(
  record: UsageWebSocketTransportLike,
): string {
  switch (normalizeWebSocketTransport(record)) {
    case 'responses':
      return 'Responses WS'
    case 'codex_live_direct':
    case 'codex_live_sideband':
      return 'Live WS'
    case 'openai_realtime':
    case 'realtime':
      return 'Realtime WS'
    default:
      return 'WS'
  }
}

export function formatUsageWebSocketTransportTitle(
  record: UsageWebSocketTransportLike,
): string {
  const transport = normalizeWebSocketTransport(record)
  switch (transport) {
    case 'responses':
      return 'OpenAI Responses WebSocket'
    case 'codex_live_direct':
      return 'Codex Live 直连 WebSocket'
    case 'codex_live_sideband':
      return 'Codex Live Sideband WebSocket'
    case 'openai_realtime':
    case 'realtime':
      return 'OpenAI Realtime WebSocket'
    default:
      return transport ? `WebSocket transport: ${transport}` : 'WebSocket'
  }
}
