<template>
  <Dialog
    :model-value="isOpen"
    :title="t('ui.editAccount')"
    :description="t('ui.editOauthAccountSettings')"
    :icon="SquarePen"
    size="xl"
    @update:model-value="handleDialogUpdate"
  >
    <form
      class="space-y-3"
      autocomplete="off"
      @submit.prevent="handleSave"
    >
      <!-- 基本信息：账号名称 + 备注 -->
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="name">{{ t('ui.accountName') }}</Label>
          <Input
            id="name"
            v-model="form.name"
            required
            :placeholder="t('ui.eGPrimaryAccountBackupAccount')"
            maxlength="100"
            autocomplete="off"
          />
        </div>
        <div>
          <Label for="note">{{ t('ui.notes') }}</Label>
          <Input
            id="note"
            v-model="form.note"
            :placeholder="t('ui.optionalNotes')"
          />
        </div>
      </div>

      <!-- 配置项 -->
      <div class="grid grid-cols-4 gap-3">
        <div>
          <Label
            for="internal_priority"
            class="text-xs"
          >{{ t('ui.priority565d646') }}</Label>
          <Input
            id="internal_priority"
            v-model.number="form.internal_priority"
            type="number"
            min="0"
            placeholder="10"
            class="h-8"
          />
          <p class="text-xs text-muted-foreground mt-0.5">
            {{ t('ui.lowerValuesHaveHigherPriority') }}
          </p>
        </div>
        <div>
          <Label
            for="rpm_limit"
            class="text-xs"
          >{{ t('ui.rpmLimit50b7143') }}</Label>
          <Input
            id="rpm_limit"
            :model-value="form.rpm_limit ?? ''"
            type="number"
            min="1"
            max="10000"
            :placeholder="t('ui.adaptive')"
            class="h-8"
            @update:model-value="(v) => form.rpm_limit = parseNullableNumberInput(v, { min: 1, max: 10000 })"
          />
          <p class="text-xs text-muted-foreground mt-0.5">
            {{ t('ui.leaveEmptyForAdaptive') }}
          </p>
        </div>
        <div>
          <Label
            for="concurrent_limit"
            class="text-xs"
          >{{ t('ui.concurrentRequestLimit') }}</Label>
          <Input
            id="concurrent_limit"
            :model-value="form.concurrent_limit ?? ''"
            type="number"
            min="0"
            :placeholder="t('ui.noRestriction')"
            class="h-8"
            @update:model-value="(v) => form.concurrent_limit = parseNullableNumberInput(v, { min: 0 })"
          />
          <p class="text-xs text-muted-foreground mt-0.5">
            {{ t('ui.leaveEmptyOrEnter0ForNoLimit') }}
          </p>
        </div>
        <div>
          <Label
            for="cache_ttl_minutes"
            class="text-xs"
          >{{ t('ui.cacheTtld8a1617') }}</Label>
          <Input
            id="cache_ttl_minutes"
            :model-value="form.cache_ttl_minutes ?? ''"
            type="number"
            min="0"
            max="60"
            class="h-8"
            @update:model-value="(v) => form.cache_ttl_minutes = parseNumberInput(v, { min: 0, max: 60 }) ?? 5"
          />
          <p class="text-xs text-muted-foreground mt-0.5">
            {{ t('ui.minutes0Disables') }}
          </p>
        </div>
        <div>
          <Label
            for="max_probe_interval_minutes"
            class="text-xs"
          >{{ t('ui.circuitProbe') }}</Label>
          <Input
            id="max_probe_interval_minutes"
            :model-value="form.max_probe_interval_minutes ?? ''"
            type="number"
            min="0"
            max="32"
            placeholder="32"
            class="h-8"
            @update:model-value="(v) => form.max_probe_interval_minutes = parseNumberInput(v, { min: 0, max: 32 }) ?? 32"
          />
          <p class="text-xs text-muted-foreground mt-0.5">
            {{ t('ui.032Minutes') }}
          </p>
        </div>
      </div>

      <!-- 自动获取模型 -->
      <div class="space-y-3 py-2 px-3 rounded-md border border-border/60 bg-muted/30">
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <Label class="text-sm font-medium">{{ t('ui.autoFetchUpstreamModels') }}</Label>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.periodicallyUpdateUpstreamModelsForUseWithModelMapping') }}
            </p>
            <p
              v-if="showAutoFetchWarning"
              class="text-xs text-amber-600 dark:text-amber-400"
            >
              {{ autoFetchWarningMessage }}
            </p>
          </div>
          <Switch v-model="form.auto_fetch_models" />
        </div>

        <!-- 模型过滤规则（仅当开启自动获取时显示） -->
        <div
          v-if="form.auto_fetch_models"
          class="space-y-2 pt-2 border-t border-border/40"
        >
          <div class="grid grid-cols-2 gap-3">
            <div>
              <Label class="text-xs">{{ t('ui.includeRules') }}</Label>
              <Input
                v-model="form.model_include_patterns_text"
                :placeholder="t('ui.gptClaudeLeaveEmptyToIncludeAll')"
                class="h-8 text-sm"
              />
            </div>
            <div>
              <Label class="text-xs">{{ t('ui.excludeRules') }}</Label>
              <Input
                v-model="form.model_exclude_patterns_text"
                placeholder="*-preview, *-beta"
                class="h-8 text-sm"
              />
            </div>
          </div>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.commaSeparatedSupportsAndWildcardsCaseInsensitive') }}
          </p>
        </div>
      </div>
    </form>

    <template #footer>
      <Button
        variant="outline"
        @click="handleCancel"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        :disabled="saving || !canSave"
        @click="handleSave"
      >
        {{ saving ? t('ui.saving') : t('ui.save') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ref, computed, watch } from 'vue'
import { Dialog, Button, Input, Label, Switch } from '@/components/ui'
import { SquarePen } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { useFormDialog } from '@/composables/useFormDialog'
import { parseApiError } from '@/utils/errorParser'
import { parseNumberInput, parseNullableNumberInput } from '@/utils/form'
import {
  updateProviderKey,
  type EndpointAPIKey,
  type EndpointAPIKeyUpdate,
} from '@/api/endpoints'

const props = defineProps<{
  open: boolean
  editingKey: EndpointAPIKey | null
}>()

const emit = defineEmits<{
  close: []
  saved: [key: EndpointAPIKey]
}>()

const { t } = useI18n()

const { success, error: showError } = useToast()

// 显示自动获取模型警告：编辑模式下，原本未启用但现在启用，且已有 allowed_models
const showAutoFetchWarning = computed(() => {
  if (!props.editingKey) return false
  // 原本已启用，不需要警告
  if (props.editingKey.auto_fetch_models) return false
  // 现在未启用，不需要警告
  if (!form.value.auto_fetch_models) return false
  // 检查是否有已配置的模型权限
  const allowedModels = props.editingKey.allowed_models
  if (!allowedModels) return false
  if (Array.isArray(allowedModels) && allowedModels.length === 0) return false
  if (typeof allowedModels === 'object' && Object.keys(allowedModels).length === 0) return false
  return true
})

const autoFetchWarningMessage = computed(() => {
  if (!showAutoFetchWarning.value || !props.editingKey?.allowed_models) return ''
  const models = Array.isArray(props.editingKey.allowed_models)
    ? props.editingKey.allowed_models
    : []
  if (models.length === 0) return ''
  return t('providers.migration.thisKeyCurrentlyAllowsTheseef7e31', { value: String(models.map(model => `“${model}”`).join('、')) })
})

// 表单是否可以保存
const canSave = computed(() => {
  // 必须填写名称
  if (!form.value.name.trim()) return false
  return true
})

const isOpen = computed(() => props.open)
const saving = ref(false)

const form = ref({
  name: '',
  internal_priority: 10,
  rpm_limit: undefined as number | null | undefined,
  concurrent_limit: undefined as number | null | undefined,
  cache_ttl_minutes: 5,
  max_probe_interval_minutes: 32,
  note: '',
  auto_fetch_models: false,
  model_include_patterns_text: '',
  model_exclude_patterns_text: ''
})

// ---------------------------------------------------------------------------
// Dirty 状态跟踪：通过快照比较判断表单是否被修改
// ---------------------------------------------------------------------------
const formSnapshot = ref('')

function takeSnapshot() {
  formSnapshot.value = JSON.stringify(form.value)
}

const isDirty = computed(() => {
  if (!formSnapshot.value) return false
  return JSON.stringify(form.value) !== formSnapshot.value
})

// 对话框关闭时清除快照（快照在 loadKeyData 中拍摄）
watch(isOpen, (val) => {
  if (!val) {
    formSnapshot.value = ''
  }
})

// 重置表单
function resetForm() {
  form.value = {
    name: '',
    internal_priority: 10,
    rpm_limit: undefined,
    concurrent_limit: undefined,
    cache_ttl_minutes: 5,
    max_probe_interval_minutes: 32,
    note: '',
    auto_fetch_models: false,
    model_include_patterns_text: '',
    model_exclude_patterns_text: ''
  }
  formSnapshot.value = ''
}

// 加载密钥数据
function loadKeyData() {
  if (!props.editingKey) return
  form.value = {
    name: props.editingKey.name,
    internal_priority: props.editingKey.internal_priority ?? 10,
    rpm_limit: props.editingKey.rpm_limit ?? undefined,
    concurrent_limit: props.editingKey.concurrent_limit ?? undefined,
    cache_ttl_minutes: props.editingKey.cache_ttl_minutes ?? 5,
    max_probe_interval_minutes: props.editingKey.max_probe_interval_minutes ?? 32,
    note: props.editingKey.note || '',
    auto_fetch_models: props.editingKey.auto_fetch_models ?? false,
    model_include_patterns_text: (props.editingKey.model_include_patterns || []).join(', '),
    model_exclude_patterns_text: (props.editingKey.model_exclude_patterns || []).join(', ')
  }
  // 数据加载完成后拍快照，作为 dirty 判断的基准
  takeSnapshot()
}

// 使用 useFormDialog 统一处理对话框逻辑
const { handleDialogUpdate, handleCancel } = useFormDialog({
  isOpen: () => props.open,
  entity: () => props.editingKey,
  isLoading: saving,
  isDirty: () => isDirty.value,
  onClose: () => emit('close'),
  loadData: loadKeyData,
  resetForm,
})

// 将逗号分隔的文本解析为数组（去空、去重）
function parsePatternText(text: string): string[] {
  if (!text.trim()) return []
  const patterns = text
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0)
  return [...new Set(patterns)]
}

