import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const source = readFileSync(
  resolve(process.cwd(), 'src/views/admin/AdminOperationsDashboard.vue'),
  'utf8',
)

describe('AdminOperationsDashboard manual refresh', () => {
  it('only starts a refresh from the explicit refresh button', () => {
    expect(source).toContain('@click="refreshAll()"')
    expect(source).not.toContain('onMounted(')
    expect(source).not.toContain('setInterval(')
    expect(source).not.toContain('visibilitychange')

    const rangeWatcher = source
      .split('watch(timeRange, () => {')[1]
      ?.split('}, { deep: true })')[0]
    expect(rangeWatcher).toBeTruthy()
    expect(rangeWatcher).not.toContain('refreshAll(')
  })

  it('does not request the heavyweight system-status fallback', () => {
    expect(source).not.toContain('monitoringApi.getSystemStatus()')
  })

  it('forces fresh analytics and renders each result as soon as it settles', () => {
    expect(source).toContain('adminApi.getTimeSeries(params, { skipCache: true })')
    expect(source).toContain('adminApi.getPercentiles(params, { skipCache: true })')
    expect(source).toContain('}, { skipCache: true })')
    expect(source).toContain('adminApi.getErrorDistribution(params, { skipCache: true })')
    expect(source).toContain('include_timeline: false')

    const progressiveSetup = source.split('const results = await Promise.allSettled([')[0]
    expect(progressiveSetup).toContain('timeSeries.value = value')
    expect(progressiveSetup).toContain('percentiles.value = value')
    expect(progressiveSetup).toContain('providerPerformance.value = value')
    expect(progressiveSetup).toContain('errorDistribution.value = value.distribution')
    expect(progressiveSetup).toContain('gatewayMetrics.value = value')
  })

  it('reuses analytics responses instead of requesting a duplicate usage summary', () => {
    expect(source).not.toContain('usageApi.getUsageStats(')
    expect(source).not.toContain('summaryStats')
    expect(source).toContain('function seriesTokenTotal(')
    expect(source).toContain('numeric(item.input_tokens)')
    expect(source).toContain('numeric(item.output_tokens)')
    expect(source).toContain('numeric(item.cache_creation_tokens)')
    expect(source).toContain('numeric(item.cache_read_tokens)')
    expect(source).toContain('timeSeries.value.map(seriesTokenTotal)')
    expect(source).toContain('label="已分类错误"')
    expect(source).toContain('errorDistribution.value.reduce(')
  })
})
