<template>
  <header class="flex flex-wrap items-start justify-between gap-4">
    <div class="min-w-0">
      <h1 class="text-lg font-semibold">
        {{ title }}
      </h1>
      <slot name="subtitle" />
    </div>
    <div class="flex max-w-full flex-wrap items-center gap-2">
      <slot />
      <slot name="range-picker">
        <TimeRangePicker
          v-if="presetsOnly"
          :model-value="pickerRange"
          :preset-options="['last1hour', 'today', 'last24hours', 'last7days', 'last30days']"
          :show-granularity="false"
          preset-only
          @update:model-value="selectSharedPreset"
        />
        <select
          v-else
          class="h-9 rounded-md border bg-background px-2 text-sm"
          :aria-label="t('时间范围', 'Time range')"
          :value="selectedPreset"
          @change="selectPreset"
        >
          <option value="custom">
            {{ t('自定义', 'Custom') }}
          </option>
          <option value="last1hour">
            {{ t('最近 1 小时', 'Last hour') }}
          </option>
          <option value="today">
            {{ t('今天', 'Today') }}
          </option>
          <option value="last24hours">
            {{ t('最近 24 小时', 'Last 24 hours') }}
          </option>
          <option value="last7days">
            {{ t('最近 7 天', 'Last 7 days') }}
          </option>
          <option value="last30days">
            {{ t('最近 30 天', 'Last 30 days') }}
          </option>
        </select>
        <Button
          v-if="!presetsOnly"
          variant="outline"
          size="icon"
          class="h-9 w-9"
          :title="t('精确时间', 'Exact time')"
          :aria-label="t('精确时间', 'Exact time')"
          :aria-expanded="editing"
          @click="editing = !editing"
        >
          <CalendarRange class="h-4 w-4" />
        </Button>
      </slot>
      <RefreshButton
        :loading="loading"
        :active="refreshActive"
        :title="refreshTitle || t('刷新', 'Refresh')"
        :aria-label="refreshTitle || t('刷新', 'Refresh')"
        @click="$emit('refresh')"
      />
    </div>
  </header>
  <form
    v-if="editing && !presetsOnly"
    class="flex flex-wrap items-end gap-3 border-y py-3"
    @submit.prevent="apply"
  >
    <label class="grid min-w-0 gap-1 text-xs"><span>{{ t('开始', 'From') }}</span><input
      v-model="fromInput"
      type="datetime-local"
      required
      class="h-9 max-w-full rounded-md border bg-background px-2 text-sm"
    ></label>
    <label class="grid min-w-0 gap-1 text-xs"><span>{{ t('结束（不含）', 'To (exclusive)') }}</span><input
      v-model="toInput"
      type="datetime-local"
      required
      class="h-9 max-w-full rounded-md border bg-background px-2 text-sm"
    ></label>
    <label class="grid min-w-0 gap-1 text-xs"><span>{{ t('报表时区', 'Report timezone') }}</span><input
      v-model="zoneInput"
      list="overview-timezones"
      required
      class="h-9 w-44 max-w-full rounded-md border bg-background px-2 text-sm"
    ><datalist id="overview-timezones"><option
      v-for="zone in zones"
      :key="zone"
      :value="zone"
    /></datalist></label>
    <Button
      type="submit"
      size="sm"
    >
      {{ t('应用', 'Apply') }}
    </Button>
    <p
      v-if="rangeError"
      role="alert"
      class="w-full text-xs text-destructive"
    >
      {{ rangeError }}
    </p>
  </form>
  <p
    v-if="showRange"
    class="text-xs text-muted-foreground tabular-nums"
  >
    {{ timestamp(range.from, range.timezone) }} - {{ timestamp(range.to, range.timezone) }} · {{ range.timezone }}
  </p>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { CalendarRange } from 'lucide-vue-next'
import { Button, RefreshButton } from '@/components/ui'
import TimeRangePicker from '@/components/common/TimeRangePicker.vue'
import type { DateRangeParams } from '@/features/usage/types'
import type { OverviewRange } from '@/api/overview'
import { browserTimezone, isTimezone, presetRange, zonedInput, zonedInstant } from '../query'
import { timestamp } from '../format'
import { useOverviewI18n } from '../i18n'

const props = withDefaults(defineProps<{
  title: string
  range: OverviewRange
  loading?: boolean
  preset?: string | null
  presetsOnly?: boolean
  showRange?: boolean
  refreshActive?: boolean
  refreshTitle?: string
}>(), { preset: undefined, showRange: true, refreshActive: undefined, refreshTitle: undefined })
const emit = defineEmits<{ 'update:range': [value: OverviewRange, preset?: string]; refresh: [] }>()
const { t } = useOverviewI18n()
const pickerRange = computed<DateRangeParams>(() => props.preset
  ? { preset: props.preset, timezone: props.range.timezone, granularity: 'day' }
  : { ...props.range, granularity: 'day' })
function selectSharedPreset(value: DateRangeParams) {
  if (value.preset) emit('update:range', presetRange(value.preset, props.range.timezone), value.preset)
}
const editing = ref(false)
const selectedPreset = ref('custom')
const fromInput = ref('')
const toInput = ref('')
const zoneInput = ref('UTC')
const rangeError = ref('')
const zones = [...new Set([browserTimezone(), 'UTC', 'Asia/Shanghai', 'America/New_York', 'Europe/Berlin'])]
function syncRangeInputs(value: OverviewRange) {
  fromInput.value = zonedInput(value.from, value.timezone)
  toInput.value = zonedInput(value.to, value.timezone)
  zoneInput.value = value.timezone
}
watch(() => props.range, value => { if (!editing.value) syncRangeInputs(value) }, { immediate: true })
watch(editing, open => { if (open) syncRangeInputs(props.range) })
watch(() => props.preset, value => { if (value !== undefined) selectedPreset.value = value || 'custom' }, { immediate: true })
function selectPreset(event: Event) {
  selectedPreset.value = (event.target as HTMLSelectElement).value
  if (selectedPreset.value === 'custom') { editing.value = true; return }
  editing.value = false
  emit('update:range', presetRange(selectedPreset.value, props.range.timezone), selectedPreset.value)
}
function apply() {
  const zone = zoneInput.value.trim()
  const from = zonedInstant(fromInput.value, zone)
  const to = zonedInstant(toInput.value, zone)
  if (!isTimezone(zone) || !from || !to || from >= to) {
    rangeError.value = t('请输入有效时区与起止时间；夏令时跳过的时间不可用。', 'Enter a valid timezone and time range. Times skipped by daylight saving are invalid.')
    return
  }
  rangeError.value = ''
  selectedPreset.value = 'custom'
  emit('update:range', { from, to, timezone: zone })
  editing.value = false
}
</script>
