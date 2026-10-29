import { describe, expect, it } from 'vitest'
import { getI18nLocale, setI18nLocale } from '@/i18n'

import { API_FORMAT_STATS_COLUMNS, MODEL_STATS_COLUMNS, PROVIDER_STATS_COLUMNS, apiFormatStatsRows, modelStatsRows, providerStatsRows, providerSuccessRateClass } from '../usageStatsRows'

describe('usageStatsRows', () => {
  it('updates existing column headers when the language changes', () => {
    const previousLocale = getI18nLocale()
    try {
      setI18nLocale('zh-CN')
      expect(MODEL_STATS_COLUMNS[0].header).toBe('效率')
      expect(PROVIDER_STATS_COLUMNS.map(column => column.header)).toEqual(['成功率', '平均响应'])
      setI18nLocale('en-US')
      expect(MODEL_STATS_COLUMNS[0].header).toBe('Efficiency')
      expect(PROVIDER_STATS_COLUMNS.map(column => column.header)).toEqual(['success rate', 'Average response'])
      expect(API_FORMAT_STATS_COLUMNS[0].header).toBe('Average response')
    } finally {
      setI18nLocale(previousLocale)
    }
  })

  it('strips the claude prefix and keeps only cache reads for model rows', () => {
    const [row] = modelStatsRows([{
      model: 'claude-sonnet', request_count: 3, total_tokens: 100, effective_input_tokens: 60, output_tokens: 30,
      cache_read_tokens: 10, cache_creation_tokens: 5, cache_hit_rate: 12.5, total_cost: 1.5, actual_cost: 2, costPerToken: '$1.00/M',
    }])
    expect(row.label).toBe('sonnet')
    expect(row.inputTokens).toBe(60)
    expect(row.cacheTokens).toBe(10)
    expect(row.extra.efficiency.text).toBe('$1.00/M')
  })

  it('falls back to total input context and sums cache tokens for provider rows', () => {
    const [row] = providerStatsRows([{
      provider: 'OpenAI', providerId: 'p1', requests: 2, totalTokens: 50, totalInputContext: 40, outputTokens: 10,
      cacheReadTokens: 3, cacheCreationTokens: 4, totalCost: 0.2, successRate: 80, avgResponseTime: '1.20s',
    }])
    expect(row.key).toBe('p1')
    expect(row.inputTokens).toBe(40)
    expect(row.cacheTokens).toBe(7)
    expect(row.extra.successRate).toEqual({ text: '80%', class: 'text-destructive' })
    expect(row.extra.avgResponseTime.text).toBe('1.20s')
  })

  it('formats api format labels and marks healthy success rates without a class', () => {
    const [row] = apiFormatStatsRows([{
      api_format: 'openai:chat', request_count: 1, total_tokens: 10, output_tokens: 4, total_cost: 0.1, avgResponseTime: '-',
    }])
    expect(row.label).not.toBe('openai:chat')
    expect(row.inputTokens).toBe(0)
    expect(providerSuccessRateClass(99)).toBe('')
  })
})
