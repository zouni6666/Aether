<template>
  <Drawer
    :open="open && !!model"
    @close="handleClose"
  >
    <template v-if="model">
      <!-- 标题栏 -->
      <div class="sticky top-0 z-10 bg-background border-b p-4 sm:p-6">
        <div class="flex items-start justify-between gap-3 sm:gap-4">
          <div class="space-y-1 flex-1 min-w-0">
            <h3 class="text-lg sm:text-xl font-bold truncate">
              {{ model.display_name || model.name }}
            </h3>
            <div class="flex items-center gap-2">
              <Badge
                :variant="model.is_active ? 'default' : 'secondary'"
                class="text-xs"
              >
                {{ model.is_active ? t('ui.available4d99c97') : t('ui.disable4e6fd0e') }}
              </Badge>
              <span class="text-sm text-muted-foreground font-mono">{{ model.name }}</span>
              <button
                class="p-0.5 rounded hover:bg-muted transition-colors"
                :title="t('ui.copyModelId')"
                @click="copyToClipboard(model.name)"
              >
                <Copy class="w-3 h-3 text-muted-foreground" />
              </button>
            </div>
            <p
              v-if="model.config?.description"
              class="text-xs text-muted-foreground"
            >
              {{ model.config?.description }}
            </p>
          </div>
          <Button
            variant="ghost"
            size="icon"
            :title="t('ui.close')"
            @click="handleClose"
          >
            <X class="w-4 h-4" />
          </Button>
        </div>
      </div>

      <div class="p-4 sm:p-6 space-y-6">
        <!-- 模型能力 -->
        <div class="space-y-3">
          <h4 class="font-semibold text-sm">
            {{ t('ui.modelCapabilities') }}
          </h4>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="flex items-center gap-2 p-3 rounded-lg border">
              <Zap class="w-5 h-5 text-muted-foreground" />
              <div class="flex-1">
                <p class="text-sm font-medium">
                  Streaming
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ t('ui.streaming') }}
                </p>
              </div>
              <Badge
                :variant="model.config?.streaming !== false ? 'default' : 'secondary'"
                class="text-xs"
              >
                {{ model.config?.streaming !== false ? t('ui.supported') : t('ui.notSupported') }}
              </Badge>
            </div>
            <div class="flex items-center gap-2 p-3 rounded-lg border">
              <ImageIcon class="w-5 h-5 text-muted-foreground" />
              <div class="flex-1">
                <p class="text-sm font-medium">
                  Image Generation
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ t('ui.imageGeneration') }}
                </p>
              </div>
              <Badge
                :variant="model.config?.image_generation === true ? 'default' : 'secondary'"
                class="text-xs"
              >
                {{ model.config?.image_generation === true ? t('ui.supported') : t('ui.notSupported') }}
              </Badge>
            </div>
            <div class="flex items-center gap-2 p-3 rounded-lg border">
              <Database class="w-5 h-5 text-muted-foreground" />
              <div class="flex-1">
                <p class="text-sm font-medium">
                  Embedding
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ t('ui.embeddings') }}
                </p>
              </div>
              <Badge
                :variant="supportsEmbedding(model) ? 'default' : 'secondary'"
                class="text-xs"
              >
                {{ supportsEmbedding(model) ? t('ui.supported') : t('ui.notSupported') }}
              </Badge>
            </div>
            <div class="flex items-center gap-2 p-3 rounded-lg border">
              <Eye class="w-5 h-5 text-muted-foreground" />
              <div class="flex-1">
                <p class="text-sm font-medium">
                  Vision
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ t('ui.vision') }}
                </p>
              </div>
              <Badge
                :variant="model.config?.vision === true ? 'default' : 'secondary'"
                class="text-xs"
              >
                {{ model.config?.vision === true ? t('ui.supported') : t('ui.notSupported') }}
              </Badge>
            </div>
            <div class="flex items-center gap-2 p-3 rounded-lg border">
              <Wrench class="w-5 h-5 text-muted-foreground" />
              <div class="flex-1">
                <p class="text-sm font-medium">
                  Tool Use
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ t('ui.toolCalling') }}
                </p>
              </div>
              <Badge
                :variant="model.config?.function_calling === true ? 'default' : 'secondary'"
                class="text-xs"
              >
                {{ model.config?.function_calling === true ? t('ui.supported') : t('ui.notSupported') }}
              </Badge>
            </div>
            <div class="flex items-center gap-2 p-3 rounded-lg border">
              <Brain class="w-5 h-5 text-muted-foreground" />
              <div class="flex-1">
                <p class="text-sm font-medium">
                  Extended Thinking
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ t('ui.reasoning') }}
                </p>
              </div>
              <Badge
                :variant="model.config?.extended_thinking === true ? 'default' : 'secondary'"
                class="text-xs"
              >
                {{ model.config?.extended_thinking === true ? t('ui.supported') : t('ui.notSupported') }}
              </Badge>
            </div>
          </div>
        </div>

        <!-- 定价信息 -->
        <div class="space-y-3">
          <h4 class="font-semibold text-sm">
            {{ t('ui.pricing') }}
          </h4>

          <ProcessingTierPricingSummary :pricing="model.default_tiered_pricing" />

          <!-- 单阶梯（固定价格）展示 -->
          <div
            v-if="getTierCount(model.default_tiered_pricing) <= 1"
            class="space-y-3"
          >
            <div class="grid grid-cols-2 gap-3">
              <div class="p-3 rounded-lg border">
                <Label class="text-xs text-muted-foreground">{{ t('ui.inputPriceM') }}</Label>
                <p class="text-lg font-semibold mt-1">
                  {{ getFirstTierPrice(model.default_tiered_pricing, 'input_price_per_1m') }}
                </p>
              </div>
              <div class="p-3 rounded-lg border">
                <Label class="text-xs text-muted-foreground">{{ t('ui.outputPriceM') }}</Label>
                <p class="text-lg font-semibold mt-1">
                  {{ getFirstTierPrice(model.default_tiered_pricing, 'output_price_per_1m') }}
                </p>
              </div>
              <div class="p-3 rounded-lg border">
                <Label class="text-xs text-muted-foreground">{{ t('ui.cacheWriteM') }}</Label>
                <p class="text-sm font-mono mt-1">
                  {{ getFirstTierPrice(model.default_tiered_pricing, 'cache_creation_price_per_1m') }}
                </p>
              </div>
              <div class="p-3 rounded-lg border">
                <Label class="text-xs text-muted-foreground">{{ t('ui.cacheReadM') }}</Label>
                <p class="text-sm font-mono mt-1">
                  {{ getFirstTierPrice(model.default_tiered_pricing, 'cache_read_price_per_1m') }}
                </p>
              </div>
            </div>
            <!-- 1h 缓存 -->
            <div
              v-if="getFirst1hCachePrice(model.default_tiered_pricing) !== '-'"
              class="flex items-center gap-3 p-3 rounded-lg border bg-muted/20"
            >
              <Label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('ui.1hCache') }}</Label>
              <span class="text-sm font-mono">{{ getFirst1hCachePrice(model.default_tiered_pricing) }}</span>
            </div>
            <!-- 按次计费 -->
            <div
              v-if="model.default_price_per_request && model.default_price_per_request > 0"
              class="flex items-center gap-3 p-3 rounded-lg border bg-muted/20"
            >
              <Label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('ui.perRequestBilling') }}</Label>
              <span class="text-sm font-mono">${{ model.default_price_per_request.toFixed(3) }}{{ t('ui.requestad9c0d7') }}</span>
            </div>
          </div>

          <!-- 多阶梯计费展示 -->
          <div
            v-else
            class="space-y-3"
          >
            <div class="flex items-center gap-2 text-sm text-muted-foreground">
              <Layers class="w-4 h-4" />
              <span>{{ t('ui.tieredPricing5233988') }}{{ getTierCount(model.default_tiered_pricing) }} {{ t('ui.tiers') }}</span>
            </div>

            <!-- 阶梯价格表格 -->
            <div class="border rounded-lg overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow class="bg-muted/30">
                    <TableHead class="text-xs h-9">
                      {{ t('ui.tier') }}
                    </TableHead>
                    <TableHead class="text-xs h-9 text-right">
                      {{ t('ui.inputM') }}
                    </TableHead>
                    <TableHead class="text-xs h-9 text-right">
                      {{ t('ui.outputM') }}
                    </TableHead>
                    <TableHead class="text-xs h-9 text-right">
                      {{ t('ui.cacheCreation') }}
                    </TableHead>
                    <TableHead class="text-xs h-9 text-right">
                      {{ t('ui.cacheRead1ded655') }}
                    </TableHead>
                    <TableHead class="text-xs h-9 text-right">
                      {{ t('ui.1hCache') }}
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow
                    v-for="(tier, index) in model.default_tiered_pricing?.tiers || []"
                    :key="index"
                    class="text-xs"
                  >
                    <TableCell class="py-2">
                      <span
                        v-if="tier.up_to === null"
                        class="text-muted-foreground"
                      >
                        {{ index === 0 ? t('ui.all') : `> ${formatTierLimit((model.default_tiered_pricing?.tiers || [])[index - 1]?.up_to)}` }}
                      </span>
                      <span v-else>
                        {{ index === 0 ? '0' : formatTierLimit((model.default_tiered_pricing?.tiers || [])[index - 1]?.up_to) }} - {{ formatTierLimit(tier.up_to) }}
                      </span>
                    </TableCell>
                    <TableCell class="py-2 text-right font-mono">
                      ${{ tier.input_price_per_1m?.toFixed(2) || '0.00' }}
                    </TableCell>
                    <TableCell class="py-2 text-right font-mono">
                      ${{ tier.output_price_per_1m?.toFixed(2) || '0.00' }}
                    </TableCell>
                    <TableCell class="py-2 text-right font-mono text-muted-foreground">
                      {{ tier.cache_creation_price_per_1m != null ? `$${tier.cache_creation_price_per_1m.toFixed(2)}` : '-' }}
                    </TableCell>
                    <TableCell class="py-2 text-right font-mono text-muted-foreground">
                      {{ tier.cache_read_price_per_1m != null ? `$${tier.cache_read_price_per_1m.toFixed(2)}` : '-' }}
                    </TableCell>
                    <TableCell class="py-2 text-right font-mono text-muted-foreground">
                      {{ get1hCachePrice(tier) }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>

            <!-- 按次计费（多阶梯时也显示） -->
            <div
              v-if="model.default_price_per_request && model.default_price_per_request > 0"
              class="flex items-center gap-3 p-3 rounded-lg border bg-muted/20"
            >
              <Label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('ui.perRequestBilling') }}</Label>
              <span class="text-sm font-mono">${{ model.default_price_per_request.toFixed(3) }}{{ t('ui.requestad9c0d7') }}</span>
            </div>
          </div>
        </div>
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import {
  X,
  Eye,
  Wrench,
  Brain,
  Zap,
  Copy,
  Layers,
  Database,
  Image as ImageIcon
} from 'lucide-vue-next'
import Drawer from '@/components/ui/drawer/Drawer.vue'
import { useClipboard } from '@/composables/useClipboard'
import Badge from '@/components/ui/badge.vue'
import Button from '@/components/ui/button.vue'
import Label from '@/components/ui/label.vue'
import Table from '@/components/ui/table.vue'
import TableHeader from '@/components/ui/table-header.vue'
import TableBody from '@/components/ui/table-body.vue'
import TableRow from '@/components/ui/table-row.vue'
import TableHead from '@/components/ui/table-head.vue'
import TableCell from '@/components/ui/table-cell.vue'
import { formatTokens } from '@/utils/format'
import ProcessingTierPricingSummary from './ProcessingTierPricingSummary.vue'