async function handleSave() {
  if (!props.editingKey) {
    showError(t('ui.cannotSaveAccountInformationIsMissing'), t('ui.error'))
    return
  }

  saving.value = true
  try {
    const shouldClearAllowedModels = !!props.editingKey.auto_fetch_models && !form.value.auto_fetch_models
    const updateData: EndpointAPIKeyUpdate = {
      name: form.value.name,
      internal_priority: form.value.internal_priority,
      rpm_limit: form.value.rpm_limit,
      concurrent_limit: form.value.concurrent_limit,
      cache_ttl_minutes: form.value.cache_ttl_minutes,
      max_probe_interval_minutes: form.value.max_probe_interval_minutes,
      note: form.value.note,
      allowed_models: shouldClearAllowedModels ? null : undefined,
      auto_fetch_models: form.value.auto_fetch_models,
      model_include_patterns: parsePatternText(form.value.model_include_patterns_text),
      model_exclude_patterns: parsePatternText(form.value.model_exclude_patterns_text)
    }

    const updatedKey = await updateProviderKey(props.editingKey.id, updateData)
    success(t('ui.accountUpdated'), t('ui.success'))
    emit('saved', updatedKey)
    emit('close')
  } catch (err: unknown) {
    const errorMessage = parseApiError(err, t('ui.failedToSave'))
    showError(errorMessage, t('ui.error'))
  } finally {
    saving.value = false
  }
}

</script>
