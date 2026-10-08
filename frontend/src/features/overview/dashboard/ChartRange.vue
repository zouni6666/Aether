<template>
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h2 class="text-sm font-semibold">
      {{ t('统计周期', 'Statistics period') }}
    </h2>
    <div class="flex max-w-full flex-wrap items-center gap-2">
      <select
        :value="preset"
        :aria-label="t('图表时间范围', 'Chart time range')"
        class="h-8 rounded-md border bg-background px-2 text-xs"
        @change="selectPreset"
      >
        <option value="last7days">
          {{ t('最近 7 天', 'Last 7 days') }}
        </option>
        <option value="last30days">
          {{ t('最近 30 天', 'Last 30 days') }}
        </option>
        <option value="today">
          {{ t('今天', 'Today') }}
        </option>
        <option value="custom">
          {{ t('自定义', 'Custom') }}
        </option>
      </select>
      <Button
        variant="outline"
        size="icon"
        class="h-8 w-8"
        :title="t('精确时间', 'Exact time')"
        :aria-label="t('精确时间', 'Exact time')"
        :aria-expanded="editing"
        @click="editing = !editing"
      >
        <CalendarRange class="h-4 w-4" />
      </Button>
      <Button
        variant="outline"
        size="icon"
        class="h-8 w-8"
        :disabled="loading"
        :title="t('刷新图表', 'Refresh charts')"
        :aria-label="t('刷新图表', 'Refresh charts')"
        @click="$emit('refresh')"
      >
        <RefreshCw
          class="h-4 w-4"
          :class="{ 'animate-spin': loading }"
        />
      </Button>
    </div>
  </div>
  <form
    v-if="editing"
    class="flex max-w-full flex-wrap items-end gap-3 border-y py-3"
    @submit.prevent="apply"
  >
    <label class="grid w-48 max-w-full gap-1 text-xs"><span>{{ t('开始', 'From') }}</span><input
      v-model="fromInput"
      type="datetime-local"
      required
      class="h-9 w-full min-w-0 rounded-md border bg-background px-2 text-sm"
    ></label>
    <label class="grid w-48 max-w-full gap-1 text-xs"><span>{{ t('结束（不含）', 'To (exclusive)') }}</span><input
      v-model="toInput"
      type="datetime-local"
      required
      class="h-9 w-full min-w-0 rounded-md border bg-background px-2 text-sm"
    ></label>
    <label class="grid w-44 max-w-full gap-1 text-xs"><span>{{ t('报表时区', 'Report timezone') }}</span><input
      v-model="zoneInput"
      list="dashboard-timezones"
      required
      class="h-9 w-full min-w-0 rounded-md border bg-background px-2 text-sm"
    ><datalist id="dashboard-timezones"><option
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
  <p class="text-xs text-muted-foreground tabular-nums">
    {{ timestamp(range.from, range.timezone) }} - {{ timestamp(range.to, range.timezone) }} · {{ range.timezone }}
  </p>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { CalendarRange, RefreshCw } from 'lucide-vue-next'
import { Button } from '@/components/ui'
import type { OverviewRange } from '@/api/overview'
import { browserTimezone, isTimezone, presetRange, zonedInput, zonedInstant } from '../query'
import { timestamp } from '../format'
import { useOverviewI18n } from '../i18n'

const props = defineProps<{ range: OverviewRange; loading?: boolean }>()
const emit = defineEmits<{ 'update:range': [value: OverviewRange]; refresh: [] }>()
const { t } = useOverviewI18n()
const editing = ref(false)
const preset = ref('custom')
const fromInput = ref('')
const toInput = ref('')
const zoneInput = ref('UTC')
const rangeError = ref('')
const zones = [...new Set([browserTimezone(), 'UTC', 'Asia/Shanghai', 'America/New_York', 'Europe/Berlin'])]
watch(() => props.range, value => {
  fromInput.value = zonedInput(value.from, value.timezone)
  toInput.value = zonedInput(value.to, value.timezone)
  zoneInput.value = value.timezone
  const duration = Date.parse(value.to) - Date.parse(value.from)
  const recent = Math.abs(Date.now() - Date.parse(value.to)) < 120_000
  preset.value = recent && duration === 7 * 86_400_000 ? 'last7days' : recent && duration === 30 * 86_400_000 ? 'last30days'
    : recent && value.from === presetRange('today', value.timezone, new Date(value.to)).from ? 'today' : 'custom'
}, { immediate: true })
function selectPreset(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  preset.value = value
  if (value === 'custom') { editing.value = true; return }
  editing.value = false
  emit('update:range', presetRange(value, props.range.timezone))
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
  emit('update:range', { from, to, timezone: zone })
  editing.value = false
}
</script>