import type { PublicGlobalModel } from '@/api/public-models'
import type { TieredPricingConfig, PricingTier } from '@/api/endpoints/types'

defineProps<Props>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()

const { copyToClipboard } = useClipboard()

interface Props {
  model: PublicGlobalModel | null
  open: boolean
}

function handleClose() {
  emit('update:open', false)
}

function getFirstTierPrice(
  tieredPricing: TieredPricingConfig | undefined | null,
  priceKey: 'input_price_per_1m' | 'output_price_per_1m' | 'cache_creation_price_per_1m' | 'cache_read_price_per_1m'
): string {
  if (!tieredPricing?.tiers?.length) return '-'
  const firstTier = tieredPricing.tiers[0]
  const value = firstTier[priceKey]
  if (value == null || value === 0) return '-'
  return `$${value.toFixed(2)}`
}

function getTierCount(tieredPricing: TieredPricingConfig | undefined | null): number {
  return tieredPricing?.tiers?.length || 0
}

function formatTierLimit(limit: number | null | undefined): string {
  if (limit == null) return ''
  return formatTokens(limit)
}

function get1hCachePrice(tier: PricingTier): string {
  const ttl1h = tier.cache_ttl_pricing?.find(t => t.ttl_minutes === 60)
  if (ttl1h) {
    return `$${ttl1h.cache_creation_price_per_1m.toFixed(2)}`
  }
  return '-'
}

function getFirst1hCachePrice(tieredPricing: TieredPricingConfig | undefined | null): string {
  if (!tieredPricing?.tiers?.length) return '-'
  return get1hCachePrice(tieredPricing.tiers[0])
}

function supportsEmbedding(model: PublicGlobalModel): boolean {
  return model.supports_embedding === true
    || model.supported_capabilities?.includes('embedding') === true
    || model.config?.embedding === true
    || model.config?.model_type === 'embedding'
    || (Array.isArray(model.config?.api_formats) && model.config.api_formats.some((format) => {
      const value = String(format).trim().toLowerCase()
      return value.endsWith(':embedding') || value === 'aliyun:multimodal_embedding'
    }))
}

</script>

