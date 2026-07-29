import { describe, expect, it } from 'vitest'

import {
  getCodexPrimaryQuotaWindow,
  getCodexQuotaWindowLimitLabel,
  getCodexQuotaWindowPresentation,
} from '../codexQuotaWindow'

describe('getCodexQuotaWindowPresentation', () => {
  it.each([
    [300, '5H'],
    [10_080, '周'],
    [43_200, '月'],
    [43_800, '月'],
    [44_640, '月'],
  ])('labels a %i-minute window as %s', (windowMinutes, expectedLabel) => {
    expect(getCodexQuotaWindowPresentation({
      code: 'primary',
      window_minutes: windowMinutes,
    })?.label).toBe(expectedLabel)
  })

  it('supports simultaneous 5H and weekly windows', () => {
    const windows = [
      getCodexQuotaWindowPresentation({ code: 'secondary', window_minutes: 10_080 }),
      getCodexQuotaWindowPresentation({ code: 'primary', window_minutes: 300 }),
    ].filter((item): item is NonNullable<typeof item> => item != null)

    expect(windows.sort((a, b) => a.sortOrder - b.sortOrder).map(item => item.label)).toEqual(['5H', '周'])
  })

  it('builds the provider limit label from the actual window duration', () => {
    expect(getCodexQuotaWindowLimitLabel({ code: 'weekly', window_minutes: 10_080 })).toBe('周限额')
    expect(getCodexQuotaWindowLimitLabel({ code: 'weekly', window_minutes: 43_800 })).toBe('月限额')
  })

  it('selects a monthly primary window over a zero-minute weekly placeholder', () => {
    const monthly = { code: 'monthly', label: '月', window_minutes: 43_800, used_ratio: 0.02 }
    const selected = getCodexPrimaryQuotaWindow([
      monthly,
      { code: 'weekly', label: '周', window_minutes: 0, used_ratio: 1 },
    ])

    expect(selected).toEqual(monthly)
    expect(getCodexQuotaWindowLimitLabel(selected!)).toBe('月限额')
  })

  it('drops zero-minute placeholder windows', () => {
    expect(getCodexQuotaWindowPresentation({
      code: 'weekly',
      label: '周',
      window_minutes: 0,
    })).toBeNull()
  })

  it('keeps legacy labels when old snapshots have no window duration', () => {
    expect(getCodexQuotaWindowPresentation({ code: '5h' })?.label).toBe('5H')
    expect(getCodexQuotaWindowPresentation({ code: 'weekly' })?.label).toBe('周')
  })
})
