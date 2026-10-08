<template>
  <div class="space-y-4 w-full">
    <!-- Tooltip 使用 Teleport 确保不受父容器 overflow 影响 -->
    <Teleport to="body">
      <div
        v-if="tooltip.visible && tooltip.day"
        ref="tooltipRef"
        class="fixed z-50 w-[200px] max-w-[calc(100vw-1rem)] break-words rounded-lg border border-border/70 bg-background px-3 py-2 text-xs shadow-lg backdrop-blur pointer-events-none"
        :style="tooltipStyle"
      >
        <p class="font-medium">
          {{ formatDay(tooltip.day.date) }}
        </p>
        <p class="mt-0.5">
          {{ t('heatmap.requests', { count: tooltip.day.requests }) }}
          <template v-if="tooltip.day.total_tokens != null">
            · {{ formatTokens(tooltip.day.total_tokens) }}
          </template>
        </p>
        <p
          v-if="tooltip.day.total_cost != null"
          class="text-[11px] text-muted-foreground"
        >
          {{ t('heatmap.cost', { value: formatCurrency(tooltip.day.total_cost) }) }}
        </p>
      </div>
    </Teleport>

    <div
      v-if="showHeader"
      class="flex flex-wrap items-center justify-between gap-4"
    >
      <div class="min-w-0 break-words">
        <p class="text-sm font-semibold">
          {{ title }}
        </p>
        <p
          v-if="subtitle"
          class="text-xs text-muted-foreground"
        >
          {{ subtitle }}
        </p>
      </div>
      <div
        v-if="weekColumns.length > 0"
        class="flex items-center gap-1 text-[11px] text-muted-foreground flex-shrink-0"
      >
        <span class="flex-shrink-0">{{ t('heatmap.less') }}</span>
        <div
          v-for="(level, index) in legendLevels"
          :key="index"
          class="w-3 h-3 rounded-[3px] flex-shrink-0"
          :style="getLegendStyle(level)"
        />
        <span class="flex-shrink-0">{{ t('heatmap.more') }}</span>
      </div>
    </div>

    <div
      v-if="weekColumns.length > 0"
      ref="heatmapScroller"
      class="flex w-full gap-3 overflow-x-auto"
      :class="compact ? 'overflow-y-hidden pb-1' : ''"
    >
      <div
        class="flex flex-col text-[10px] text-muted-foreground flex-shrink-0"
        :class="compact ? 'sticky left-0 z-10 bg-card pr-2' : ''"
        :style="[verticalGapStyle, compact ? { paddingTop: '24px' } : undefined]"
      >
        <!-- Placeholder to align with month markers -->
        <div
          v-if="!compact"
          class="text-[10px] mb-3 invisible"
        >
          M
        </div>
        <span
          v-for="(weekday, index) in weekdayLabels"
          :key="index"
          :style="dayLabelStyle"
          class="flex items-center"
          :class="{ invisible: index % 2 === 0 }"
        >{{ weekday }}</span>
      </div>
      <div class="flex-1 min-w-[200px]">
        <div
          ref="heatmapWrapper"
          class="relative block w-full"
        >
          <div
            v-if="compact"
            class="relative mb-2 h-4 text-[10px] leading-4 text-muted-foreground/80"
            :style="{ width: `${gridWidth}px` }"
          >
            <span
              v-for="marker in compactMonthMarkers"
              :key="marker.weekIndex"
              class="absolute top-0 whitespace-nowrap"
              :style="{ left: `${marker.left}px` }"
            >{{ marker.label }}</span>
          </div>
          <div
            v-else
            class="flex text-[10px] text-muted-foreground/80 mb-3"
            :style="horizontalGapStyle"
          >
            <div
              v-for="(week, weekIndex) in weekColumns"
              :key="`month-${weekIndex}`"
              :style="monthCellStyle"
              class="whitespace-nowrap text-left"
              :class="compact ? 'shrink-0' : ''"
            >
              <span v-if="monthMarkers[weekIndex]">{{ monthMarkers[weekIndex] }}</span>
            </div>
          </div>
          <div
            class="flex"
            :style="horizontalGapStyle"
          >
            <div
              v-for="(week, weekIndex) in weekColumns"
              :key="weekIndex"
              class="flex flex-col"
              :class="compact ? 'shrink-0' : ''"
              :style="verticalGapStyle"
            >
              <div
                v-for="(day, dayIndex) in week"
                :key="dayIndex"
                class="relative group"
              >
                <div
                  v-if="day"
                  class="cursor-pointer cell-emerge rounded-[6px]"
                  :class="compact
                    ? 'transition-[filter] duration-150 hover:brightness-90'
                    : 'transition-all duration-200 hover:shadow-lg'"
                  :style="[cellStyle, getCellStyle(day.requests), getCellAnimationDelay(weekIndex, dayIndex)]"
                  :title="buildTooltip(day)"
                  @mouseenter="handleHover(day, $event)"
                  @mouseleave="clearHover"
                />
                <div
                  v-else
                  :style="cellStyle"
                  class="bg-transparent rounded-[6px]"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <p
      v-else
      class="text-xs text-muted-foreground"
    >
      {{ t('heatmap.empty') }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { ActivityHeatmap, ActivityHeatmapDay } from '@/types/activity'
import { formatCurrency, formatTokens } from '@/utils/format'
import { useI18n } from '@/i18n'

const props = withDefaults(defineProps<{
  data?: ActivityHeatmap | null
  title?: string
  subtitle?: string
  showHeader?: boolean
  compact?: boolean
}>(), {
  data: undefined,
  title: undefined,
  subtitle: undefined,
  showHeader: true,
  compact: false
})

const legendLevels = [0.08, 0.25, 0.45, 0.65, 0.85]
const { locale, t } = useI18n()
const weekdayLabels = computed(() => {
  const formatter = new Intl.DateTimeFormat(locale.value, { weekday: 'short', timeZone: 'UTC' })
  return Array.from({ length: 7 }, (_, day) => formatter.format(new Date(Date.UTC(2024, 0, 7 + day))))
})

function formatDay(value: string): string {
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'medium', timeZone: 'UTC' })
    .format(new Date(`${value}T00:00:00Z`))
}

