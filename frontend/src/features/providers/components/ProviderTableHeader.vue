<template>
  <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/50">
    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
      <!-- 左侧：标题 -->
      <div class="flex shrink-0 items-center gap-2">
        <Button
          v-if="policyPanelCollapsed !== undefined"
          variant="ghost"
          size="icon"
          class="-ml-2 h-8 w-8 shrink-0"
          :class="policyPanelCollapsed ? 'text-muted-foreground' : 'text-primary'"
          :title="policyPanelCollapsed ? t('ui.expandPolicyGroups') : t('ui.collapsePolicyGroups')"
          :aria-label="policyPanelCollapsed ? t('ui.expandPolicyGroups') : t('ui.collapsePolicyGroups')"
          :aria-expanded="!policyPanelCollapsed"
          aria-controls="provider-policy-editor"
          @click="$emit('togglePolicyPanel')"
        >
          <SlidersHorizontal class="h-4 w-4" />
        </Button>
        <h3 class="text-sm sm:text-base font-semibold text-foreground">
          {{ t('ui.providerManagement') }}
        </h3>
      </div>

      <!-- 右侧：操作区 -->
      <div class="flex flex-wrap items-center gap-2">
        <!-- 搜索框 -->
        <div class="relative">
          <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground/70 z-10 pointer-events-none" />
          <Input
            id="provider-search"
            :model-value="searchQuery"
            type="text"
            :placeholder="t('ui.searchProviders')"
            class="w-32 sm:w-44 pl-8 pr-3 h-8 text-sm bg-muted/30 border-border/50 focus:border-primary/50 transition-colors"
            @update:model-value="$emit('update:searchQuery', $event)"
          />
        </div>

        <!-- API 格式筛选 -->
        <div class="xl:hidden">
          <Select
            :model-value="filterApiFormat"
            @update:model-value="$emit('update:filterApiFormat', $event)"
          >
            <SelectTrigger class="w-20 sm:w-28 h-8 text-xs border-border/60">
              <SelectValue :placeholder="t('ui.allFormats')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="fmt in apiFormatFilters"
                :key="fmt.value"
                :value="fmt.value"
              >
                {{ fmt.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 模型筛选 -->
        <div
          v-if="showModelFilter !== false"
          class="xl:hidden"
        >
          <Select
            :model-value="filterModel"
            @update:model-value="$emit('update:filterModel', $event)"
          >
            <SelectTrigger class="w-20 sm:w-36 h-8 text-xs border-border/60">
              <SelectValue :placeholder="t('ui.allModels')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="model in modelFilters"
                :key="model.value"
                :value="model.value"
              >
                {{ model.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 重置筛选 -->
        <Button
          v-if="hasActiveFilters"
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.resetFilters')"
          @click="$emit('resetFilters')"
        >
          <FilterX class="w-3.5 h-3.5" />
        </Button>

        <div class="hidden sm:block h-4 w-px bg-border" />

        <!-- 操作按钮 -->
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.batchProcessProviders')"
          :disabled="loading"
          @click="$emit('batchProcess')"
        >
          <Users class="w-3.5 h-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.addProvider')"
          @click="$emit('addProvider')"
        >
          <Plus class="w-3.5 h-3.5" />
        </Button>
        <RefreshButton
          :loading="loading"
          @click="$emit('refresh')"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Search, Plus, FilterX, SlidersHorizontal, Users } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import RefreshButton from '@/components/ui/refresh-button.vue'
import type { FilterOption } from '@/features/providers/composables/useProviderFilters'
import { useI18n } from '@/i18n'

withDefaults(defineProps<{
  policyPanelCollapsed?: boolean
  searchQuery: string
  filterApiFormat: string
  showModelFilter?: boolean
  filterModel: string
  apiFormatFilters: FilterOption[]
  modelFilters: FilterOption[]
  hasActiveFilters: boolean
  loading: boolean
}>(), { showModelFilter: true, policyPanelCollapsed: undefined })

defineEmits<{
  'togglePolicyPanel': []
  'update:searchQuery': [value: string]
  'update:filterApiFormat': [value: string]
  'update:filterModel': [value: string]
  'resetFilters': []
  'batchProcess': []
  'addProvider': []
  'refresh': []
}>()

const { t } = useI18n()

</script>
