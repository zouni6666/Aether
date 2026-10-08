import type { ChartData } from 'chart.js'
import type {
  ProviderPerformanceItem,
  ProviderPerformanceTimelineItem,
} from '@/api/admin'

export type ProviderPerformanceMetricKey = 'avg_output_tps' | 'avg_first_byte_time_ms'

const PROVIDER_CHART_COLORS = [
  'rgb(59, 130, 246)',
  'rgb(16, 185, 129)',
  'rgb(234, 179, 8)',
  'rgb(239, 68, 68)',
  'rgb(139, 92, 246)',
  'rgb(14, 165, 233)',
  'rgb(249, 115, 22)',
  'rgb(20, 184, 166)',
]

export function formatProviderPerformanceMetric(
  value: number | null | undefined,
  suffix = '',
  decimals = 2
): string {
  if (value == null || Number.isNaN(value)) {
    return '-'
  }
  if (suffix === 'ms') {
    return formatDurationMs(value, decimals)
  }
  return `${value.toFixed(decimals)}${suffix}`
}

export function formatDurationMs(
  value: number | null | undefined,
  msDecimals = 0,
  secondsDecimals = 2
): string {
  if (value == null || Number.isNaN(value)) {
    return '-'
  }
  if (Math.abs(value) >= 1000) {
    return `${(value / 1000).toFixed(secondsDecimals)}s`
  }
  return `${value.toFixed(msDecimals)}ms`
}

export function buildProviderPerformanceChartData(
  timeline: ProviderPerformanceTimelineItem[],
  metric: ProviderPerformanceMetricKey,
  providers: Pick<ProviderPerformanceItem, 'provider_id' | 'provider'>[] = []
): ChartData<'line'> {
  const labels = Array.from(new Set(timeline.map(item => item.date)))
  const providerMap = new Map<string, string>()
  for (const provider of providers) {
    providerMap.set(provider.provider_id, provider.provider)
  }
  for (const item of timeline) {
    if (!providerMap.has(item.provider_id)) {
      providerMap.set(item.provider_id, item.provider)
    }
  }

  return {
    labels,
    datasets: Array.from(providerMap.entries()).map(([providerId, provider], index) => {
      const byDate = new Map(
        timeline
          .filter(item => item.provider_id === providerId)
          .map(item => [item.date, item[metric]] as const)
      )
      return {
        label: provider,
        data: labels.map(label => byDate.get(label) ?? null),
        borderColor: PROVIDER_CHART_COLORS[index % PROVIDER_CHART_COLORS.length],
        tension: 0.25,
        pointRadius: 2,
      }
    }),
  }
}
