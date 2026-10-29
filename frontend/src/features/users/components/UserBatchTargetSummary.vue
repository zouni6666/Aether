<template>
  <div class="rounded-2xl border border-primary/15 bg-gradient-to-br from-primary/10 via-background to-muted/40 p-4 shadow-sm">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div class="min-w-0 space-y-1">
        <div class="flex items-center gap-2 text-sm font-semibold text-foreground">
          <UsersRound class="h-4 w-4 text-primary" />
          <span>{{ impactLabel }}</span>
        </div>
        <p class="text-xs leading-relaxed text-muted-foreground">
          {{ selectAllFiltered ? t('ui.targetsAllUsersMatchingTheCurrentFiltersTheBackendWill') : t('ui.targetsTheCurrentlySelectedUsersDuplicateIdsAreDeduplicatedAutomatically') }}
        </p>
      </div>
      <Badge
        variant="secondary"
        class="shrink-0"
      >
        {{ selectAllFiltered ? t('ui.selectAllFilteredResults') : t('ui.manualSelection') }}
      </Badge>
    </div>

    <div
      v-if="loading"
      class="mt-3 rounded-xl border border-border/60 bg-background/65 px-3 py-2 text-xs text-muted-foreground"
    >
      {{ t('ui.resolvingImpactScope') }}
    </div>
    <div
      v-else-if="previewItems.length > 0"
      class="mt-3 flex flex-wrap items-center gap-1.5"
    >
      <Badge
        v-for="item in previewItems"
        :key="item.user_id"
        variant="outline"
        class="bg-background/70 text-[11px]"
      >
        {{ item.username }}
      </Badge>
      <span
        v-if="impactCount > previewItems.length"
        class="text-xs text-muted-foreground"
      >
        {{ overflowPreviewLabel }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Badge } from '@/components/ui'
import { UsersRound } from 'lucide-vue-next'
import { useI18n } from '@/i18n'
import type { UserBatchSelectionItem } from '@/api/users'

defineProps<{
  selectAllFiltered: boolean
  impactLabel: string
  impactCount: number
  overflowPreviewLabel: string
  loading: boolean
  previewItems: UserBatchSelectionItem[]
}>()

const { t } = useI18n()
</script>
