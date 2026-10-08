<template>
  <div class="relative min-w-0 max-w-full space-y-5 pb-8">
    <header class="flex flex-wrap items-start justify-between gap-4 border-b border-border pb-5">
      <div>
        <h1 class="text-xl font-semibold">
          {{ publicPage ? '服务状态' : '健康监控' }}
        </h1>
        <p
          v-if="meta"
          class="mt-2 text-xs text-muted-foreground"
        >
          更新于 {{ dateTime(meta.generated_at) }}
          <span
            v-if="meta.freshness === 'stale'"
            class="ml-2 text-amber-600"
          >数据已过期</span>
          <span
            v-else-if="meta.freshness === 'unknown'"
            class="ml-2"
          >采集进度未知</span>
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <label
          class="sr-only"
          for="health-window"
        >时间范围</label>
        <select
          id="health-window"
          :value="window"
          class="h-9 rounded-md border border-input bg-background px-3 text-sm"
          @change="changeWindow"
        >
          <option value="1h">
            近 1 小时
          </option><option value="6h">
            近 6 小时
          </option><option value="24h">
            近 24 小时
          </option><option value="72h">
            近 72 小时
          </option>
        </select>
        <button
          v-if="isAdmin"
          type="button"
          class="flex h-9 items-center gap-2 rounded-md border border-input px-3 text-sm hover:bg-muted"
          @click="openPublication"
        >
          <Globe class="h-4 w-4" />状态页发布
        </button>
        <button
          type="button"
          title="刷新"
          aria-label="刷新健康数据"
          class="flex h-9 w-9 items-center justify-center rounded-md border border-input hover:bg-muted disabled:opacity-50"
          :disabled="loading"
          @click="load"
        >
          <RefreshCw
            class="h-4 w-4"
            :class="{ 'animate-spin': loading }"
          />
        </button>
      </div>
    </header>
    <nav
      class="flex gap-5 overflow-x-auto border-b border-border"
      aria-label="健康对象"
    >
      <button
        v-for="tab in tabs"
        :key="tab.kind"
        type="button"
        class="shrink-0 border-b-2 px-1 pb-3 text-sm"
        :class="kind === tab.kind ? 'border-primary font-medium text-foreground' : 'border-transparent text-muted-foreground'"
        :aria-current="kind === tab.kind ? 'page' : undefined"
        @click="setQuery({ kind: tab.kind, offset: undefined, object: undefined }, true)"
      >
        {{ tab.label }}
      </button>
    </nav>
    <div
      v-if="error"
      role="alert"
      class="flex items-start gap-2 border-l-2 border-destructive bg-destructive/5 px-4 py-3 text-sm text-destructive"
    >
      <AlertTriangle class="mt-0.5 h-4 w-4 shrink-0" />{{ error }}
    </div>
    <section
      v-if="summary"
      class="grid grid-cols-2 gap-y-5 border-b border-border pb-5 md:grid-cols-4"
      aria-label="健康摘要"
    >
      <div>
        <div class="text-xs text-muted-foreground">
          总体状态
        </div><div class="mt-2 flex items-center gap-2 text-lg font-semibold">
          <span
            class="h-2.5 w-2.5 rounded-full"
            :class="statusColor(summary.status)"
          />{{ statusLabel(summary.status) }}
        </div><div class="mt-1 text-xs text-muted-foreground">
          {{ summary.object_count }} 个对象
        </div>
      </div>
      <div>
        <div class="text-xs text-muted-foreground">
          服务可用率
        </div><div class="mt-2 text-lg font-semibold tabular-nums">
          {{ percentage(summary.requests.service_availability.value) }}
        </div><div class="mt-1 text-xs text-muted-foreground">
          {{ summary.requests.service_availability.denominator.toLocaleString() }} 个有效样本
        </div>
      </div>
      <div>
        <div class="text-xs text-muted-foreground">
          异常对象
        </div><div class="mt-2 text-lg font-semibold tabular-nums">
          {{ summary.degraded_count + summary.unavailable_count }}
        </div><div class="mt-1 text-xs text-muted-foreground">
          {{ summary.unknown_count }} 个状态未知
        </div>
      </div>
      <div>
        <div class="text-xs text-muted-foreground">
          最终请求成功率
        </div><div class="mt-2 text-lg font-semibold tabular-nums">
          {{ percentage(summary.requests.request_success.value) }}
        </div><div class="mt-1 text-xs text-muted-foreground">
          {{ summary.requests.request_count.toLocaleString() }} 次请求
        </div>
      </div>
    </section>
    <div
      v-if="loading && !page"
      role="status"
      class="flex min-h-48 items-center justify-center gap-2 text-sm text-muted-foreground"
    >
      <LoaderCircle class="h-4 w-4 animate-spin" />加载中
    </div>
    <template v-else-if="page">
      <div
        v-if="page.items.length === 0"
        class="border-y border-dashed border-border py-16 text-center text-sm text-muted-foreground"
      >
        {{ publicPage ? '此视角暂无公开服务' : '此视角暂无监控对象' }}
      </div>
      <div
        v-else
        class="relative max-w-full overflow-x-auto"
      >
        <table class="health-object-table w-full min-w-[740px] text-left text-sm">
          <thead class="border-b border-border text-xs text-muted-foreground">
            <tr>
              <th class="py-3 pr-4 font-medium">
                {{ activeLabel }}
              </th><th class="px-3 py-3 font-medium">
                状态
              </th><th class="px-3 py-3 text-right font-medium">
                服务可用率
              </th><th class="px-3 py-3 text-right font-medium">
                请求数
              </th><th class="px-3 py-3 text-right font-medium">
                平均耗时
              </th><th class="px-3 py-3 font-medium">
                窗口状态
              </th><th class="w-10">
                <span class="sr-only">详情</span>
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border">
            <tr
              v-for="object in page.items"
              :key="object.id"
              class="hover:bg-muted/40"
            >
              <td class="max-w-64 py-4 pr-4">
                <button
                  class="break-words text-left font-medium hover:text-primary"
                  type="button"
                  @click="selectObject(object.id)"
                >
                  {{ object.name }}
                </button>
              </td>
              <td class="whitespace-nowrap px-3 py-4">
                <span class="inline-flex items-center gap-2"><span
                  class="h-2 w-2 rounded-full"
                  :class="statusColor(object.status)"
                />{{ statusLabel(object.status) }}</span><div
                  v-if="object.coverage.sample_status !== 'sufficient'"
                  class="mt-1 text-xs text-muted-foreground"
                >
                  {{ object.coverage.sample_status === 'empty' ? '暂无样本' : '样本不足' }}
                </div><div
                  v-else-if="object.coverage.status === 'partial'"
                  class="mt-1 text-xs text-amber-600"
                >
                  分类不完整
                </div>
              </td>
              <td class="px-3 py-4 text-right tabular-nums">
                {{ percentage(object.service_availability.value) }}
              </td><td class="px-3 py-4 text-right tabular-nums">
                {{ object.request_count.toLocaleString() }}
              </td><td class="whitespace-nowrap px-3 py-4 text-right tabular-nums">
                {{ object.average_latency_ms == null ? '未知' : `${Math.round(object.average_latency_ms).toLocaleString()} ms` }}
              </td>
              <td class="px-3 py-4">
                <div class="flex h-7 min-w-36 gap-0.5">
                  <span
                    v-for="(bucket, index) in object.timeline"
                    :key="index"
                    class="min-w-1 flex-1 rounded-sm"
                    :class="statusColor(bucket.status)"
                    :title="`${dateTime(bucket.from)} - ${dateTime(bucket.to)}: ${statusLabel(bucket.status)} (${bucket.service_availability.denominator} 个样本)`"
                  /><span
                    v-if="object.timeline.length === 0"
                    class="flex-1 rounded-sm bg-muted"
                    title="暂无样本"
                  />
                </div>
              </td>
              <td class="pl-2">
                <button
                  type="button"
                  class="flex h-8 w-8 items-center justify-center rounded-md hover:bg-muted"
                  :aria-label="`查看 ${object.name}`"
                  title="查看详情"
                  @click="selectObject(object.id)"
                >
                  <ChevronRight class="h-4 w-4" />
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="flex items-center justify-between gap-3 border-t border-border pt-4 text-xs text-muted-foreground">
        <span>共 {{ page.total }} 个对象</span><div class="flex items-center gap-3">
          <button
            type="button"
            title="上一页"
            aria-label="上一页"
            class="flex h-8 w-8 items-center justify-center rounded-md border border-input disabled:opacity-30"
            :disabled="offset === 0 || loading"
            @click="setQuery({ offset: String(Math.max(0, offset - 25)) })"
          >
            <ChevronLeft class="h-4 w-4" />
          </button><span>{{ Math.floor(offset / 25) + 1 }} / {{ Math.max(1, Math.ceil(page.total / 25)) }}</span><button
            type="button"
            title="下一页"
            aria-label="下一页"
            class="flex h-8 w-8 items-center justify-center rounded-md border border-input disabled:opacity-30"
            :disabled="offset + 25 >= page.total || loading"
            @click="setQuery({ offset: String(offset + 25) })"
          >
            <ChevronRight class="h-4 w-4" />
          </button>
        </div>
      </div>
    </template>
    <Dialog
      :open="Boolean(selectedId)"
      :title="detail?.name || '健康详情'"
      max-width="2xl"
      @update:open="closeDetail"
    >
      <div
        v-if="detailLoading"
        role="status"
        class="py-8 text-center text-sm text-muted-foreground"
      >
        加载中
      </div>
      <div
        v-else-if="detailError"
        role="alert"
        class="text-sm text-destructive"
      >
        {{ detailError }}
      </div>
      <div
        v-else-if="detail"
        class="space-y-5"
      >
        <div class="flex items-center gap-2 font-medium">
          <span
            class="h-2.5 w-2.5 rounded-full"
            :class="statusColor(detail.status)"
          />{{ statusLabel(detail.status) }}
        </div>
        <dl class="grid grid-cols-2 gap-x-6 gap-y-4 text-sm">
          <div>
            <dt class="text-muted-foreground">
              服务可用率
            </dt><dd class="mt-1 font-medium">
              {{ percentage(detail.service_availability.value) }}
            </dd>
          </div><div>
            <dt class="text-muted-foreground">
              最终请求成功率
            </dt><dd class="mt-1 font-medium">
              {{ percentage(detail.request_success.value) }}
            </dd>
          </div><div>
            <dt class="text-muted-foreground">
              有效服务样本
            </dt><dd class="mt-1">
              {{ detail.service_availability.denominator.toLocaleString() }}
            </dd>
          </div><div>
            <dt class="text-muted-foreground">
              已排除请求
            </dt><dd class="mt-1">
              {{ detail.coverage.excluded_count.toLocaleString() }}
            </dd>
          </div><div>
            <dt class="text-muted-foreground">
              未分类失败
            </dt><dd class="mt-1">
              {{ detail.coverage.unknown_failure_count.toLocaleString() }}
            </dd>
          </div><div>
            <dt class="text-muted-foreground">
              最后请求
            </dt><dd class="mt-1">
              {{ detail.last_request_at ? dateTime(detail.last_request_at) : '无流量' }}
            </dd>
          </div><template v-if="adminDetail">
            <div>
              <dt class="text-muted-foreground">
                渠道尝试成功率
              </dt><dd class="mt-1">
                {{ percentage(adminDetail.attempts.success.value) }}
              </dd>
            </div><div>
              <dt class="text-muted-foreground">
                执行中渠道尝试
              </dt><dd class="mt-1">
                {{ adminDetail.attempts.in_progress_count }}
              </dd>
            </div>
          </template>
        </dl>
        <div class="flex h-10 gap-1">
          <span
            v-for="(bucket, index) in detail.timeline"
            :key="index"
            class="min-w-1 flex-1 rounded-sm"
            :class="statusColor(bucket.status)"
            :title="`${dateTime(bucket.from)}: ${percentage(bucket.service_availability.value)}`"
          />
        </div>
        <RouterLink
          v-if="adminDetail"
          :to="usageLink"
          class="inline-flex items-center gap-2 text-sm font-medium text-primary"
        >
          使用记录<ArrowUpRight class="h-4 w-4" />
        </RouterLink>
      </div>
    </Dialog>
    <Dialog
      v-model:open="publicationOpen"
      title="状态页发布（/status）"
      max-width="3xl"
    >
      <div
        v-if="publicationLoading"
        class="py-8 text-center text-sm text-muted-foreground"
      >
        加载中
      </div>
      <div
        v-else-if="publication"
        class="space-y-4"
      >
        <label class="flex items-center gap-2 text-sm font-medium"><input
          v-model="publication.enabled"
          type="checkbox"
          class="h-4 w-4 accent-primary"
        >启用 /status 状态页</label>
        <div class="overflow-x-auto">
          <table class="w-full min-w-[560px] text-left text-xs">
            <thead>
              <tr class="border-b border-border text-muted-foreground">
                <th class="pb-2 font-medium">
                  类型
                </th><th class="pb-2 font-medium">
                  监控对象
                </th><th class="pb-2 font-medium">
                  公开 ID
                </th><th class="pb-2 font-medium">
                  公开名称
                </th><th />
              </tr>
            </thead><tbody>
              <tr
                v-for="(object, index) in publication.objects"
                :key="index"
              >
                <td class="py-2 pr-2">
                  <select
                    v-model="object.kind"
                    class="h-8 w-24 rounded-md border border-input bg-background px-2"
                  >
                    <option value="api_format">
                      API 格式
                    </option><option value="model">
                      模型
                    </option>
                  </select>
                </td><td class="pr-2">
                  <input
                    v-model="object.value"
                    aria-label="监控对象"
                    class="h-8 w-full rounded-md border border-input bg-background px-2"
                    maxlength="256"
                  >
                </td><td class="pr-2">
                  <input
                    v-model="object.public_id"
                    aria-label="公开 ID"
                    class="h-8 w-full rounded-md border border-input bg-background px-2"
                    maxlength="80"
                  >
                </td><td class="pr-2">
                  <input
                    v-model="object.display_name"
                    aria-label="公开名称"
                    class="h-8 w-full rounded-md border border-input bg-background px-2"
                    maxlength="120"
                  >
                </td><td>
                  <button
                    type="button"
                    title="移除公开对象"
                    aria-label="移除公开对象"
                    class="flex h-8 w-8 items-center justify-center rounded-md hover:bg-muted"
                    @click="publication.objects.splice(index, 1)"
                  >
                    <Trash2 class="h-4 w-4" />
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <button
          type="button"
          class="inline-flex items-center gap-2 text-sm text-primary"
          @click="publication.objects.push({ public_id: '', kind: 'api_format', value: '', display_name: '' })"
        >
          <Plus class="h-4 w-4" />添加对象
        </button>
        <div
          v-if="publicationError"
          role="alert"
          class="text-sm text-destructive"
        >
          {{ publicationError }}
        </div>
        <div class="flex justify-between border-t border-border pt-4">
          <RouterLink
            to="/status"
            target="_blank"
            class="inline-flex items-center gap-2 text-sm text-muted-foreground"
          >
            公开状态页<ArrowUpRight class="h-4 w-4" />
          </RouterLink><button
            type="button"
            class="inline-flex h-9 items-center gap-2 rounded-md bg-primary px-4 text-sm text-primary-foreground disabled:opacity-50"
            :disabled="savingPublication"
            @click="savePublication"
          >
            <Save class="h-4 w-4" />保存
          </button>
        </div>
      </div>
      <div
        v-else-if="publicationError"
        role="alert"
        class="text-sm text-destructive"
      >
        {{ publicationError }}
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { AlertTriangle, ArrowUpRight, ChevronLeft, ChevronRight, Globe, LoaderCircle, Plus, RefreshCw, Save, Trash2 } from 'lucide-vue-next'
import Dialog from '@/components/ui/dialog/Dialog.vue'
import {
  getAdminHealthObject, getAdminHealthObjects, getAdminHealthSummary, getHealthPublication,
  getPublicHealthObject, getPublicHealthObjects, getPublicHealthSummaryV2, saveHealthPublication,
  getUserHealthObject, getUserHealthObjects, getUserHealthSummary,
  type AdminHealthObject, type HealthMeta, type HealthObjectKind, type HealthObjectsPage,
  type HealthPublication, type HealthQuery, type HealthSummaryV2, type HealthWindow,
  type PublicHealthObject, type PublicHealthQuery, type ServiceHealthStatus,
} from '@/api/endpoints/health-v2'

