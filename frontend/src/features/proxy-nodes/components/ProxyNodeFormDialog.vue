<template>
  <Dialog
    :model-value="open"
    :title="editingNode ? t('ui.editProxyNode') : t('ui.addProxyNode')"
    :description="editingNode ? t('ui.modifyTheManualProxyNodeConfiguration') : t('ui.useTheOneClickScriptToDeployAetherTunnelOr')"
    :icon="editingNode ? SquarePen : Plus"
    size="lg"
    @update:model-value="$emit('update:open', $event)"
  >
    <div
      v-if="!editingNode"
      class="mb-4 grid grid-cols-1 sm:grid-cols-3 gap-2 rounded-lg border border-border/60 bg-muted/30 p-1"
    >
      <Button
        type="button"
        :variant="addMode === 'script' ? 'default' : 'ghost'"
        class="h-9"
        @click="$emit('update:addMode', 'script')"
      >
        <Terminal class="w-3.5 h-3.5 mr-1.5" />
        {{ t('ui.addWithScript') }}
      </Button>
      <Button
        type="button"
        :variant="addMode === 'manual' ? 'default' : 'ghost'"
        class="h-9"
        @click="$emit('update:addMode', 'manual')"
      >
        <Plus class="w-3.5 h-3.5 mr-1.5" />
        {{ t('ui.addManually') }}
      </Button>
      <Button
        type="button"
        :variant="addMode === 'batch' ? 'default' : 'ghost'"
        class="h-9"
        @click="$emit('update:addMode', 'batch')"
      >
        <ListPlus class="w-3.5 h-3.5 mr-1.5" />
        {{ t('ui.batchAdd') }}
      </Button>
    </div>

    <div
      v-if="!editingNode && addMode === 'script'"
      class="space-y-4"
    >
      <div class="rounded-lg border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground">
        {{ t('ui.enterANodeNameToGenerateAOneTimeInstall') }}
      </div>

      <div class="space-y-1.5">
        <Label>{{ t('ui.nodeName') }}</Label>
        <Input
          v-model="installNodeName"
          :placeholder="t('ui.exampleJpProxy01')"
          @keyup.enter="$emit('refresh-install-command')"
        />
      </div>

      <div class="space-y-2">
        <Label class="text-sm font-semibold">{{ t('ui.targetSystem') }}</Label>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
          <Button
            type="button"
            :variant="installSystem === 'unix' ? 'default' : 'outline'"
            class="justify-start h-auto py-3"
            @click="$emit('update:installSystem', 'unix')"
          >
            macOS / Linux
          </Button>
          <Button
            type="button"
            :variant="installSystem === 'windows' ? 'default' : 'outline'"
            class="justify-start h-auto py-3"
            @click="$emit('update:installSystem', 'windows')"
          >
            Windows PowerShell
          </Button>
        </div>
      </div>

      <div class="space-y-2">
        <div class="flex items-center justify-between gap-2">
          <Label class="text-sm font-semibold">{{ t('ui.copyAndRunOnTheProxyNodeMachine') }}</Label>
          <div class="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              class="gap-1.5"
              :disabled="installLoading || !proxyInstallCommand"
              @click="$emit('copy-install-command')"
            >
              <CheckCircle
                v-if="installCopied"
                class="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400"
              />
              <Copy
                v-else
                class="h-3.5 w-3.5"
              />
              {{ installCopied ? t('common.copied') : t('ui.copyCommand') }}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :disabled="installLoading || !installForm.node_name.trim()"
              @click="$emit('refresh-install-command')"
            >
              {{ installLoading ? t('ui.generating') : proxyInstallCommand ? t('ui.regenerate') : t('ui.generateCommand') }}
            </Button>
          </div>
        </div>
        <div class="rounded-lg border border-border/60 bg-background overflow-hidden">
          <pre class="max-h-32 overflow-x-auto whitespace-pre-wrap break-all p-3 text-xs font-mono">{{ proxyInstallCommand || t('ui.enterANodeNameAndClickGenerateCommand') }}</pre>
        </div>
        <p class="text-xs text-muted-foreground">
          {{ proxyInstallHint }}
        </p>
      </div>
    </div>

    <div
      v-else-if="!editingNode && addMode === 'batch'"
      class="space-y-4"
    >
      <div class="rounded-lg border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground">
        {{ t('ui.supportsOneEntryPerLineOrCommaSeparatedEntriesUsernames') }}
      </div>

      <div class="space-y-1.5">
        <Label>{{ t('ui.proxyAddress') }}</Label>
        <Textarea
          v-model="batchContent"
          class="min-h-[180px] font-mono text-xs break-all !rounded-xl"
          placeholder="socks5://username:password@1.2.3.4:1080&#10;http://username:password@5.6.7.8:8080"
        />
      </div>

      <div
        v-if="batchParseResult.errors.length"
        class="rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive"
      >
        <div class="font-medium">
          {{ t('ui.valueFormatErrors', { count: batchParseResult.errors.length }) }}
        </div>
        <ul class="mt-1 space-y-1">
          <li
            v-for="message in batchParseResult.errors.slice(0, 3)"
            :key="message"
          >
            {{ message }}
          </li>
        </ul>
        <div
          v-if="batchParseResult.errors.length > 3"
          class="mt-1 text-destructive/80"
        >
          {{ t('ui.valueMoreErrorsHidden', { count: batchParseResult.errors.length - 3 }) }}
        </div>
      </div>
      <p
        v-else-if="batchForm.content.trim()"
        class="text-xs text-muted-foreground"
      >
        {{ t('ui.valueProxyNodesRecognized', { count: batchParseResult.nodes.length }) }}
      </p>
    </div>

    <form
      v-else
      class="space-y-4"
      @submit.prevent="$emit('submit-manual')"
    >
      <div class="space-y-1.5">
        <Label>{{ t('ui.name') }}</Label>
        <Input
          v-model="manualName"
          :placeholder="t('ui.exampleUsWestVpnProxy')"
        />
      </div>
      <div class="space-y-1.5">
        <Label>{{ t('ui.proxyAddress') }}</Label>
        <Input
          v-model="manualProxyUrl"
          :placeholder="t('ui.httpProxyPortOrSocks5ProxyPort')"
        />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1.5">
          <Label>{{ t('auth.register.username') }}</Label>
          <Input
            v-model="manualUsername"
            :placeholder="t('ui.optional')"
            autocomplete="off"
            data-form-type="other"
            data-lpignore="true"
            data-1p-ignore="true"
          />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('auth.register.password') }}</Label>
          <Input
            v-model="manualPassword"
            type="text"
            masked
            :placeholder="t('ui.optional')"
            autocomplete="new-password"
            data-form-type="other"
            data-lpignore="true"
            data-1p-ignore="true"
          />
        </div>
      </div>
      <div class="space-y-1.5">
        <Label>{{ t('ui.region') }}</Label>
        <Input
          v-model="manualRegion"
          :placeholder="t('ui.optionalEGUsWest')"
        />
      </div>
    </form>

    <template #footer>
      <div
        v-if="!editingNode && addMode === 'script'"
        class="flex items-center justify-end gap-2 w-full"
      >
        <Button
          variant="outline"
          @click="$emit('update:open', false)"
        >
          {{ t('ui.close') }}
        </Button>
        <Button
          :disabled="installLoading || !proxyInstallCommand"
          @click="$emit('copy-install-command')"
        >
          {{ installCopied ? t('common.copied') : t('ui.copyCommand605c82d') }}
        </Button>
      </div>
      <div
        v-else-if="!editingNode && addMode === 'batch'"
        class="flex items-center justify-between gap-3 w-full"
      >
        <span class="text-xs text-muted-foreground">
          {{ batchForm.content.trim() ? t('ui.valuePending', { count: batchParseResult.nodes.length }) : t('ui.waitingForProxyAddresses') }}
        </span>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            @click="$emit('update:open', false)"
          >
            {{ t('common.cancel') }}
          </Button>
          <Button
            :disabled="addingNode || !batchForm.content.trim() || batchParseResult.errors.length > 0 || batchParseResult.nodes.length === 0"
            @click="$emit('submit-batch')"
          >
            {{ addingNode ? t('ui.adding') : t('ui.batchAdd') }}
          </Button>
        </div>
      </div>
      <div
        v-else
        class="flex items-center justify-between w-full"
      >
        <Button
          variant="outline"
          :disabled="testingUrl || !addForm.proxy_url"
          @click="$emit('test-url')"
        >
          {{ testingUrl ? t('ui.testing7b6187c') : t('ui.test') }}
        </Button>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            @click="$emit('update:open', false)"
          >
            {{ t('common.cancel') }}
          </Button>
          <Button
            :disabled="addingNode || !addForm.name || !addForm.proxy_url"
            @click="$emit('submit-manual')"
          >
            {{ submitLabel }}
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircle, Copy, ListPlus, Plus, SquarePen, Terminal } from 'lucide-vue-next'
import { Button, Dialog, Input, Label, Textarea } from '@/components/ui'
import type { ProxyNode } from '@/api/proxy-nodes'
import { useI18n } from '@/i18n'
import type { BatchProxyNodeParseResult } from '../utils/proxy-node-batch'
import type {
  ProxyNodeAddMode,
  ProxyNodeBatchForm,
  ProxyNodeInstallForm,
  ProxyNodeInstallSystem,
  ProxyNodeManualForm,
} from './proxy-node-types'