type DayWithMeta = ActivityHeatmapDay & { dateObj: Date }
const heatmapScroller = ref<HTMLElement | null>(null)
const heatmapWrapper = ref<HTMLElement | null>(null)
const heatmapWidth = ref(0)
const cellSize = ref(10)
const wideScreen = ref(true)
const cellGap = computed(() => !props.compact && wideScreen.value ? 4 : 2)
const tooltipRef = ref<HTMLElement | null>(null)
const tooltip = ref<{ day: ActivityHeatmapDay | null; x: number; y: number; visible: boolean; below: boolean }>({
  day: null,
  x: 0,
  y: 0,
  visible: false,
  below: false,
})
const tooltipStyle = computed(() => ({
  top: `${tooltip.value.y}px`,
  left: `${tooltip.value.x}px`,
  transform: tooltip.value.below ? 'translate(-50%, 0)' : 'translate(-50%, -100%)',
}))

const cellStyle = computed(() => ({
  width: `${cellSize.value}px`,
  height: `${cellSize.value}px`,
}))

const dayLabelStyle = computed(() => ({
  height: `${cellSize.value}px`,
  lineHeight: `${cellSize.value}px`,
}))

const monthCellStyle = computed(() => ({
  width: `${cellSize.value}px`,
}))

const horizontalGapStyle = computed(() => ({
  gap: `${cellGap.value}px`,
}))

const verticalGapStyle = computed(() => ({
  rowGap: `${cellGap.value}px`,
}))

