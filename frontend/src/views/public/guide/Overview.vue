<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { ref } from 'vue'
import {
  Code,
  Server,
  Key,
  Container,
  Shield,
  Monitor,
  Check,
  Copy,
  Zap,
} from 'lucide-vue-next'
import { panelClasses } from './guide-config'
import { useClipboard } from '@/composables/useClipboard'

// 部署步骤数据
const activeDeployTab = ref(0)
const copiedStep = ref<string | null>(null)

const productionSteps = [
  {
    get title() { return t('ui.cloneRepository') },
    code: 'git clone https://github.com/fawney19/Aether.git\ncd Aether',
    icon: Code
  },
  {
    get title() { return t('ui.configureEnvironmentVariables') },
    note: '生成密钥并填入 .env',
    code: 'cp .env.example .env\npython generate_keys.py',
    icon: Key
  },
  {
    get title() { return t('ui.deployUpdate') },
    note: '自动执行数据库迁移',
    code: 'docker compose pull && docker compose up -d',
    icon: Container
  },
  {
    get title() { return t('ui.backupBeforeUpgrade') },
    note: '可选',
    code: 'docker compose exec postgres pg_dump -U postgres aether | gzip > backup_$(date +%Y%m%d_%H%M%S).sql.gz',
    icon: Shield,
    optional: true
  }
]

const developmentSteps = [
  {
    get title() { return t('ui.startDependencies') },
    note: '可选，make dev 会自动启动',
    code: 'docker compose up -d postgres redis',
    icon: Container
  },
  {
    get title() { return t('ui.installFrontendDependencies') },
    note: '首次本地开发',
    code: '(cd frontend && npm install)',
    icon: Server
  },
  {
    get title() { return t('ui.startDevelopmentServices') },
    note: '后端 + Vite dev server',
    code: 'make dev',
    icon: Monitor
  }
]

const { copyToClipboard } = useClipboard()

async function copyStep(stepId: string, code: string) {
  if (!(await copyToClipboard(code, false))) return
  copiedStep.value = stepId
  setTimeout(() => {
    copiedStep.value = null
  }, 2000)
}

</script>

