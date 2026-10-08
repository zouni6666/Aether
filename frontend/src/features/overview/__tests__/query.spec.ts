import { describe, expect, it } from 'vitest'
import { presetRange, rangeFromQuery, zonedInput, zonedInstant } from '../query'

describe('overview time range contract', () => {
  it('keeps a rolling hour precise across midnight', () => {
    expect(presetRange('last1hour', 'Asia/Shanghai', new Date('2026-09-10T16:20:42Z'))).toEqual({
      from: '2026-09-10T15:20:42.000Z', to: '2026-09-10T16:20:42.000Z', timezone: 'Asia/Shanghai',
    })
  })
  it('uses the report timezone for today rather than the browser calendar', () => {
    expect(presetRange('today', 'America/New_York', new Date('2026-03-08T16:00:00Z')).from).toBe('2026-03-08T05:00:00.000Z')
  })
  it('rejects a skipped DST time and picks the earlier repeated time', () => {
    expect(zonedInstant('2026-03-08T02:30', 'America/New_York')).toBeNull()
    expect(zonedInstant('2026-11-01T01:30', 'America/New_York')).toBe('2026-11-01T05:30:00.000Z')
    expect(zonedInput('2026-11-01T05:30:00Z', 'America/New_York')).toBe('2026-11-01T01:30')
  })
  it('normalizes offset timestamps and preserves the half-open boundary', () => {
    const fallback = presetRange('last7days')
    expect(rangeFromQuery({ from: '2026-09-11T00:00:00+08:00', to: '2026-09-11T00:01:00+08:00', timezone: 'Asia/Shanghai' }, fallback)).toEqual({
      from: '2026-09-10T16:00:00.000Z', to: '2026-09-10T16:01:00.000Z', timezone: 'Asia/Shanghai',
    })
    expect(rangeFromQuery({ from: 'invalid', to: 'invalid' }, fallback)).toBe(fallback)
  })
  it('migrates legacy calendar dates across DST and yesterday presets', () => {
    const fallback = { from: '2026-03-01T00:00:00Z', to: '2026-03-09T16:00:00Z', timezone: 'America/New_York' }
    expect(rangeFromQuery({ start_date: '2026-03-08', end_date: '2026-03-08', timezone: 'America/New_York' }, fallback)).toEqual({ from: '2026-03-08T05:00:00.000Z', to: '2026-03-09T04:00:00.000Z', timezone: 'America/New_York' })
    expect(rangeFromQuery({ preset: 'yesterday' }, fallback)).toEqual({ from: '2026-03-08T05:00:00.000Z', to: '2026-03-09T04:00:00.000Z', timezone: 'America/New_York' })
    expect(rangeFromQuery({ preset: 'last7days' }, fallback)).toEqual({ from: '2026-03-03T05:00:00.000Z', to: '2026-03-10T04:00:00.000Z', timezone: 'America/New_York' })
  })
  it('aligns generated presets to seconds for request-detail compatibility', () => {
    const range = presetRange('last1hour', 'UTC', new Date('2026-09-11T10:25:06.987Z'))
    expect(range.from).toBe('2026-09-11T09:25:06.000Z')
    expect(range.to).toBe('2026-09-11T10:25:06.000Z')
  })
})