const weekColumns = computed(() => {
  if (!props.data) {
    return []
  }

  const dayEntries: DayWithMeta[] = [...props.data.days].sort((a, b) => a.date.localeCompare(b.date)).map(day => ({
    ...day,
    dateObj: new Date(`${day.date}T00:00:00Z`)
  }))

  const firstDay = new Date(`${props.data.start_date}T00:00:00Z`)
  const lastDay = new Date(`${props.data.end_date}T00:00:00Z`)
  if (!Number.isFinite(firstDay.getTime()) || !Number.isFinite(lastDay.getTime()) || firstDay > lastDay) {
    return []
  }
  const padding: (DayWithMeta | null)[] = []
  if (firstDay) {
    const weekday = firstDay.getUTCDay() // 周日=0, 周一=1, ..., 周六=6
    for (let i = 0; i < weekday; i++) {
      padding.push(null)
    }
  }

  // Keep every date in the declared range, including inactive dates before the
  // first request and after the latest one.
  const daysByDate = new Map(dayEntries.map(day => [day.date, day]))
  const paddedDays: (DayWithMeta | null)[] = [...padding]
  if (firstDay && lastDay) {
    const cursor = new Date(firstDay)
    while (cursor <= lastDay) {
      const date = cursor.toISOString().slice(0, 10)
      paddedDays.push(daysByDate.get(date) ?? { date, requests: 0, dateObj: new Date(cursor) })
      cursor.setUTCDate(cursor.getUTCDate() + 1)
    }
  }
  const remainder = paddedDays.length % 7
  if (remainder !== 0) {
    for (let i = remainder; i < 7; i++) {
      paddedDays.push(null)
    }
  }

  const chunked: (DayWithMeta | null)[][] = []
  for (let i = 0; i < paddedDays.length; i += 7) {
    chunked.push(paddedDays.slice(i, i + 7))
  }

  // Trim trailing empty weeks (weeks with all null cells)
  let lastIndex = chunked.length - 1
  while (lastIndex >= 0) {
    const week = chunked[lastIndex]
    const hasAnyDay = week.some(day => day !== null)
    if (hasAnyDay) {
      break
    }
    lastIndex--
  }

  return chunked.slice(0, lastIndex + 1)
})

const monthMarkers = computed(() => {
  const markers: Record<number, string> = {}
  const columns = weekColumns.value
  const formatter = new Intl.DateTimeFormat(locale.value, { month: 'short', timeZone: 'UTC' })
  let lastMonth: number | null = null

  columns.forEach((week, index) => {
    const firstValid = week.find((day): day is DayWithMeta => day !== null)
    if (!firstValid) {
      return
    }
    const month = firstValid.dateObj.getUTCMonth()
    if (month === lastMonth) {
      return
    }
    markers[index] = formatter.format(firstValid.dateObj)
    lastMonth = month
  })

  return markers
})

const gridWidth = computed(() => {
  const columns = weekColumns.value.length
  return columns * cellSize.value + Math.max(0, columns - 1) * cellGap.value
})

const compactMonthMarkers = computed(() => {
  const markers = Object.entries(monthMarkers.value).map(([index, label]) => {
    const weekIndex = Number(index)
    // At 10px, digits and Latin letters need less room than full-width month labels.
    const width = Array.from(label).reduce((total, character) => {
      return total + (character.charCodeAt(0) <= 0xff ? 6 : 10)
    }, 0)
    return {
      weekIndex,
      label,
      width,
      left: Math.max(0, Math.min(weekIndex * (cellSize.value + cellGap.value), gridWidth.value - width)),
    }
  })

  // Keep the most recent month visible when neighboring labels would overlap.
  let nextLeft = Infinity
  return markers.reverse().filter(marker => {
    if (marker.left + marker.width + 8 > nextLeft) return false
    nextLeft = marker.left
    return true
  }).reverse()
})

watch(
  [() => props.compact, heatmapScroller, heatmapWidth, gridWidth],
  async () => {
    if (!props.compact) return
    await nextTick()
    const scroller = heatmapScroller.value
    if (scroller) scroller.scrollLeft = scroller.scrollWidth
  },
  { flush: 'post' }
)

let resizeObserver: ResizeObserver | null = null
let mediaQuery: MediaQueryList | null = null
let mediaQueryHandler: ((event?: MediaQueryListEvent) => void) | null = null

const recalcCellSize = () => {
  const columnCount = weekColumns.value.length
  if (!columnCount || !heatmapWidth.value) {
    return
  }

  const totalGap = Math.max(columnCount - 1, 0) * cellGap.value
  const availableSpace = Math.max(heatmapWidth.value - totalGap, 0)
  const rawSize = availableSpace / columnCount
  // Let a complete year fill the card, while short histories never grow cells
  // beyond the size they would have in a full calendar.
  const yearCellSize = (heatmapWidth.value - 52 * cellGap.value) / 53
  const maxSize = Math.max(props.compact ? 12 : 16, yearCellSize)
  cellSize.value = Math.min(maxSize, Math.max(6, rawSize))
}