const props = withDefaults(defineProps<{ isAdmin?: boolean; publicPage?: boolean }>(), { isAdmin: false, publicPage: false })
const route = useRoute()
const router = useRouter()
const tabs = computed(() => [
  { kind: 'api_format' as const, label: 'API 格式' },
  { kind: 'model' as const, label: '模型' },
  ...(props.isAdmin ? [{ kind: 'provider' as const, label: '提供商' }] : []),
])
const kind = computed<HealthObjectKind>(() => tabs.value.find(tab => tab.kind === route.query.kind)?.kind ?? 'api_format')
const window = computed<HealthWindow>(() => ['1h', '6h', '24h', '72h'].includes(String(route.query.window)) ? route.query.window as HealthWindow : '6h')
const offset = computed(() => { const value = Number(route.query.offset); return Number.isInteger(value) && value >= 0 && value <= 10_000 ? value : 0 })
const selectedId = computed(() => typeof route.query.object === 'string' ? route.query.object : '')
const activeLabel = computed(() => tabs.value.find(tab => tab.kind === kind.value)?.label)
const summary = ref<HealthSummaryV2 | null>(null)
const page = ref<HealthObjectsPage<PublicHealthObject | AdminHealthObject> | null>(null)
const meta = ref<HealthMeta | null>(null)
const loading = ref(false)
const error = ref('')
const detail = ref<PublicHealthObject | AdminHealthObject | null>(null)
const detailLoading = ref(false)
const detailError = ref('')
const adminDetail = computed(() => detail.value && 'attempts' in detail.value ? detail.value as AdminHealthObject : null)
let controller: AbortController | null = null
let detailController: AbortController | null = null
let loadVersion = 0
let detailVersion = 0
let refreshTimer: ReturnType<typeof setTimeout> | undefined
const query = computed<HealthQuery>(() => ({ kind: kind.value, window: window.value, limit: 25, offset: offset.value }))
function setQuery(patch: Record<string, string | undefined>, push = false) { void router[push ? 'push' : 'replace']({ query: { ...route.query, ...patch } }) }
function changeWindow(event: Event) { setQuery({ window: (event.target as HTMLSelectElement).value, offset: undefined, object: undefined }) }
function selectObject(id: string) { setQuery({ object: id }, true) }
function closeDetail(open: boolean) { if (!open) setQuery({ object: undefined }) }
function percentage(value: number | null) { return value == null ? '未知' : `${(value * 100).toFixed(2)}%` }
function dateTime(value: string) { return new Date(value).toLocaleString(undefined, { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }) }
function statusLabel(status: ServiceHealthStatus) { return { healthy: '正常', degraded: '波动', unavailable: '异常', unknown: '未知' }[status] }
function statusColor(status: ServiceHealthStatus) { return { healthy: 'bg-emerald-500', degraded: 'bg-amber-500', unavailable: 'bg-red-500', unknown: 'bg-muted-foreground/25' }[status] }
async function load() {
  const version = ++loadVersion
  controller?.abort()
  clearTimeout(refreshTimer)
  controller = new AbortController()
  loading.value = true
  error.value = ''
  try {
    const [nextSummary, nextPage] = props.isAdmin
      ? await Promise.all([getAdminHealthSummary(query.value, controller.signal), getAdminHealthObjects(query.value, controller.signal)])
      : props.publicPage
        ? await Promise.all([getPublicHealthSummaryV2(query.value as PublicHealthQuery, controller.signal), getPublicHealthObjects(query.value as PublicHealthQuery, controller.signal)])
        : await Promise.all([getUserHealthSummary(query.value as PublicHealthQuery, controller.signal), getUserHealthObjects(query.value as PublicHealthQuery, controller.signal)])
    if (version !== loadVersion) return
    summary.value = nextSummary.data
    page.value = nextPage.data
    meta.value = nextSummary.meta
  } catch (cause) {
    if (version !== loadVersion || controller.signal.aborted) return
    const status = (cause as { response?: { status?: number } }).response?.status
    error.value = status === 404 && props.publicPage ? '公开状态页尚未启用' : '健康数据暂时不可用，请稍后刷新。'
  } finally { if (version === loadVersion) { loading.value = false; refreshTimer = setTimeout(load, 60_000) } }
}
async function loadDetail() {
  const version = ++detailVersion
  detailController?.abort()
  detail.value = null
  detailError.value = ''
  if (!selectedId.value) { detailLoading.value = false; return }
  detailController = new AbortController()
  detailLoading.value = true
  try {
    const result = props.isAdmin
      ? await getAdminHealthObject(selectedId.value, query.value, detailController.signal)
      : props.publicPage
        ? await getPublicHealthObject(selectedId.value, query.value as PublicHealthQuery, detailController.signal)
        : await getUserHealthObject(selectedId.value, query.value as PublicHealthQuery, detailController.signal)
    if (version === detailVersion) detail.value = result.data
  } catch { if (version === detailVersion && !detailController.signal.aborted) detailError.value = '对象详情暂时不可用' }
  finally { if (version === detailVersion) detailLoading.value = false }
}
const usageLink = computed(() => ({ path: '/admin/usage', query: {
  from: meta.value?.range.from, to: meta.value?.range.to, timezone: 'UTC',
  [kind.value === 'model' ? 'model' : kind.value === 'provider' ? 'provider_id' : 'api_format']: adminDetail.value?.source_value,
} }))
const publicationOpen = ref(false)
const publicationLoading = ref(false)
const publication = ref<HealthPublication | null>(null)
const publicationError = ref('')
const savingPublication = ref(false)
async function openPublication() {
  publicationOpen.value = true; publicationLoading.value = true; publicationError.value = ''
  try { publication.value = await getHealthPublication() }
  catch { publicationError.value = '状态页发布配置暂时不可用' }
  finally { publicationLoading.value = false }
}
async function savePublication() {
  if (!publication.value) return
  savingPublication.value = true; publicationError.value = ''
  try { publication.value = await saveHealthPublication(publication.value); publicationOpen.value = false }
  catch (cause) { publicationError.value = (cause as { response?: { data?: { detail?: string } } }).response?.data?.detail || '保存失败，请稍后重试' }
  finally { savingPublication.value = false }
}
watch([kind, window, offset, () => props.isAdmin, () => props.publicPage], () => { summary.value = null; page.value = null; void load() }, { immediate: true })
watch([selectedId, kind, window], loadDetail, { immediate: true })
onBeforeUnmount(() => { ++loadVersion; ++detailVersion; controller?.abort(); detailController?.abort(); clearTimeout(refreshTimer) })
</script>

<style scoped>
.health-object-table th {
  white-space: nowrap;
}
.health-object-table td:first-child {
  min-width: 160px;
  overflow-wrap: anywhere;
}
</style>
