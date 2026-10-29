<template>
  <Dialog
    :model-value="open"
    :title="t('ui.remoteConfig')"
    :description="t('ui.changesAreSentToTheAetherTunnelNodeOnIts')"
    :icon="Settings"
    size="md"
    @update:model-value="$emit('update:open', $event)"
  >
    <form
      class="space-y-4"
      @submit.prevent
    >
      <div class="space-y-1.5">
        <Label>{{ t('ui.allowedPorts') }}</Label>
        <Input
          v-model="allowedPorts"
          placeholder="80, 443, 8080, 8443"
        />
        <p class="text-xs text-muted-foreground">
          {{ t('ui.commaSeparatedTargetPortAllowlist') }}
        </p>
      </div>
      <div class="space-y-1.5">
        <Label>{{ t('ui.logLevel') }}</Label>
        <Select v-model="logLevel">
          <SelectTrigger><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="trace">
              trace
            </SelectItem>
            <SelectItem value="debug">
              debug
            </SelectItem>
            <SelectItem value="info">
              info
            </SelectItem>
            <SelectItem value="warn">
              warn
            </SelectItem>
            <SelectItem value="error">
              error
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div class="space-y-1.5">
          <Label>{{ t('ui.heartbeatIntervalSeconds') }}</Label>
          <Input
            v-model="heartbeatInterval"
            type="number"
            min="5"
            max="600"
          />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('ui.schedulingState') }}</Label>
          <Select v-model="schedulingState">
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="active">
                active
              </SelectItem>
              <SelectItem value="draining">
                draining
              </SelectItem>
              <SelectItem value="cordoned">
                cordoned
              </SelectItem>
            </SelectContent>
          </Select>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.drainingAndCordonedBothStopNewTunnelRequestsDrainingIs') }}
          </p>
        </div>
      </div>
      <div class="space-y-1.5">
        <Label>{{ t('ui.upgradeToVersion') }}</Label>
        <Input
          v-model="upgradeTo"
          :placeholder="t('ui.example023')"
        />
        <p class="text-xs text-muted-foreground">
          {{ t('ui.leaveEmptyToClearAnExistingUpgradeCommand') }}
        </p>
      </div>
      <div
        v-if="node"
        class="text-xs text-muted-foreground"
      >
        {{ t('ui.configVersion') }} v{{ node.config_version }}
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
        :disabled="saving"
        @click="$emit('save')"
      >
        {{ saving ? t('ui.saving') : t('ui.save') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Settings } from 'lucide-vue-next'
import { Button, Dialog, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui'
import type { ProxyNode, ProxyNodeSchedulingState } from '@/api/proxy-nodes'
import { useI18n } from '@/i18n'
import type { ProxyNodeConfigForm } from './proxy-node-types'

const props = defineProps<{
  open: boolean
  node: ProxyNode | null
  form: ProxyNodeConfigForm
  saving: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'update:form': [value: ProxyNodeConfigForm]
  save: []
}>()

const { t } = useI18n()

const allowedPorts = computed({
  get: () => props.form.allowed_ports,
  set: (value: string) => emit('update:form', { ...props.form, allowed_ports: value }),
})

const logLevel = computed({
  get: () => props.form.log_level,
  set: (value: string) => emit('update:form', { ...props.form, log_level: value }),
})

const heartbeatInterval = computed({
  get: () => props.form.heartbeat_interval,
  set: (value: string) => emit('update:form', { ...props.form, heartbeat_interval: value }),
})

const schedulingState = computed({
  get: () => props.form.scheduling_state,
  set: (value: string) => emit('update:form', { ...props.form, scheduling_state: value as ProxyNodeSchedulingState }),
})

const upgradeTo = computed({
  get: () => props.form.upgrade_to,
  set: (value: string) => emit('update:form', { ...props.form, upgrade_to: value }),
})
</script>
