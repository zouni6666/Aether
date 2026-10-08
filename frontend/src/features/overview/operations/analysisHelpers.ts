export interface AnalysisRow {
  id: string | null
  label: string
  requestCount: number
  failedCount: number | null
  /** Percentage points, from 0 to 100. */
  successRate: number | null
  firstByteMs: number | null
  outputTps: number | null
  concurrency?: number | null
  rpm?: number | null
}

export interface LiveAnalysisRow {
  id: string | null
  label: string
  concurrency: number
  rpm: number
}

/** A complete node snapshot makes absent groups a measured zero. An unavailable
 * snapshot must never turn those same groups into zero load. Include active
 * groups before their usage rows have reached the historical report. */
export function mergeLiveAnalysis(
  historical: readonly AnalysisRow[] | null,
  live: readonly LiveAnalysisRow[] | null,
): AnalysisRow[] | null {
  if (historical === null) return null
  const current = new Map(live?.map(row => [row.id, row]))
  const result = historical.map(row => {
    const activity = current.get(row.id)
    current.delete(row.id)
    return { ...row, concurrency: activity?.concurrency ?? (live === null ? null : 0), rpm: activity?.rpm ?? (live === null ? null : 0) }
  })
  for (const row of current.values()) {
    result.push({
      id: row.id, label: row.label, requestCount: 0, failedCount: 0,
      successRate: null, firstByteMs: null, outputTps: null,
      concurrency: row.concurrency, rpm: row.rpm,
    })
  }
  return result
}

export function measuredCount(value: number | null | undefined): number | null {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null
}

/** An unavailable or inconsistent denominator must not turn into a 0% share. */
export function analysisShare(value: number | null, total: number | null): number | null {
  const part = measuredCount(value)
  const denominator = measuredCount(total)
  if (part === null || denominator === null || denominator === 0 || part > denominator) return null
  return part / denominator * 100
}

function stableOrder(left: AnalysisRow, right: AnalysisRow): number {
  const leftKey = `${left.id ?? ''}\u0000${left.label}`
  const rightKey = `${right.id ?? ''}\u0000${right.label}`
  return leftKey < rightKey ? -1 : leftKey > rightKey ? 1 : 0
}

/** Known failures first, then traffic; unavailable failures follow measured zero. */
export function rankAnalysisRows(rows: readonly AnalysisRow[]): AnalysisRow[] {
  return [...rows].sort((left, right) => {
    const failures = (measuredCount(right.failedCount) ?? -1) - (measuredCount(left.failedCount) ?? -1)
    const traffic = (measuredCount(right.requestCount) ?? -1) - (measuredCount(left.requestCount) ?? -1)
    return failures || traffic || stableOrder(left, right)
  })
}

export interface AnalysisInsight {
  state: 'ready' | 'unavailable' | 'empty' | 'none'
  row: AnalysisRow | null
  share: number | null
  partial: boolean
}

export function analyzeRows(rows: readonly AnalysisRow[] | null, totalRequests: number | null, totalFailures: number | null): {
  traffic: AnalysisInsight
  failures: AnalysisInsight
} {
  const unavailable: AnalysisInsight = { state: 'unavailable', row: null, share: null, partial: false }
  if (rows === null) return { traffic: { ...unavailable }, failures: { ...unavailable } }
  const empty: AnalysisInsight = { state: 'empty', row: null, share: null, partial: false }
  if (!rows.length && totalRequests === 0) return { traffic: { ...empty }, failures: { ...empty } }

  const trafficRow = [...rows].sort((left, right) =>
    (measuredCount(right.requestCount) ?? -1) - (measuredCount(left.requestCount) ?? -1) || stableOrder(left, right),
  ).find(row => (measuredCount(row.requestCount) ?? 0) > 0)
  const failedRow = rankAnalysisRows(rows).find(row => (measuredCount(row.failedCount) ?? 0) > 0)
  const traffic: AnalysisInsight = trafficRow
    ? { state: 'ready', row: trafficRow, share: analysisShare(trafficRow.requestCount, totalRequests), partial: rows.some(row => measuredCount(row.requestCount) === null) }
    : totalRequests === 0 ? { ...empty } : { ...unavailable }
  const failures: AnalysisInsight = failedRow
    ? { state: 'ready', row: failedRow, share: analysisShare(failedRow.failedCount, totalFailures), partial: rows.some(row => measuredCount(row.failedCount) === null) }
    : totalFailures === 0
      ? { state: 'none', row: null, share: null, partial: false }
      : { ...unavailable }
  return { traffic, failures }
}
