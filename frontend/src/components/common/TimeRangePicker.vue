<template>
  <div class="flex max-w-full flex-wrap items-center gap-2">
    <Select
      v-model="selectedPreset"
    >
      <SelectTrigger
        class="h-8 w-40 text-xs border-border/60"
        :class="[presetTriggerClass]"
        :aria-label="legacyT('时间范围')"
      >
        <SelectValue :placeholder="legacyT('选择时间段')">
          {{ presetOnly && selectedPreset === 'custom' ? legacyT('已选时段') : presetLabels[selectedPreset] }}
        </SelectValue>
      </SelectTrigger>
      <SelectContent :searchable="false">
        <SelectItem
          v-for="preset in activePresetOptions"
          :key="preset"
          :value="preset"
        >
          {{ presetLabels[preset] }}
        </SelectItem>
      </SelectContent>
    </Select>

    <div
      v-if="selectedPreset === 'custom' && !presetOnly"
      class="flex max-w-full flex-wrap items-center gap-2"
    >
      <Input
        v-model="startDate"
        :type="precise ? 'datetime-local' : 'date'"
        :class="precise ? 'w-48' : 'w-36'"
        class="h-8 max-w-full text-xs border-border/60"
      />
      <span class="text-xs text-muted-foreground">{{ legacyT('至') }}</span>
      <Input
        v-model="endDate"
        :type="precise ? 'datetime-local' : 'date'"
        :class="precise ? 'w-48' : 'w-36'"
        class="h-8 max-w-full text-xs border-border/60"
      />
    </div>

    <Select
      v-if="showGranularity"
      v-model="selectedGranularity"
    >
      <SelectTrigger class="h-8 w-24 text-xs border-border/60">
        <SelectValue :placeholder="legacyT('粒度')" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-if="allowHourly && canUseHourly"
          value="hour"
        >
          {{ legacyT('小时') }}
        </SelectItem>
        <SelectItem value="day">
          {{ legacyT('天') }}
        </SelectItem>
        <SelectItem value="week">
          {{ legacyT('周') }}
        </SelectItem>
        <SelectItem value="month">
          {{ legacyT('月') }}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Input
} from '@/components/ui'
import type { DateRangeParams } from '@/features/usage/types'
import { useI18n } from '@/i18n'
import { browserTimezone, zonedInput, zonedInstant } from '@/features/overview/query'

const props = withDefaults(defineProps<{
  modelValue: DateRangeParams
  showGranularity?: boolean
  allowHourly?: boolean
  presetOptions?: SelectablePreset[]
  presetTriggerClass?: string
  presetOnly?: boolean
}>(), {
  presetOptions: () => ['today', 'yesterday', 'last7days', 'last30days', 'last90days', 'custom'],
  presetTriggerClass: undefined,
  presetOnly: false,
})
const emit = defineEmits<{
  'update:modelValue': [value: DateRangeParams]
}>()
const { legacyT } = useI18n()
const selectablePresets = ['last1hour', 'today', 'yesterday', 'last24hours', 'last7days', 'last30days', 'last90days', 'custom'] as const
type SelectablePreset = typeof selectablePresets[number]

const presetLabels = computed<Record<SelectablePreset, string>>(() => ({
  last1hour: legacyT('最近 1 小时'),
  today: legacyT('今天'),
  yesterday: legacyT('昨天'),
  last24hours: legacyT('最近 24 小时'),
  last7days: legacyT('最近7天'),
  last30days: legacyT('最近30天'),
  last90days: legacyT('最近90天'),
  custom: legacyT('自定义')
}))

const activePresetOptions = computed<SelectablePreset[]>(() => {
  const unique = new Set(props.presetOptions)
  const available = selectablePresets.filter(preset => !props.presetOnly || preset !== 'custom')
  const filtered = available.filter((preset) => unique.has(preset))
  return filtered.length > 0 ? filtered : available
})

function defaultPreset(): SelectablePreset {
  const options = activePresetOptions.value
  if (options.includes('last7days')) return 'last7days'
  return options[0] ?? 'last7days'
}

function normalizePreset(value: DateRangeParams): SelectablePreset {
  if (value.preset && activePresetOptions.value.includes(value.preset as SelectablePreset)) {
    return value.preset as SelectablePreset
  }
  if (!value.preset && (value.from || value.start_date || value.end_date) && (props.presetOnly || activePresetOptions.value.includes('custom'))) {
    return 'custom'
  }
  return defaultPreset()
}

