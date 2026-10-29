<template>
  <Dialog
    :model-value="open"
    :title="t('ui.batchUpgrade')"
    :description="t('ui.writeAnUpgradeTargetToAllTunnelNodesNodesPick')"
    :icon="Settings"
    size="sm"
    @update:model-value="$emit('update:open', $event)"
  >
    <form
      class="space-y-4"
      @submit.prevent="$emit('submit')"
    >
      <div class="space-y-1.5">
        <Label>{{ t('update.version.target') }}</Label>
        <Input
          :model-value="version"
          :placeholder="t('ui.example023')"
          @update:model-value="$emit('update:version', String($event))"
        />
        <p class="text-xs text-muted-foreground">
          {{ t('ui.theGatewayOnlyWritesTheUpgradeToTargetVersionAnd') }}
        </p>
      </div>
    </form>
    <template #footer>
      <Button
        variant="outline"
        @click="$emit('update:open', false)"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        :disabled="upgrading || !version.trim()"
        @click="$emit('submit')"
      >
        {{ upgrading ? t('ui.dispatching') : t('ui.confirmDispatch') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import { Button, Dialog, Input, Label } from '@/components/ui'
import { useI18n } from '@/i18n'

defineProps<{
  open: boolean
  version: string
  upgrading: boolean
}>()

defineEmits<{
  'update:open': [value: boolean]
  'update:version': [value: string]
  submit: []
}>()

const { t } = useI18n()
</script>
