<template>
  <div :class="mobile ? 'grid grid-cols-2 gap-2 pt-0.5' : 'flex justify-center gap-1'">
    <Button
      v-if="canOperateAdmin"
      :variant="mobile ? 'outline' : 'ghost'"
      :size="mobile ? 'sm' : 'icon'"
      :class="mobile ? 'h-8 text-xs' : 'h-8 w-8'"
      :title="t('ui.editUser')"
      @click="$emit('edit')"
    >
      <SquarePen :class="iconClass" />
      <span v-if="mobile">{{ t('ui.edit0518365') }}</span>
    </Button>
    <Button
      v-if="canOperateAdmin"
      :variant="mobile ? 'outline' : 'ghost'"
      :size="mobile ? 'sm' : 'icon'"
      :class="mobile ? 'h-8 text-xs' : 'h-8 w-8'"
      :title="t('ui.walletOperation')"
      @click="$emit('wallet')"
    >
      <DollarSign :class="iconClass" />
      <span v-if="mobile">{{ t('ui.wallet4cd1c82') }}</span>
    </Button>
    <Button
      v-if="canOperateAdmin"
      :variant="mobile ? 'outline' : 'ghost'"
      :size="mobile ? 'sm' : 'icon'"
      :class="mobile ? 'h-8 text-xs' : 'h-8 w-8'"
      :title="t('ui.plan')"
      @click="$emit('plans')"
    >
      <PackageCheck :class="iconClass" />
      <span v-if="mobile">{{ t('ui.plan') }}</span>
    </Button>
    <Button
      :variant="mobile ? 'outline' : 'ghost'"
      :size="mobile ? 'sm' : 'icon'"
      :class="mobile ? 'h-8 text-xs' : 'h-8 w-8'"
      title="API Keys"
      @click="$emit('api-keys')"
    >
      <Key :class="iconClass" />
      <span v-if="mobile">API Keys</span>
    </Button>
    <Button
      v-if="canOperateAdmin"
      :variant="mobile ? 'outline' : 'ghost'"
      :size="mobile ? 'sm' : 'icon'"
      :class="mobile ? 'h-8 text-xs' : 'h-8 w-8'"
      :title="t('ui.signedInDevices')"
      @click="$emit('sessions')"
    >
      <MonitorSmartphone :class="iconClass" />
      <span v-if="mobile">{{ t('ui.devices') }}</span>
    </Button>
    <Button
      :variant="mobile ? 'outline' : 'ghost'"
      :size="mobile ? 'sm' : 'icon'"
      :class="mobile ? 'h-8 text-xs' : 'h-8 w-8'"
      :title="isActive ? t('ui.disableUser') : t('ui.enableUser')"
      @click="$emit('toggle-status')"
    >
      <PauseCircle
        v-if="isActive"
        :class="iconClass"
      />
      <PlayCircle
        v-else
        :class="iconClass"
      />
      <span v-if="mobile">{{ isActive ? t('ui.disable') : t('ui.enablef4f0ead') }}</span>
    </Button>
    <Button
      :variant="mobile ? 'outline' : 'ghost'"
      :size="mobile ? 'sm' : 'icon'"
      :class="mobile ? 'col-span-2 h-8 border-rose-200 text-xs text-rose-600 hover:bg-rose-50 dark:border-rose-900/60 dark:hover:bg-rose-950/40' : 'h-8 w-8'"
      :title="t('ui.deleteUser')"
      @click="$emit('delete')"
    >
      <Trash2 :class="iconClass" />
      <span v-if="mobile">{{ t('ui.delete') }}</span>
    </Button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  DollarSign,
  Key,
  MonitorSmartphone,
  PackageCheck,
  PauseCircle,
  PlayCircle,
  SquarePen,
  Trash2,
} from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import { useI18n } from '@/i18n'

const props = withDefaults(defineProps<{
  canOperateAdmin: boolean
  isActive: boolean
  mobile?: boolean
}>(), {
  mobile: false,
})

defineEmits<{
  edit: []
  wallet: []
  plans: []
  'api-keys': []
  sessions: []
  'toggle-status': []
  delete: []
}>()

const { t } = useI18n()
const iconClass = computed(() => props.mobile ? 'mr-1.5 h-3.5 w-3.5' : 'h-4 w-4')
</script>
