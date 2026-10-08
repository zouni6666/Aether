import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, ref, type App } from 'vue'
import ActivityHeatmap from '../ActivityHeatmap.vue'
import type { ActivityHeatmap as HeatmapData } from '@/types/activity'

const mounted: Array<{ app: App; root: HTMLElement }> = []
const observers: ResizeObserverStub[] = []

class ResizeObserverStub {
  target: Element | null = null
  constructor(private callback: ResizeObserverCallback) { observers.push(this) }
  observe(target: Element) { this.target = target }
  unobserve() {}
  disconnect() { this.target = null }
  resize(width: number) {
    if (!this.target) throw new Error('Heatmap has not registered its resize observer')
    this.callback([{ target: this.target, contentRect: { width } } as ResizeObserverEntry], this as unknown as ResizeObserver)
  }
}

beforeEach(() => {
  observers.length = 0
  vi.stubGlobal('ResizeObserver', ResizeObserverStub)
  vi.stubGlobal('matchMedia', vi.fn(() => ({ matches: true, addEventListener: vi.fn(), removeEventListener: vi.fn() })))
})

afterEach(() => {
  for (const { app, root } of mounted.splice(0)) { app.unmount(); root.remove() }
  vi.unstubAllGlobals()
})

function heatmap(startDate: string, length: number): HeatmapData {
  const start = new Date(`${startDate}T00:00:00Z`).getTime()
  const days = Array.from({ length }, (_, index) => ({
    date: new Date(start + index * 86400000).toISOString().slice(0, 10),
    requests: index === length - 1 ? 250 : 0,
  }))
  return { start_date: startDate, end_date: days[days.length - 1]!.date, total_days: length, max_requests: 250, days }
}

async function settle() { await nextTick(); await nextTick() }

async function mount(data: HeatmapData, compact = true) {
  const model = ref(data)
  const isCompact = ref(compact)
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(defineComponent({
    setup: () => () => h(ActivityHeatmap, { data: model.value, compact: isCompact.value, showHeader: false }),
  }))
  mounted.push({ app, root })
  app.mount(root)
  await settle()
  return { root, model, isCompact, observer: observers[observers.length - 1]! }
}

function cells(root: HTMLElement) {
  return Array.from(root.querySelectorAll<HTMLElement>('.cell-emerge'))
}

function size(root: HTMLElement) { return Number.parseFloat(cells(root)[0]!.style.width) }

describe('activity heatmap layout', () => {
  it.each([
    { compact: true, length: 1, maximum: 18 },
    { compact: false, length: 1, maximum: 16 },
    { compact: true, length: 31, maximum: 18 },
    { compact: false, length: 31, maximum: 16 },
  ])('keeps $length days within $maximum px cells (compact: $compact)', async ({ compact, length, maximum }) => {
    const { root, observer } = await mount(heatmap('2026-10-05', length), compact)
    observer.resize(1040)
    await settle()
    expect(cells(root)).toHaveLength(length)
    for (const cell of cells(root)) {
      expect(Number.parseFloat(cell.style.width)).toBeLessThanOrEqual(maximum)
      expect(Number.parseFloat(cell.style.width)).toBeGreaterThanOrEqual(6)
      expect(cell.style.height).toBe(cell.style.width)
    }
    const firstWeek = cells(root)[0]!.parentElement!.parentElement!
    expect(firstWeek.children).toHaveLength(7)
    expect(firstWeek.children[0]!.querySelector('.cell-emerge')).toBeNull()
    expect(firstWeek.children[1]!.querySelector('.cell-emerge')).toBe(cells(root)[0])
  })

  it('keeps a full year legible in a narrow container and resizes when its width or data changes', async () => {
    const { root, observer, model } = await mount(heatmap('2025-10-06', 365))
    observer.resize(280)
    await settle()
    expect(cells(root)).toHaveLength(365)
    expect(size(root)).toBe(6)
    observer.resize(640)
    await settle()
    expect(size(root)).toBeGreaterThan(6)
    expect(size(root)).toBeLessThanOrEqual(12)
    const yearCellSize = size(root)
    model.value = heatmap('2026-10-05', 1)
    await settle()
    expect(cells(root)).toHaveLength(1)
    expect(size(root)).toBeGreaterThan(yearCellSize)
    expect(size(root)).toBeLessThanOrEqual(12)
    model.value = heatmap('2025-10-06', 365)
    await settle()
    expect(size(root)).toBe(yearCellSize)
    observer.resize(1400)
    await settle()
    expect(size(root) * 53 + 52 * 2).toBeCloseTo(1400)
    expect(cells(root)[0]!.style.height).toBe(cells(root)[0]!.style.width)
  })

  it('updates the cell size and spacing when compact mode changes', async () => {
    const { root, observer, isCompact } = await mount(heatmap('2026-10-05', 1), false)
    observer.resize(1040)
    await settle()
    expect(size(root)).toBeLessThanOrEqual(16)
    expect(cells(root)[0]!.parentElement!.parentElement!.style.rowGap).toBe('4px')
    isCompact.value = true
    await settle()
    expect(size(root)).toBeLessThanOrEqual(18)
    expect(cells(root)[0]!.parentElement!.parentElement!.style.rowGap).toBe('2px')
  })

  it('preserves calendar gaps between activity dates without inventing token or cost data', async () => {
    const data = heatmap('2026-10-05', 4)
    data.days = [
      { date: '2026-10-05', requests: 250, total_tokens: 1000, total_cost: 1.25 },
      { date: '2026-10-08', requests: 650 },
    ]
    data.max_requests = 650
    const { root } = await mount(data)
    const days = cells(root)
    expect(days).toHaveLength(4)
    expect(days[0]!.title).toContain('250')
    expect(days[0]!.title).toContain('$1.25')
    expect(days[3]!.title).toContain('650')
    for (const [index, date] of ['2026-10-06', '2026-10-07'].entries()) {
      const cell = days[index + 1]!
      expect(cell.title).toContain(new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium', timeZone: 'UTC' }).format(new Date(`${date}T00:00:00Z`)))
      expect(cell.title.split(' · ')[1]).toMatch(/^0\D/)
      expect(cell.title).not.toContain('tokens')
      expect(cell.title).not.toContain('$')
    }
    const week = days[0]!.parentElement!.parentElement!
    expect(week.children[1]!.querySelector('.cell-emerge')).toBe(days[0])
    expect(week.children[4]!.querySelector('.cell-emerge')).toBe(days[3])
  })

  it.each([false, true])('renders the full declared year with sparse or empty activity (empty: %s)', async empty => {
    const data = heatmap('2025-10-06', 365)
    data.days = empty ? [] : [{ date: '2026-05-17', requests: 250 }]
    const { root, observer } = await mount(data)
    observer.resize(1040)
    await settle()
    const days = cells(root)
    expect(days).toHaveLength(365)
    expect(days[0]!.title).toContain('2025年10月6日')
    expect(days[364]!.title).toContain('2026年10月5日')
    expect(days[0]!.title.split(' · ')[1]).toMatch(/^0\D/)
    expect(days[364]!.title.split(' · ')[1]).toMatch(/^0\D/)
    expect(days.filter(day => day.title.split(' · ')[1]?.startsWith('250'))).toHaveLength(empty ? 0 : 1)
    expect(size(root) * 53 + 52 * 2).toBeCloseTo(1040)
  })
})