const selectedPreset = ref<SelectablePreset>(normalizePreset(props.modelValue))
const precise = computed(() => !!props.modelValue.from && !!props.modelValue.to)
const startDate = ref(props.modelValue.from ? zonedInput(props.modelValue.from, props.modelValue.timezone || browserTimezone()) : props.modelValue.start_date || '')
const endDate = ref(props.modelValue.to ? zonedInput(props.modelValue.to, props.modelValue.timezone || browserTimezone()) : props.modelValue.end_date || '')
const selectedGranularity = ref(props.modelValue.granularity || 'day')

const showGranularity = computed(() => props.showGranularity !== false)
const allowHourly = computed(() => props.allowHourly === true)

const canUseHourly = computed(() => {
  if (selectedPreset.value === 'today' || selectedPreset.value === 'yesterday') return true
  if (selectedPreset.value === 'custom' && startDate.value && endDate.value) {
    return startDate.value === endDate.value
  }
  return false
})

// 记录上次 emit 的值，避免重复触发
let lastEmittedValue: string | null = null

function buildEmitValue(): DateRangeParams {
  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone
  const tz_offset_minutes = -new Date().getTimezoneOffset()

  if (selectedPreset.value === 'custom') {
    if (precise.value) {
      const zone = props.modelValue.timezone || timezone
      if (startDate.value === zonedInput(props.modelValue.from!, zone)
        && endDate.value === zonedInput(props.modelValue.to!, zone)) {
        return { ...props.modelValue, granularity: selectedGranularity.value }
      }
      const from = zonedInstant(startDate.value, zone)
      const to = zonedInstant(endDate.value, zone)
      if (!from || !to || from >= to) return props.modelValue
      return { from, to, timezone: zone, granularity: selectedGranularity.value }
    }
    const start = startDate.value <= endDate.value ? startDate.value : endDate.value
    const end = endDate.value >= startDate.value ? endDate.value : startDate.value
    return {
      start_date: start,
      end_date: end,
      granularity: selectedGranularity.value,
      timezone,
      tz_offset_minutes
    }
  }

  return {
    preset: selectedPreset.value,
    granularity: selectedGranularity.value,
    timezone,
    tz_offset_minutes
  }
}

function getValueKey(value: DateRangeParams): string {
  if (value.from && value.to) return `precise:${value.from}:${value.to}:${value.timezone}:${value.granularity}`
  // 只比较核心字段，忽略 timezone 和 tz_offset_minutes（这些每次都会重新计算）
  if (value.preset) {
    return `preset:${value.preset}:${value.granularity}`
  }
  return `custom:${value.start_date}:${value.end_date}:${value.granularity}`
}

watch(() => props.modelValue, (value) => {
  selectedPreset.value = normalizePreset(value)
  if (value.from && value.to) {
    startDate.value = zonedInput(value.from, value.timezone || browserTimezone())
    endDate.value = zonedInput(value.to, value.timezone || browserTimezone())
  } else {
    startDate.value = startDate.value.slice(0, 10)
    endDate.value = endDate.value.slice(0, 10)
  }
  if (value.start_date !== undefined) startDate.value = value.start_date || ''
  if (value.end_date !== undefined) endDate.value = value.end_date || ''
  if (value.granularity) selectedGranularity.value = value.granularity
  // 同步更新 lastEmittedValue，避免外部设置值后触发重复 emit
  lastEmittedValue = getValueKey(value)
}, { deep: true })

watch(activePresetOptions, () => {
  if (!activePresetOptions.value.includes(selectedPreset.value)) {
    selectedPreset.value = normalizePreset(props.modelValue)
  }
})

watch([selectedPreset, startDate, endDate, selectedGranularity], () => {
  // A fixed range can be displayed without exposing a custom date editor.
  if (props.presetOnly && selectedPreset.value === 'custom') return
  if (!allowHourly.value || !canUseHourly.value) {
    if (selectedGranularity.value === 'hour') {
      selectedGranularity.value = 'day'
    }
  }

  if (selectedPreset.value === 'custom') {
    if (!startDate.value || !endDate.value) return
  }

  const newValue = buildEmitValue()
  const newKey = getValueKey(newValue)

  // 只有当值真正变化时才 emit，避免初始化时的重复触发
  if (newKey !== lastEmittedValue) {
    lastEmittedValue = newKey
    emit('update:modelValue', newValue)
  }
}, { immediate: !props.presetOnly })
</script>