watch(
  [heatmapWidth, () => weekColumns.value.length, cellGap, () => props.compact],
  () => {
    recalcCellSize()
  },
  { immediate: true }
)

watch(
  () => heatmapWrapper.value,
  el => {
    resizeObserver?.disconnect()
    if (el && typeof ResizeObserver !== 'undefined') {
      resizeObserver = new ResizeObserver(entries => {
        if (!entries.length) {
          return
        }
        heatmapWidth.value = entries[0].contentRect.width
        recalcCellSize()
      })
      resizeObserver.observe(el)
    } else {
      heatmapWidth.value = 0
    }
  },
  { immediate: true }
)

onMounted(() => {
  if (typeof window === 'undefined') {
    return
  }
  mediaQuery = window.matchMedia('(min-width: 640px)')
  const updateGap = () => {
    wideScreen.value = mediaQuery?.matches ?? false
  }
  mediaQueryHandler = () => updateGap()
  updateGap()
  mediaQuery?.addEventListener('change', mediaQueryHandler)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  if (mediaQuery && mediaQueryHandler) {
    mediaQuery.removeEventListener('change', mediaQueryHandler)
  }
})

async function handleHover(day: ActivityHeatmapDay, event: MouseEvent) {
  const cellRect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  tooltip.value = { day, x: cellRect.left, y: cellRect.top, visible: true, below: false }
  await nextTick()
  if (!tooltip.value.visible || tooltip.value.day?.date !== day.date) return

  const tooltipWidth = tooltipRef.value?.offsetWidth || 200
  const tooltipHeight = tooltipRef.value?.offsetHeight || 72

  // Calculate horizontal position (centered on cell)
  let left = cellRect.left + cellRect.width / 2
  const minLeft = tooltipWidth / 2 + 8
  const maxLeft = window.innerWidth - tooltipWidth / 2 - 8
  left = Math.min(Math.max(left, minLeft), maxLeft)

  // Calculate vertical position
  let top = cellRect.top - 12
  let below = false

  // If tooltip would go above viewport, show it below the cell
  if (top - tooltipHeight < 0) {
    top = cellRect.bottom + 12
    below = true
  }

  tooltip.value = {
    day,
    x: left,
    y: top,
    visible: true,
    below,
  }
}

function clearHover() {
  tooltip.value.visible = false
}

function getLegendStyle(alpha: number) {
  return {
    backgroundColor: `rgba(var(--color-primary-rgb), ${alpha})`
  }
}

function getCellStyle(requests: number) {
  const max = props.data?.max_requests || 1
  if (!requests || max === 0) {
    return {
      backgroundColor: `rgba(var(--color-primary-rgb), 0.08)`
    }
  }

  const ratio = Math.min(1, requests / max)
  const minAlpha = 0.2
  const maxAlpha = 0.95
  const alpha = minAlpha + (maxAlpha - minAlpha) * ratio
  return {
    backgroundColor: `rgba(var(--color-primary-rgb), ${alpha})`
  }
}

function buildTooltip(day: ActivityHeatmapDay): string {
  const dateLabel = formatDay(day.date)
  const parts = [dateLabel, t('heatmap.requests', { count: day.requests })]
  if (day.total_tokens != null) parts.push(`${formatTokens(day.total_tokens)} tokens`)
  if (day.total_cost != null) parts.push(formatCurrency(day.total_cost))
  if (day.actual_total_cost !== undefined) {
    parts.push(t('heatmap.actualCost', { value: formatCurrency(day.actual_total_cost) }))
  }
  return parts.join(' · ')
}

function getCellAnimationDelay(weekIndex: number, dayIndex: number) {
  const delay = Math.min(120, weekIndex * 2 + dayIndex * 4)
  return {
    animationDelay: `${delay}ms`
  }
}
</script>

<style scoped>
.cell-emerge {
  opacity: 0;
  animation: cellEmerge 0.18s ease-out forwards;
}

@keyframes cellEmerge {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}
</style>
