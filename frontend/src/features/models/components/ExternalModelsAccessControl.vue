<template>
  <div
    data-testid="external-models-access-control"
  >
    <Popover
      :open="popoverOpen"
      @update:open="popoverOpen = $event"
    >
      <PopoverTrigger as-child>
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :class="proxyNodeId ? 'text-blue-500' : ''"
          :disabled="loading || !configLoaded"
          :title="t('models.externalCatalog.accessTitle')"
          :aria-label="t('models.externalCatalog.accessTitle')"
          data-testid="external-models-access-trigger"
        >
          <Globe class="w-3.5 h-3.5" />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        class="w-72 p-3"
        side="bottom"
        align="end"
      >
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium">
              {{ t('models.externalCatalog.accessTitle') }}
            </span>
            <Button
              v-if="proxyNodeId"
              variant="ghost"
              size="sm"
              class="h-6 px-2 text-[10px] text-muted-foreground"
              :disabled="loading"
              data-testid="external-models-access-clear"
              @click="updateAccessConfig(null)"
            >
              {{ legacyT('清除') }}
            </Button>
          </div>
          <ProxyNodeSelect
            :model-value="proxyNodeId || ''"
            :disabled="loading || !configLoaded"
            :trigger-aria-label="t('models.externalCatalog.accessTitle')"
            trigger-class="h-8"
            data-testid="external-models-access-select"
            @update:model-value="updateAccessConfig"
          />
          <p class="text-[10px] text-muted-foreground">
            {{ t(proxyNodeId
              ? 'models.externalCatalog.proxyEnabled'
              : 'models.externalCatalog.direct') }}
          </p>
        </div>
      </PopoverContent>
    </Popover>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Globe } from 'lucide-vue-next'
import {
  getExternalModelsAccessConfig,
  updateExternalModelsAccessConfig,
} from '@/api/models-dev'
import { Button, Popover, PopoverContent, PopoverTrigger } from '@/components/ui'
import ProxyNodeSelect from '@/features/providers/components/ProxyNodeSelect.vue'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/i18n'
import { parseApiError } from '@/utils/errorParser'

const { success, error: showError } = useToast()
const { t, legacyT } = useI18n()

const proxyNodeId = ref<string | null>(null)
const popoverOpen = ref(false)
const configLoaded = ref(false)
const loading = ref(true)

async function loadAccessConfig() {
  loading.value = true
  try {
    const config = await getExternalModelsAccessConfig()
    proxyNodeId.value = config.proxy_node_id
    configLoaded.value = true
  } catch (err: unknown) {
    showError(parseApiError(err, t('models.externalCatalog.loadFailed')))
  } finally {
    loading.value = false
  }
}

async function updateAccessConfig(nextProxyNodeId: string | null) {
  if (!configLoaded.value || loading.value) return
  if (nextProxyNodeId === proxyNodeId.value) return

  loading.value = true
  try {
    const result = await updateExternalModelsAccessConfig(nextProxyNodeId)
    proxyNodeId.value = result.proxy_node_id
    popoverOpen.value = false
    success(t('models.externalCatalog.saved'))
  } catch (err: unknown) {
    showError(parseApiError(err, t('models.externalCatalog.saveFailed')))
  } finally {
    loading.value = false
  }
}

onMounted(loadAccessConfig)
</script>
