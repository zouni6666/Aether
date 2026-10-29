import type { ProxyNode, ProxyNodeEvent } from '@/api/proxy-nodes'
import { translate, type Locale } from '@/i18n'
import { translateMessage } from '@/i18n/messages'
import { formatCompactNumber } from '@/utils/format'
import { formatRegion } from '@/utils/region'

export type BadgeVariant = 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning' | 'dark'

export function formatProxyNodeNumber(value: number): string {
  return formatCompactNumber(value, { fractionDigits: 1 })
}

export function formatProxyNodeRegion(region: string | null): string {
  return formatRegion(region)
}

export function formatProxyNodeTime(iso: string | null, locale: Locale): string {
  if (!iso) return '-'
  const date = new Date(iso)
  const diff = (Date.now() - date.getTime()) / 1000

  if (Number.isNaN(date.getTime())) return '-'
  if (diff >= 0 && diff < 60) return translateMessage('feedback.proxy.justNow', locale)
  if (diff >= 0 && diff < 3600) return translateMessage('feedback.proxy.minutesAgo', locale, { count: Math.floor(diff / 60) })
  if (diff >= 0 && diff < 86400) return translateMessage('feedback.proxy.hoursAgo', locale, { count: Math.floor(diff / 3600) })
  return date.toLocaleDateString(locale, { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

export function proxyNodeFailureRate(node: ProxyNode): number {
  if (!node.total_requests) return 0
  const failed = (node.failed_requests || 0) + (node.dns_failures || 0) + (node.stream_errors || 0)
  return (failed / node.total_requests) * 100
}

export function formatProxyNodeFailureRate(node: ProxyNode): string {
  if (!node.total_requests) return '-'
  const rate = proxyNodeFailureRate(node)
  if (rate === 0) return '0%'
  if (rate < 0.1) return '<0.1%'
  return `${rate.toFixed(1)}%`
}

export function proxyNodeAddress(node: ProxyNode): string {
  if (node.is_manual) return node.proxy_url || `${node.ip}:${node.port}`
  if (node.tunnel_mode) return node.ip || 'WebSocket Tunnel'
  return `${node.ip}:${node.port}`
}

export function proxyNodeVersion(node: ProxyNode): string {
  const metadata = node.proxy_metadata
  if (!metadata || typeof metadata !== 'object') return '-'
  const version = (metadata as Record<string, unknown>).version
  if (typeof version !== 'string') return '-'
  const normalized = version.trim()
  return normalized || '-'
}

export function proxyNodeSchedulingBadge(node: ProxyNode): { label: string; variant: BadgeVariant } | null {
  switch (node.remote_config?.scheduling_state) {
    case 'draining':
      return { label: translate('ui.draining'), variant: 'warning' }
    case 'cordoned':
      return { label: translate('ui.cordoned'), variant: 'dark' }
    default:
      return null
  }
}

export function proxyNodeStatusVariant(status: string): BadgeVariant {
  switch (status) {
    case 'online': return 'success'
    case 'offline': return 'destructive'
    default: return 'secondary'
  }
}

export function proxyNodeStatusLabel(node: ProxyNode): string {
  if (node.tunnel_mode && !node.is_manual) {
    switch (node.status) {
      case 'online': return translate('ui.tunnelOnline')
      case 'offline': return translate('ui.tunnelOffline')
      default: return node.status
    }
  }

  switch (node.status) {
    case 'online': return translate('vscodex.devices.online')
    case 'offline': return translate('vscodex.devices.offline')
    default: return node.status
  }
}

export function proxyNodeStatusTitle(node: ProxyNode): string {
  if (node.tunnel_mode && !node.is_manual) {
    if (node.status === 'online') {
      return translate('ui.theGatewayCanStillSeeTheTunnelHeartbeatThisDoes')
    }
    return translate('ui.theGatewayCurrentlyDoesNotDetectAnAvailableTunnelConnection')
  }

  switch (node.status) {
    case 'online': return translate('ui.theNodeIsCurrentlyMarkedOnline')
    case 'offline': return translate('ui.theNodeIsCurrentlyMarkedOffline')
    default: return node.status
  }
}

export function proxyNodeEventTypeLabel(type: string): string {
  switch (type) {
    case 'connected': return translate('ui.connected')
    case 'disconnected': return translate('ui.disconnected')
    case 'error': return translate('ui.error')
    default: return type
  }
}

export function proxyNodeEventTypeVariant(type: string): BadgeVariant {
  switch (type) {
    case 'connected': return 'success'
    case 'disconnected': return 'destructive'
    case 'error': return 'destructive'
    default: return 'secondary'
  }
}

export function proxyNodeEventDetail(event: ProxyNodeEvent): string {
  return event.detail || '-'
}
