<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { Target } from 'lucide-vue-next'

</script>

<template>
  <div class="space-y-12 pb-12">
    <!-- Hero 区域 -->
    <div class="space-y-4">
      <div class="inline-flex items-center gap-1.5 rounded-full bg-[#cc785c]/10 dark:bg-[#cc785c]/20 border border-[#cc785c]/20 dark:border-[#cc785c]/40 px-3 py-1 text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
        <Target class="h-3 w-3" /> {{ t('ui.schedulingAndStrategy') }}
      </div>
      <h1 class="text-3xl font-bold text-[#262624] dark:text-[#f1ead8]">
        {{ t('guide.nav.strategy.description') }}
      </h1>
      <p class="text-base text-[#666663] dark:text-[#a3a094] max-w-2xl">
        {{ t('ui.learnAboutRequestLogsIntelligentSchedulingModesAndServiceLimit') }}
      </p>
    </div>

    <section
      id="request-logging"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.1RequestBodyLogging') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.inSystemSettingsYouCanChangeTheRequestBodyLogging') }}
      </p>
      
      <div class="overflow-hidden rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] bg-white dark:bg-[#191714] shadow-sm max-w-2xl">
        <table class="w-full text-sm text-left">
          <thead class="bg-[#f5f5f0] dark:bg-[rgba(227,224,211,0.05)] border-b border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)]">
            <tr>
              <th
                scope="col"
                class="px-6 py-3 font-medium text-[#262624] dark:text-[#f1ead8]"
              >
                {{ t('ui.logLevel0f8effe') }}
              </th>
              <th
                scope="col"
                class="px-6 py-3 font-medium text-[#262624] dark:text-[#f1ead8]"
              >
                {{ t('ui.recordedContent') }}
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[#e5e4df] dark:divide-[rgba(227,224,211,0.06)]">
            <tr class="hover:bg-black/5 dark:hover:bg-white/5 transition-colors">
              <td class="px-6 py-4 font-mono font-medium text-[#cc785c] dark:text-[#d4a27f]">
                Base
              </td>
              <td class="px-6 py-4 text-[#666663] dark:text-[#a3a094]">
                {{ t('ui.basicRequestInformationIpModelDurationTokensEtc') }}
              </td>
            </tr>
            <tr class="hover:bg-black/5 dark:hover:bg-white/5 transition-colors">
              <td class="px-6 py-4 font-mono font-medium text-[#cc785c] dark:text-[#d4a27f]">
                Headers
              </td>
              <td class="px-6 py-4 text-[#666663] dark:text-[#a3a094]">
                {{ t('ui.baseRequestHeaders') }}
              </td>
            </tr>
            <tr class="hover:bg-black/5 dark:hover:bg-white/5 transition-colors border-b-0">
              <td class="px-6 py-4 font-mono font-medium text-[#cc785c] dark:text-[#d4a27f]">
                Full
              </td>
              <td class="px-6 py-4 text-[#666663] dark:text-[#a3a094]">
                {{ t('ui.headersFullRequestAndResponseBodiesPayloads') }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      
      <img
        loading="lazy"
        src="/guide/strategy-request-logging.webp"
        :alt="t('ui.requestBodyLoggingSettings')"
        class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-6 w-full max-w-3xl"
      >
    </section>

    <section
      id="scheduling"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.2RoutingModes') }}</h2>
      <ul class="list-decimal pl-5 space-y-2 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.providerPriority') }}</strong> {{ t('ui.preferSchedulingAccordingToTheProviderOrder') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.keyPriority') }}</strong> {{ t('ui.ignoreProviderHierarchyAndScheduleDirectlyAcrossAllAvailableKeys') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.cacheAffinityfc256e9') }}</strong> {{ t('ui.tryToRouteTheSameUserRequestsToTheProvider') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.loadBalancing') }}</strong> {{ t('ui.evenlyDistributeTrafficAcrossNodesWithTheSamePriority') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.fixedOrder') }}</strong> {{ t('ui.disableRandomnessAndDynamicAdjustmentThenTryTargetsStrictlyIn') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.failoverb6b2b2c') }}</strong> {{ t('ui.whenARequestFailsAutomaticallySwitchToTheNextAvailable') }}</li>
      </ul>
    </section>

    <section
      id="rate-limit"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.3AccessRestrictions') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.theSystemSupportsMultiDimensionalRateLimitsToPreventMalicious') }}
      </p>
    </section>

    <section
      id="payload-cleanup"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.4RequestBodyCompressionAndCleanup') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.toSaveDatabaseSpaceAndImproveQueryPerformanceTheSystem') }}
      </p>
    </section>

    <section
      id="cron-tasks"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.5ScheduledTasks') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.thePlatformIncludesScheduledTasksForModelListSyncCache') }}
      </p>
    </section>
  </div>
</template>