<template>
  <div class="space-y-12">
    <!-- Hero 区域 -->
    <div class="space-y-4">
      <div class="inline-flex items-center gap-1.5 rounded-full bg-[#cc785c]/10 dark:bg-[#cc785c]/20 border border-[#cc785c]/20 dark:border-[#cc785c]/40 px-3 py-1 text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
        <Zap class="h-3 w-3" /> {{ t('ui.aetherDocumentation') }}
      </div>
      <h1 class="text-3xl font-bold text-[#262624] dark:text-[#f1ead8]">
        {{ t('guide.nav.overview') }}
      </h1>
      <p class="text-base text-[#666663] dark:text-[#a3a094] max-w-2xl">
        {{ t('ui.thisGuideWalksYouThroughAetherDeploymentConfigurationReverseProxying') }}
      </p>
    </div>

    <!-- 1. 项目部署 -->
    <section
      id="production"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.1Deployment') }}</h2>
      
      <div
        :class="[panelClasses.card]"
        class="mt-6"
      >
        <!-- Tab 切换 -->
        <div class="flex max-w-full overflow-x-auto border-b border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] px-5">
          <button
            v-for="(tab, idx) in [
              { icon: Container, label: 'Docker 预构建镜像' },
              { icon: Monitor, label: '本地开发' }
            ]"
            :key="idx"
            class="flex shrink-0 items-center gap-2 px-4 py-3 text-sm font-medium whitespace-nowrap transition-colors border-b-2 -mb-px hover:text-[#262624] dark:hover:text-[#f1ead8]"
            :class="activeDeployTab === idx
              ? 'border-[#cc785c] text-[#cc785c] dark:text-[#d4a27f]'
              : 'border-transparent text-[#666663] dark:text-[#a3a094]'"
            @click="activeDeployTab = idx"
          >
            <component
              :is="tab.icon"
              class="h-4 w-4 shrink-0"
            />
            {{ tab.label }}
          </button>
        </div>

        <!-- 生产环境步骤 -->
        <div
          v-show="activeDeployTab === 0"
          class="p-5 space-y-3"
        >
          <div
            v-for="(step, idx) in productionSteps"
            :key="idx"
            class="group rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] overflow-hidden transition-colors"
            :class="step.optional ? 'border-dashed opacity-80' : ''"
          >
            <div class="flex items-center gap-3 px-4 py-3">
              <span class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0 bg-[#cc785c] text-white">
                {{ idx + 1 }}
              </span>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-[#262624] dark:text-[#f1ead8]">{{ step.title }}</span>
                  <span
                    v-if="step.optional"
                    class="text-[10px] px-1.5 py-0.5 rounded-full bg-[#e5e4df] dark:bg-[rgba(227,224,211,0.12)] text-[#666663] dark:text-[#a3a094]"
                  > {{ t('ui.optional') }} </span>
                </div>
                <span
                  v-if="step.note"
                  class="text-xs text-[#91918d] dark:text-[#a3a094]/80"
                >{{ step.note }}</span>
              </div>
              <button
                class="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs text-[#666663] dark:text-[#a3a094] hover:bg-[#f0f0eb] dark:hover:bg-[#3a3731] transition-colors shrink-0"
                @click="copyStep(`prod-${idx}`, step.code)"
              >
                <Check
                  v-if="copiedStep === `prod-${idx}`"
                  class="h-3.5 w-3.5 text-green-500"
                />
                <Copy
                  v-else
                  class="h-3.5 w-3.5"
                />
                {{ copiedStep === `prod-${idx}` ? t('common.copied') : t('common.copy') }}
              </button>
            </div>
            <pre class="px-4 pb-3 text-[13px] font-mono text-[#262624] dark:text-[#f1ead8] overflow-x-auto leading-relaxed border-t border-[#e5e4df]/50 dark:border-[rgba(227,224,211,0.06)] pt-3 mx-4 mb-1"><code>{{ step.code }}</code></pre>
          </div>
        </div>

        <!-- 开发环境步骤 -->
        <div
          v-show="activeDeployTab === 1"
          class="p-5 space-y-3"
        >
          <div
            v-for="(step, idx) in developmentSteps"
            :key="idx"
            class="group rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] overflow-hidden transition-colors"
          >
            <div class="flex items-center gap-3 px-4 py-3">
              <span class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0 bg-[#cc785c] text-white">
                {{ idx + 1 }}
              </span>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-[#262624] dark:text-[#f1ead8]">{{ step.title }}</span>
                </div>
                <span
                  v-if="step.note"
                  class="text-xs text-[#91918d] dark:text-[#a3a094]/80"
                >{{ step.note }}</span>
              </div>
              <button
                class="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs text-[#666663] dark:text-[#a3a094] hover:bg-[#f0f0eb] dark:hover:bg-[#3a3731] transition-colors shrink-0"
                @click="copyStep(`dev-${idx}`, step.code)"
              >
                <Check
                  v-if="copiedStep === `dev-${idx}`"
                  class="h-3.5 w-3.5 text-green-500"
                />
                <Copy
                  v-else
                  class="h-3.5 w-3.5"
                />
                {{ copiedStep === `dev-${idx}` ? t('common.copied') : t('common.copy') }}
              </button>
            </div>
            <pre class="px-4 pb-3 text-[13px] font-mono text-[#262624] dark:text-[#f1ead8] overflow-x-auto leading-relaxed border-t border-[#e5e4df]/50 dark:border-[rgba(227,224,211,0.06)] pt-3 mx-4 mb-1"><code>{{ step.code }}</code></pre>
          </div>
        </div>
      </div>
    </section>

    <!-- 2. 配置流程 -->
    <section
      id="config-steps"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.2ConfigurationWorkflow') }}</h2>

      <div class="space-y-8 mt-6">
        <div>
          <h3>{{ t('ui.1CreateAUnifiedModel') }}</h3>
          <p>{{ t('ui.usingOpus46AsAnExampleAddOtherModels') }}</p>
          <img
            loading="lazy"
            src="/guide/quickstart-create-model.webp"
            :alt="t('guide.nav.concepts.createModel')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
          >
        </div>

        <div>
          <h3>{{ t('ui.2AddAProvider') }}</h3>
          <img
            loading="lazy"
            src="/guide/quickstart-add-provider.webp"
            :alt="t('guide.nav.concepts.addProvider')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
          >
        </div>

        <div>
          <h3>{{ t('ui.3AddAnEndpoint') }}</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
            <img
              loading="lazy"
              src="/guide/quickstart-add-endpoint-1.webp"
              :alt="t('ui.addAnEndpointStep1')"
              class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
            >
            <img
              loading="lazy"
              src="/guide/quickstart-add-endpoint-2.webp"
              :alt="t('ui.addAnEndpointStep2')"
              class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
            >
          </div>
        </div>

        <div>
          <h3>{{ t('ui.4AddAnApiKey') }}</h3>
          <img
            loading="lazy"
            src="/guide/quickstart-add-key.webp"
            :alt="t('guide.nav.concepts.addKey')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
          >
        </div>

        <div>
          <h3>{{ t('ui.5LinkAGlobalModel') }}</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
            <img
              loading="lazy"
              src="/guide/quickstart-link-model-1.webp"
              :alt="t('ui.linkAGlobalModelStep1')"
              class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
            >
            <img
              loading="lazy"
              src="/guide/quickstart-link-model-2.webp"
              :alt="t('ui.linkAGlobalModelStep2')"
              class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
            >
          </div>
        </div>

        <div>
          <h3>{{ t('ui.6ModelMapping') }}</h3>
          <img
            loading="lazy"
            src="/guide/quickstart-model-mapping.webp"
            :alt="t('guide.nav.concepts.modelMapping')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
          >
        </div>
      </div>
    </section>

    <!-- 3. 反向代理 -->
    <section
      id="reverse-proxy"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.3ReverseProxy') }}</h2>
      <p>{{ t('ui.whenAddingAProviderChooseTheMatchingProviderTypeReverse') }}</p>

      <ul class="list-decimal pl-5 space-y-4 mt-4 text-[#666663] dark:text-[#a3a094]">
        <li>
          <span class="font-medium text-[#262624] dark:text-[#f1ead8]">Codex</span>
          <ul class="list-disc pl-5 mt-2 space-y-1">
            <li>{{ t('ui.oauthAuthorizationLogin') }}</li>
            <li>{{ t('ui.importRefreshtokenWithBatchImportSupport') }}</li>
          </ul>
        </li>
        <li>
          <span class="font-medium text-[#262624] dark:text-[#f1ead8]">Kiro</span>
          <ul class="list-disc pl-5 mt-2 space-y-2">
            <li>Build ID</li>
            <li>Identity Center (Start URL, Region)</li>
            <li>
              {{ t('ui.importRefreshtokenWithBatchImportSupport098b3e4') }} <div
                :class="[panelClasses.commandPanel]"
                class="mt-3"
              >
                <div class="px-4 py-2 border-b border-[#e5e4df]/50 dark:border-[rgba(227,224,211,0.06)] bg-[#f5f5f0]/50 dark:bg-[rgba(227,224,211,0.05)] text-xs text-[#91918d] dark:text-[#a3a094]/70">
                  {{ t('ui.socialFormatRequirements') }}
                </div>
                <pre class="px-4 py-3 text-[13px] font-mono text-[#262624] dark:text-[#f1ead8] overflow-x-auto m-0 bg-transparent"><code>{
  "refresh_token": ""
}</code></pre>
              </div>
              <div
                :class="[panelClasses.commandPanel]"
                class="mt-3"
              >
                <div class="px-4 py-2 border-b border-[#e5e4df]/50 dark:border-[rgba(227,224,211,0.06)] bg-[#f5f5f0]/50 dark:bg-[rgba(227,224,211,0.05)] text-xs text-[#91918d] dark:text-[#a3a094]/70">
                  {{ t('ui.idcFormatRequirements') }}
                </div>
                <pre class="px-4 py-3 text-[13px] font-mono text-[#262624] dark:text-[#f1ead8] overflow-x-auto m-0 bg-transparent"><code>{
  "refresh_token": "",
  "client_id": "",
  "client_secret": "",
  "machine_id": ""
}</code></pre>
              </div>
            </li>
          </ul>
        </li>
        <li>
          <span class="font-medium text-[#262624] dark:text-[#f1ead8]">Antigravity</span>
          <ul class="list-disc pl-5 mt-2 space-y-1">
            <li>{{ t('ui.oauthAuthorizationLogin') }}</li>
            <li>{{ t('ui.importRefreshtokenWithBatchImportSupport') }}</li>
          </ul>
        </li>
      </ul>

      <img
        loading="lazy"
        src="/guide/quickstart-reverse-proxy.webp"
        :alt="t('ui.reverseProxyConfigurationExample')"
        class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-6 w-full max-w-2xl"
      >
    </section>

    <!-- 4. 异步任务 -->
    <section
      id="async-tasks"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.4AsynchronousTasks') }}</h2>
      <p>{{ t('ui.requiresProviderEndpointSupport') }}</p>
      
      <ul class="list-decimal pl-5 mt-4 text-[#666663] dark:text-[#a3a094] space-y-1">
        <li><span class="font-medium text-[#262624] dark:text-[#f1ead8]">Veo</span></li>
        <li><span class="font-medium text-[#262624] dark:text-[#f1ead8]">Sora</span></li>
      </ul>
    </section>

    <!-- 5. 代理配置 -->
    <section
      id="proxy-config"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.5ProxyConfiguration') }}</h2>

      <div class="space-y-6 mt-6">
        <div>
          <h3>1. Aether-Proxy</h3>
          <p>{{ t('ui.implementedInRustWithVeryLowResourceUsageSuitableFor') }}</p>
          <a
            href="https://github.com/fawney19/Aether/tree/main/aether-tunnel"
            target="_blank"
            rel="noopener noreferrer"
            class="text-[#cc785c] dark:text-[#d4a27f] hover:underline mt-2 inline-block"
          > {{ t('ui.githubRepository') }} </a>
        </div>

        <div>
          <h3>{{ t('ui.2ProxyNodes') }}</h3>
          <p>{{ t('ui.afterEnablingTheProxyModuleInModuleManagementYouCan') }}</p>
        </div>

        <div>
          <h3>{{ t('ui.3ProxyHierarchy') }}</h3>
          <p>{{ t('ui.prioritye097ced') }}<span class="text-[#262624] dark:text-[#f1ead8] font-medium bg-[#cc785c]/10 px-2 py-0.5 rounded">{{ t('ui.keyProxy') }}</span> > <span class="text-[#262624] dark:text-[#f1ead8] font-medium bg-[#cc785c]/10 px-2 py-0.5 rounded">{{ t('ui.providerProxy') }}</span> > <span class="text-[#262624] dark:text-[#f1ead8] font-medium bg-[#cc785c]/10 px-2 py-0.5 rounded">{{ t('ui.globalProxy') }}</span></p>
          
          <ul class="list-decimal pl-5 mt-4 space-y-2 text-[#666663] dark:text-[#a3a094]">
            <li>{{ t('ui.globalProxySystemSettings') }}</li>
            <li>{{ t('ui.providerProxyProviderSettings') }}</li>
            <li>{{ t('ui.keyProxyKeySettings') }}</li>
          </ul>
        </div>
      </div>
    </section>
  </div>
</template>