const props = defineProps<{
  open: boolean
  editingNode: ProxyNode | null
  addMode: ProxyNodeAddMode
  addForm: ProxyNodeManualForm
  batchForm: ProxyNodeBatchForm
  installForm: ProxyNodeInstallForm
  installSystem: ProxyNodeInstallSystem
  installLoading: boolean
  installCopied: boolean
  proxyInstallCommand: string
  proxyInstallHint: string
  batchParseResult: BatchProxyNodeParseResult
  addingNode: boolean
  testingUrl: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'update:addMode': [value: ProxyNodeAddMode]
  'update:addForm': [value: ProxyNodeManualForm]
  'update:batchForm': [value: ProxyNodeBatchForm]
  'update:installForm': [value: ProxyNodeInstallForm]
  'update:installSystem': [value: ProxyNodeInstallSystem]
  'refresh-install-command': []
  'copy-install-command': []
  'submit-manual': []
  'submit-batch': []
  'test-url': []
}>()

const { t } = useI18n()

const installNodeName = computed({
  get: () => props.installForm.node_name,
  set: (value: string) => emit('update:installForm', { ...props.installForm, node_name: value }),
})

const batchContent = computed({
  get: () => props.batchForm.content,
  set: (value: string) => emit('update:batchForm', { ...props.batchForm, content: value }),
})

const manualName = computed({
  get: () => props.addForm.name,
  set: (value: string) => emit('update:addForm', { ...props.addForm, name: value }),
})

const manualProxyUrl = computed({
  get: () => props.addForm.proxy_url,
  set: (value: string) => emit('update:addForm', { ...props.addForm, proxy_url: value }),
})

const manualUsername = computed({
  get: () => props.addForm.username,
  set: (value: string) => emit('update:addForm', { ...props.addForm, username: value }),
})

const manualPassword = computed({
  get: () => props.addForm.password,
  set: (value: string) => emit('update:addForm', { ...props.addForm, password: value }),
})

const manualRegion = computed({
  get: () => props.addForm.region,
  set: (value: string) => emit('update:addForm', { ...props.addForm, region: value }),
})

const submitLabel = computed(() => {
  if (props.addingNode) {
    return props.editingNode ? t('ui.saving') : t('ui.adding')
  }
  return props.editingNode ? t('ui.save') : t('ui.add')
})
</script>
