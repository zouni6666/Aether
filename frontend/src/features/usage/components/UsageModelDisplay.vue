<template>
  <div
    class="flex min-w-0 max-w-full flex-col gap-0.5"
    :class="shouldStackBadges && stackFullWidth ? 'w-full items-start' : 'items-start'"
    :data-usage-model-layout="shouldStackBadges ? 'stacked' : 'inline'"
    :data-request-detail-model-layout="context === 'detail'
      ? (shouldStackBadges ? 'stacked' : 'inline')
      : undefined"
  >
    <div
      class="flex min-w-0 max-w-full items-center gap-1"
      :class="modelRowClass"
    >
      <span
        class="min-w-0 truncate"
        :class="modelClass"
        data-usage-model-source
      >{{ record.model }}</span>
      <template v-if="actualModel">
        <span class="shrink-0 text-muted-foreground/70">-&gt;</span>
        <span
          class="min-w-0 truncate"
          :class="modelClass"
          data-usage-model-target
        >{{ actualModel }}</span>
      </template>
      <template v-if="!shouldStackBadges">
        <Badge
          v-for="badge in modelBadges"
          :key="badge.key"
          :data-usage-model-badge="badge.key"
          :data-request-detail-model-badge="context === 'detail' ? badge.key : undefined"
          :variant="badge.variant"
          class="h-4 shrink-0 whitespace-nowrap rounded-full px-1.5 text-[10px] leading-4"
          :class="badge.className"
          :title="badge.title"
          :aria-label="badge.ariaLabel"
        >
          {{ badge.label }}
        </Badge>
      </template>
    </div>

    <div
      v-if="shouldStackBadges && modelBadges.length > 0"
      class="flex min-w-0 max-w-full flex-wrap items-center gap-1"
      data-usage-model-badges-row
      :data-request-detail-model-badges-row="context === 'detail' ? '' : undefined"
    >
      <Badge
        v-for="badge in modelBadges"
        :key="badge.key"
        :data-usage-model-badge="badge.key"
        :data-request-detail-model-badge="context === 'detail' ? badge.key : undefined"
        :variant="badge.variant"
        class="h-4 shrink-0 whitespace-nowrap rounded-full px-1.5 text-[10px] leading-4"
        :class="badge.className"
        :title="badge.title"
        :aria-label="badge.ariaLabel"
      >
        {{ badge.label }}
      </Badge>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { Badge } from '@/components/ui'
import { isCyberPolicyError } from '../utils/cyberError'
import { formatServiceTierFact } from '../utils/service-tier'

type ModelBadgeKey = 'reasoning' | 'fast' | 'cyber'

interface ModelBadgePresentation {
  key: ModelBadgeKey
  label: string
  variant: 'outline' | 'outline-transparent'
  className: string
  title: string
  ariaLabel: string
}

interface UsageModelDisplayRecord {
  model: string
  target_model?: string | null
  model_version?: string | null
  requested_reasoning_effort?: string | null
  reasoning_effort?: string | null
  service_tier?: string | null
  error_message?: string | null
}

const props = withDefaults(defineProps<{
  record: UsageModelDisplayRecord
  modelClass?: string
  modelRowClass?: string
  context?: 'usage' | 'detail'
  cyber?: boolean | null
  stackFullWidth?: boolean
}>(), {
  modelClass: '',
  modelRowClass: '',
  context: 'usage',
  cyber: null,
  stackFullWidth: false,
})

const actualModel = computed(() => {
  const targetModel = normalizeText(props.record.target_model)
  if (targetModel && targetModel !== props.record.model) return targetModel

  const modelVersion = normalizeText(props.record.model_version)
  if (modelVersion && modelVersion !== props.record.model) return modelVersion
  return null
})

const reasoningLabel = computed(() => {
  const requested = normalizeText(props.record.requested_reasoning_effort)
  const actual = normalizeText(props.record.reasoning_effort)
  if (requested && actual && requested.toLowerCase() !== actual.toLowerCase()) {
    return `${requested} -> ${actual}`
  }
  return actual ?? requested
})

const modelBadges = computed<ModelBadgePresentation[]>(() => {
  const badges: ModelBadgePresentation[] = []
  if (reasoningLabel.value) {
    badges.push({
      key: 'reasoning',
      label: reasoningLabel.value,
      variant: 'outline',
      className: 'border-primary/30 bg-primary/5 text-primary',
      title: `Reasoning: ${reasoningLabel.value}`,
      ariaLabel: `Reasoning: ${reasoningLabel.value}`,
    })
  }

  if (formatServiceTierFact(props.record.service_tier) === 'Fast') {
    badges.push({
      key: 'fast',
      label: 'Fast',
      variant: 'outline-transparent',
      className: 'text-amber-700 dark:text-amber-300',
      title: '上游请求档位：Fast\n计费档位：Fast',
      ariaLabel: '上游请求档位：Fast，计费档位：Fast',
    })
  }

  if (props.cyber ?? isCyberPolicyError(props.record.error_message)) {
    badges.push({
      key: 'cyber',
      label: 'Cyber',
      variant: 'outline',
      className: 'border-primary/30 bg-primary/5 text-rose-600 dark:text-rose-300',
      title: '上游 Cyber Policy 拒绝',
      ariaLabel: '上游 Cyber Policy 拒绝',
    })
  }
  return badges
})

const shouldStackBadges = computed(() => (
  actualModel.value !== null || modelBadges.value.length >= 3
))

function normalizeText(value: string | null | undefined): string | null {
  const normalized = value?.trim()
  return normalized || null
}
</script>
