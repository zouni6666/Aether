import { describe, expect, it } from 'vitest'
import { analysisShare, analyzeRows, mergeLiveAnalysis, rankAnalysisRows, type AnalysisRow } from '../operations/analysisHelpers'

function row(id: string, requestCount: number, failedCount: number | null): AnalysisRow {
  return { id, label: id, requestCount, failedCount, successRate: null, firstByteMs: null, outputTps: null }
}

describe('automatic provider and model analysis', () => {
  it('merges observed load without deriving RPM or concurrency from historical request counts', () => {
    const history = [row('busy', 50_000, 12), row('idle', 8_000, 0)]
    const merged = mergeLiveAnalysis(history, [{ id: 'busy', label: 'busy', concurrency: 7, rpm: 83 }])
    expect(merged).toEqual([
      { ...history[0], concurrency: 7, rpm: 83 },
      { ...history[1], concurrency: 0, rpm: 0 },
    ])
    expect(history[0].concurrency).toBeUndefined()
    expect(mergeLiveAnalysis(history, null)).toEqual(history.map(item => ({ ...item, concurrency: null, rpm: null })))
    expect(mergeLiveAnalysis(history, [])).toEqual(history.map(item => ({ ...item, concurrency: 0, rpm: 0 })))
  })

  it('includes active entities before their requests are recorded without inventing historical rates', () => {
    expect(mergeLiveAnalysis([], [{ id: 'new-model', label: 'New model', concurrency: 3, rpm: 9 }])).toEqual([{
      id: 'new-model', label: 'New model', requestCount: 0, failedCount: 0,
      successRate: null, firstByteMs: null, outputTps: null, concurrency: 3, rpm: 9,
    }])
    expect(mergeLiveAnalysis(null, [])).toBeNull()
  })

  it('prioritizes the largest failure impact, then traffic, and preserves unknown failure counts', () => {
    const rows = [row('unknown', 9000, null), row('healthy', 5000, 0), row('b', 200, 8), row('a', 200, 8), row('more-traffic', 300, 8), row('most-failures', 50, 12)]
    expect(rankAnalysisRows(rows).map(item => item.id)).toEqual(['most-failures', 'more-traffic', 'a', 'b', 'healthy', 'unknown'])
    expect(rows[0].id).toBe('unknown')
  })

  it('computes each leader independently and uses the complete request and failure totals', () => {
    const result = analyzeRows([row('busy', 900, 1), row('failing', 100, 9)], 1000, 10)
    expect(result.traffic).toMatchObject({ state: 'ready', row: { id: 'busy' }, share: 90 })
    expect(result.failures).toMatchObject({ state: 'ready', row: { id: 'failing' }, share: 90 })
  })

  it('never invents a percentage for missing, empty or inconsistent denominators', () => {
    expect(analysisShare(0, 10)).toBe(0)
    expect(analysisShare(1, 4)).toBe(25)
    for (const [part, total] of [[0, 0], [1, null], [null, 10], [NaN, 10], [1, Infinity], [11, 10], [-1, 10], [1, -10]]) {
      expect(analysisShare(part, total)).toBeNull()
    }
  })

  it('distinguishes no requests, measured no failures, missing analysis and missing failures', () => {
    expect(analyzeRows([], 0, 0).traffic.state).toBe('empty')
    expect(analyzeRows([], 0, 0).failures.state).toBe('empty')
    expect(analyzeRows([row('busy', 20, 0)], 20, 0).failures.state).toBe('none')
    expect(analyzeRows(null, 20, 0).failures.state).toBe('unavailable')
    expect(analyzeRows([row('busy', 20, null)], 20, null).failures.state).toBe('unavailable')
    expect(analyzeRows([], 20, 5).traffic.state).toBe('unavailable')
  })

  it('labels an observed failure leader as partial when other rows have no failure counts', () => {
    const result = analyzeRows([row('unknown', 100, null), row('measured', 10, 3)], 110, null)
    expect(result.failures).toMatchObject({ state: 'ready', row: { id: 'measured' }, share: null, partial: true })
  })

  it('uses stable names for unidentified entities and does not drop rows beyond a top ten', () => {
    const rows = Array.from({ length: 12 }, (_, index) => ({ ...row(`model-${index}`, 10, 0), id: null }))
    const ranked = rankAnalysisRows(rows.reverse())
    expect(ranked).toHaveLength(12)
    expect(ranked[0].label).toBe('model-0')
    expect(analyzeRows(rows, 120, 0).traffic.row?.label).toBe('model-0')
  })
})
