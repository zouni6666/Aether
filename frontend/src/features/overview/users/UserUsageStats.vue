<template>
  <div
    class="space-y-6"
    data-user-usage-stats
  >
    <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
      <div>
        <h2 class="text-sm font-semibold">
          {{ t('userStats.title') }}
        </h2>
        <p class="text-xs text-muted-foreground">
          {{ t('userStats.description') }}
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <Select v-model="scope">
          <SelectTrigger class="h-8 w-32 text-xs">
            <SelectValue :placeholder="t('userStats.scope.placeholder')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="user">
              {{ t('userStats.scope.user') }}
            </SelectItem>
            <SelectItem value="user_group">
              {{ t('userStats.scope.userGroup') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="selectedEntityId">
          <SelectTrigger class="h-8 w-52 text-xs">
            <SelectValue :placeholder="scope === 'user' ? t('userStats.select.user') : t('userStats.select.userGroup')" />
          </SelectTrigger>
          <SelectContent
            :search-threshold="0"
            :search-placeholder="scope === 'user' ? t('userStats.search.user') : t('userStats.search.userGroup')"
          >
            <SelectItem
              v-for="entity in allEntities"
              :key="entity.id"
              :value="entity.id"
            >
              {{ entity.name }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="compareEntityId">
          <SelectTrigger class="h-8 w-52 text-xs">
            <SelectValue :placeholder="t('userStats.compare.placeholder')" />
          </SelectTrigger>
          <SelectContent
            :search-threshold="0"
            :search-placeholder="scope === 'user' ? t('userStats.search.user') : t('userStats.search.userGroup')"
          >
            <SelectItem value="__none__">
              {{ t('userStats.compare.none') }}
            </SelectItem>
            <SelectItem
              v-for="entity in comparisonEntities"
              :key="`compare-${entity.id}`"
              :value="entity.id"
            >
              {{ entity.name }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="granularity">
          <SelectTrigger
            class="h-8 w-24 text-xs"
            :aria-label="legacyT('粒度')"
          >
            <SelectValue :placeholder="legacyT('粒度')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-if="canUseHourly"
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
    </div>

    <OverviewStatus
      :error="entitiesError || leaderboardError || panelsError"
      @retry="reload"
    />

    <slot
      v-if="scope === 'user'"
      name="user-leaderboard"
      :select-user="selectUser"
    />
    <div
      v-else
      class="grid grid-cols-1 gap-4"
    >
      <LeaderboardTable
        :title="t('userStats.leaderboard.userGroup')"
        :items="leaderboard"
        :metric="metric"
        :loading="leaderboardLoading"
        show-member-count
        selectable
        @update:metric="metric = $event"
        @select="selectLeaderboardItem"
      >
        <template #pagination>
          <div class="flex items-center justify-between border-t px-4 py-3 text-xs text-muted-foreground">
            <span>{{ t('userStats.pagination.summary', { total: leaderboardTotal, page: currentPage }) }}</span>
            <div class="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                :disabled="leaderboardOffset === 0 || leaderboardLoading"
                @click="changeLeaderboardPage(-1)"
              >
                {{ t('userStats.pagination.previous') }}
              </Button>
              <Button
                variant="outline"
                size="sm"
                :disabled="!hasNextLeaderboardPage || leaderboardLoading"
                @click="changeLeaderboardPage(1)"
              >
                {{ t('userStats.pagination.next') }}
              </Button>
            </div>
          </div>
        </template>
      </LeaderboardTable>
    </div>

    <LeaderboardTable
      v-if="scope === 'user_group'"
      :title="t('userStats.memberLeaderboard')"
      :items="memberLeaderboard"
      :metric="metric"
      :loading="memberLeaderboardLoading"
      :show-metric-select="false"
      selectable
      @select="selectMember"
    />

    <Card class="space-y-4 p-4">
      <div>
        <h3
          ref="trendHeading"
          class="text-sm font-semibold scroll-mt-4"
        >
          {{ scope === 'user' ? t('userStats.trend.user') : t('userStats.trend.userGroup') }}
        </h3>
        <p class="mt-0.5 truncate text-xs text-muted-foreground">
          {{ selectedEntityName || t('userStats.selectPrompt') }}
        </p>
      </div>
      <div
        v-if="seriesLoading"
        class="p-6"
      >
        <LoadingState />
      </div>
      <div
        v-else
        class="h-[280px]"
      >
        <LineChart :data="seriesChartData" />
      </div>
    </Card>

    <Card
      v-if="comparisonSeries.length > 0"
      class="space-y-4 p-4"
    >
      <h3 class="text-sm font-semibold">
        {{ scope === 'user' ? t('userStats.comparisonTrend.user') : t('userStats.comparisonTrend.userGroup') }}
      </h3>
      <div class="h-[280px]">
        <LineChart :data="comparisonChartData" />
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import {
  Button,
  Card,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui'
import LineChart from '@/components/charts/LineChart.vue'
import { LoadingState } from '@/components/common'
import { LeaderboardTable } from '@/components/stats'
import { adminApi, type LeaderboardItem } from '@/api/admin'
import { usersApi, type User, type UserGroup } from '@/api/users'
import { useI18n } from '@/i18n'
import type { OverviewRange } from '@/api/overview'
import OverviewStatus from '../components/OverviewStatus.vue'

type StatsScope = 'user' | 'user_group'
type SelectableEntity = { id: string; name: string }
type SelectedUser = { user_id: string; username: string }

interface TimeSeriesItem {
  date: string
  total_cost: number
}

const props = defineProps<{ range: OverviewRange; revision: number }>()
defineSlots<{
  'user-leaderboard'(props: { selectUser: (user: SelectedUser) => void }): unknown
}>()
const { t, legacyT } = useI18n()

const PAGE_SIZE = 10
const granularity = ref<'hour' | 'day' | 'week' | 'month'>('day')
const canUseHourly = computed(() => Date.parse(props.range.to) - Date.parse(props.range.from) <= 48 * 3_600_000)
const metric = ref<'requests' | 'tokens' | 'cost'>('requests')
const scope = ref<StatsScope>('user')

const users = ref<User[]>([])
const selectedUserNames = ref(new Map<string, string>())
const userGroups = ref<UserGroup[]>([])
const trendHeading = ref<HTMLElement | null>(null)
const selectedUserId = ref('')
const selectedUserGroupId = ref('')
const compareUserId = ref('__none__')
const compareUserGroupId = ref('__none__')

const leaderboard = ref<LeaderboardItem[]>([])
const leaderboardTotal = ref(0)
const leaderboardOffset = ref(0)
const leaderboardLoading = ref(false)
const memberLeaderboard = ref<LeaderboardItem[]>([])
const memberLeaderboardLoading = ref(false)
const series = ref<TimeSeriesItem[]>([])
const comparisonSeries = ref<TimeSeriesItem[]>([])
const seriesLoading = ref(false)
const entitiesError = ref<string | null>(null)
const leaderboardError = ref<string | null>(null)
const panelsError = ref<string | null>(null)

let leaderboardRequestId = 0
let panelRequestId = 0
let leaderboardDebounceTimer: ReturnType<typeof setTimeout> | null = null
let panelDebounceTimer: ReturnType<typeof setTimeout> | null = null
let ready = false
let disposed = false

const allEntities = computed<SelectableEntity[]>(() => {
  if (scope.value === 'user_group') {
    return [...userGroups.value.map(group => ({ id: group.id, name: group.name })),
      { id: '__ungrouped__', name: t('userStats.ungrouped') }]
  }
  const entities = new Map(users.value.map(user => [user.id, {
    id: user.id, name: user.username || user.email || user.id,
  }]))
  selectedUserNames.value.forEach((name, id) => entities.set(id, { id, name }))
  return [...entities.values()]
})

const selectedEntityId = computed({
  get: () => scope.value === 'user' ? selectedUserId.value : selectedUserGroupId.value,
  set: (value: string) => {
    if (scope.value === 'user') selectedUserId.value = value
    else selectedUserGroupId.value = value
  }
})

const compareEntityId = computed({
  get: () => scope.value === 'user' ? compareUserId.value : compareUserGroupId.value,
  set: (value: string) => {
    if (scope.value === 'user') compareUserId.value = value
    else compareUserGroupId.value = value
  }
})

const comparisonEntities = computed(() => allEntities.value.filter(
  entity => entity.id !== selectedEntityId.value
))
const selectedEntityName = computed(() => allEntities.value.find(
  entity => entity.id === selectedEntityId.value
)?.name ?? '')
const comparedEntityName = computed(() => allEntities.value.find(
  entity => entity.id === compareEntityId.value
)?.name ?? '')
const currentPage = computed(() => Math.floor(leaderboardOffset.value / PAGE_SIZE) + 1)
const hasNextLeaderboardPage = computed(
  () => leaderboardOffset.value + leaderboard.value.length < leaderboardTotal.value
)

function buildTimeRangeParams() {
  return {
    ...props.range,
    granularity: canUseHourly.value || granularity.value !== 'hour' ? granularity.value : 'day' as const,
  }
}

function scopeParams(id: string) {
  return scope.value === 'user' ? { user_id: id } : { user_group_id: id }
}

function ensureSelectedEntity() {
  const entities = allEntities.value
  if (!entities.some(entity => entity.id === selectedEntityId.value)) {
    selectedEntityId.value = entities[0]?.id ?? ''
  }
  if (compareEntityId.value !== '__none__' && !entities.some(entity => entity.id === compareEntityId.value)) {
    compareEntityId.value = '__none__'
  }
}

async function loadEntities() {
  entitiesError.value = null
  try {
    const [loadedUsers, groupsResponse] = await Promise.all([
      usersApi.getAllUsers(),
      usersApi.listUserGroups()
    ])
    if (disposed) return
    users.value = loadedUsers
    userGroups.value = groupsResponse.items
    ensureSelectedEntity()
  } catch (error) {
    if (!disposed) entitiesError.value = error instanceof Error ? error.message : String(error)
  }
}

async function loadLeaderboard() {
  const requestId = ++leaderboardRequestId
  if (scope.value !== 'user_group') {
    leaderboard.value = []
    leaderboardTotal.value = 0
    leaderboardLoading.value = false
    leaderboardError.value = null
    return
  }
  leaderboardLoading.value = true
  leaderboardError.value = null
  try {
    const params = {
      ...buildTimeRangeParams(),
      metric: metric.value,
      limit: PAGE_SIZE,
      offset: leaderboardOffset.value
    }
    const response = await adminApi.getLeaderboardUserGroups(params, { skipCache: true })
    if (requestId !== leaderboardRequestId) return
    leaderboard.value = response.items
    leaderboardTotal.value = response.total
  } catch (error) {
    if (requestId === leaderboardRequestId) leaderboardError.value = error instanceof Error ? error.message : String(error)
  } finally {
    if (requestId === leaderboardRequestId) leaderboardLoading.value = false
  }
}

async function loadPanels() {
  const selectedId = selectedEntityId.value
  const requestId = ++panelRequestId
  panelsError.value = null
  if (!selectedId) {
    series.value = []
    comparisonSeries.value = []
    memberLeaderboard.value = []
    seriesLoading.value = false
    memberLeaderboardLoading.value = false
    return
  }
  seriesLoading.value = true
  memberLeaderboardLoading.value = scope.value === 'user_group'
  try {
    const primaryParams = { ...buildTimeRangeParams(), ...scopeParams(selectedId) }
    const shouldCompare = compareEntityId.value !== '__none__'
    const comparisonPromise: Promise<TimeSeriesItem[]> = shouldCompare
      ? adminApi.getTimeSeries({
        ...buildTimeRangeParams(),
        ...scopeParams(compareEntityId.value)
      }, { skipCache: true })
      : Promise.resolve([])
    const memberPromise: Promise<{ items: LeaderboardItem[] }> = scope.value === 'user_group'
      ? adminApi.getLeaderboardUsers({
        ...buildTimeRangeParams(),
        metric: metric.value,
        user_group_id: selectedId,
        limit: PAGE_SIZE
      }, { skipCache: true })
      : Promise.resolve({ items: [] })
    const [primarySeries, compareSeries, members] = await Promise.all([
      adminApi.getTimeSeries(primaryParams, { skipCache: true }),
      comparisonPromise,
      memberPromise,
    ])
    if (requestId !== panelRequestId) return
    series.value = primarySeries
    comparisonSeries.value = compareSeries
    memberLeaderboard.value = members.items
  } catch (error) {
    if (requestId === panelRequestId) panelsError.value = error instanceof Error ? error.message : String(error)
  } finally {
    if (requestId === panelRequestId) {
      seriesLoading.value = false
      memberLeaderboardLoading.value = false
    }
  }
}

function selectLeaderboardItem(item: LeaderboardItem) {
  selectedEntityId.value = item.id
}

function selectMember(item: LeaderboardItem) {
  selectUser({ user_id: item.id, username: item.name })
}

function selectUser(user: SelectedUser) {
  selectedUserNames.value.set(user.user_id, user.username || user.user_id)
  scope.value = 'user'
  selectedUserId.value = user.user_id
  void nextTick(() => trendHeading.value?.scrollIntoView?.({ behavior: 'smooth', block: 'start' }))
}

function changeLeaderboardPage(direction: -1 | 1) {
  leaderboardOffset.value = Math.max(0, leaderboardOffset.value + direction * PAGE_SIZE)
  void loadLeaderboard()
}

const seriesChartData = computed(() => ({
  labels: series.value.map(item => item.date),
  datasets: [{
    label: t('stats.metric.cost'),
    data: series.value.map(item => item.total_cost),
    borderColor: 'rgb(59, 130, 246)',
    tension: 0.25,
    pointRadius: 2
  }]
}))

const comparisonChartData = computed(() => ({
  labels: series.value.map(item => item.date),
  datasets: [
    {
      label: selectedEntityName.value || t('userStats.chart.current'),
      data: series.value.map(item => item.total_cost),
      borderColor: 'rgb(59, 130, 246)',
      tension: 0.25,
      pointRadius: 2
    },
    {
      label: comparedEntityName.value || t('userStats.chart.comparison'),
      data: comparisonSeries.value.map(item => item.total_cost),
      borderColor: 'rgb(234, 179, 8)',
      tension: 0.25,
      pointRadius: 2
    }
  ]
}))

function scheduleLeaderboardLoad() {
  if (!ready) return
  leaderboardRequestId += 1
  if (leaderboardDebounceTimer) clearTimeout(leaderboardDebounceTimer)
  leaderboardDebounceTimer = setTimeout(() => {
    leaderboardDebounceTimer = null
    void loadLeaderboard()
  }, 120)
}

function schedulePanelLoad() {
  if (!ready) return
  panelRequestId += 1
  if (panelDebounceTimer) clearTimeout(panelDebounceTimer)
  panelDebounceTimer = setTimeout(() => {
    panelDebounceTimer = null
    void loadPanels()
  }, 120)
}

watch(scope, () => {
  leaderboardOffset.value = 0
  ensureSelectedEntity()
  scheduleLeaderboardLoad()
  schedulePanelLoad()
})
watch([() => props.range, () => props.revision, granularity, metric], (next, previous) => {
  if (next[1] === previous[1]) leaderboardOffset.value = 0
  if (!canUseHourly.value && granularity.value === 'hour') granularity.value = 'day'
  scheduleLeaderboardLoad()
  schedulePanelLoad()
}, { deep: true })
watch([selectedEntityId, compareEntityId], schedulePanelLoad)

async function reload() {
  if (!ready || entitiesError.value) await loadEntities()
  if (disposed) return
  ready = true
  await Promise.all([loadLeaderboard(), loadPanels()])
}
onMounted(reload)

onUnmounted(() => {
  disposed = true
  if (leaderboardDebounceTimer) clearTimeout(leaderboardDebounceTimer)
  if (panelDebounceTimer) clearTimeout(panelDebounceTimer)
  leaderboardRequestId += 1
  panelRequestId += 1
})
</script>
