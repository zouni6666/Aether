<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { BookOpen } from 'lucide-vue-next'

</script>

<template>
  <div class="space-y-12">
    <!-- Hero 区域 -->
    <div class="space-y-4">
      <div class="inline-flex items-center gap-1.5 rounded-full bg-[#cc785c]/10 dark:bg-[#cc785c]/20 border border-[#cc785c]/20 dark:border-[#cc785c]/40 px-3 py-1 text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
        <BookOpen class="h-3 w-3" /> {{ t('ui.documentationCore') }}
      </div>
      <h1 class="text-3xl font-bold text-[#262624] dark:text-[#f1ead8]">
        {{ t('guide.nav.concepts') }}
      </h1>
      <p class="text-base text-[#666663] dark:text-[#a3a094] max-w-2xl">
        {{ t('ui.understandTheCoreModulesAndHierarchyOfTheAetherRuntime') }}
      </p>
    </div>

    <!-- 1. 创建统一模型 -->
    <section
      id="create-model"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.1CreateAUnifiedModel') }}</h2>
      <div class="space-y-4 mt-4">
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8]">
            {{ t('ui.1LeftModelSelectionAndSearchArea') }}
          </h4>
          <ul class="list-disc pl-5 mt-2 space-y-1 text-sm text-[#666663] dark:text-[#a3a094]">
            <li>{{ t('ui.modelSourceHttpsModelsDevMaintainedByOpencode') }}</li>
            <li>{{ t('ui.onlyAFewOfficialModelProvidersAreShownSearchTo') }}</li>
          </ul>
        </div>
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8]">
            {{ t('ui.2RightSideParameterConfigurationArea') }}
          </h4>
          <ul class="list-disc pl-5 mt-2 space-y-1 text-sm text-[#666663] dark:text-[#a3a094]">
            <li>{{ t('ui.addAModelManuallyOrAutoFillAfterSelectingOne') }}</li>
            <li>{{ t('ui.modelPreferencesMarkWhetherThisModelSupportsTheseCapabilitiesUsually') }}</li>
            <li>
              {{ t('ui.pricingConfiguration') }} <ul class="list-[circle] pl-5 mt-1 space-y-1">
                <li>{{ t('ui.billingMethodTokenPricePerRequestPriceVideoBillingFinal') }}</li>
                <li>{{ t('ui.priceTiersUseTheTotalTokenRangeNotTieredOverflow') }}</li>
              </ul>
            </li>
          </ul>
        </div>
        <img
          loading="lazy"
          src="/guide/concepts-create-model.webp"
          :alt="t('guide.nav.concepts.createModel')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
        >
      </div>
    </section>

    <!-- 2. 添加提供商 -->
    <section
      id="add-provider"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.2AddAProvider') }}</h2>
      <div class="space-y-4 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <ul class="list-decimal pl-5 space-y-2">
          <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.providerType') }}</strong>{{ t('ui.customOrReverseProxyCustomIsUsuallyEnoughUseThe') }}</li>
          <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.maximumRetries') }}</strong>{{ t('ui.retryCountAfterTheFirstRequestFailureInCacheAffinity') }}</li>
          <li>
            <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.timeout7ec14f5') }}</strong>
            <ul class="list-disc pl-5 mt-1 space-y-1">
              <li>{{ t('ui.streamingFirstByteTimeoutTimeoutBeforeTheFirstByteIs') }}</li>
              <li>{{ t('ui.nonStreamingRequestTimeoutTotalTimeoutForNonStreamingRequests') }}</li>
            </ul>
          </li>
          <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.preservePriority') }}</strong>{{ t('ui.whetherRequestsUsingFormatConversionKeepTheCurrentPriority') }}</li>
        </ul>
        <img
          loading="lazy"
          src="/guide/concepts-add-provider.webp"
          :alt="t('guide.nav.concepts.addProvider')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
        >
      </div>
    </section>

    <!-- 3. 添加端点 -->
    <section
      id="add-endpoint"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.3AddAnEndpoint') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.addingAnEndpointMeansAddingAnUpstreamSupportedEndpointNot') }}
      </p>
      <ul class="list-decimal pl-5 space-y-2 text-[#666663] dark:text-[#a3a094] text-sm">
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.selectFormat90634d2') }}</strong>{{ t('ui.selectAnEndpointFormatSupportedByTheUpstreamService') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.customBaseUrlPath') }}</strong>{{ t('ui.fillInTheInformationProvidedByTheUpstreamServiceLeave') }}</li>
      </ul>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-add-endpoint-1.webp"
          :alt="t('ui.selectAFormatWhenAddingAnEndpoint')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <img
          loading="lazy"
          src="/guide/concepts-add-endpoint-2.webp"
          :alt="t('ui.endpointCustomization')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
      </div>
    </section>

    <!-- 4. 添加密钥 -->
    <section
      id="add-key"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.4AddAnApiKey') }}</h2>
      <ul class="list-decimal pl-5 space-y-3 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.authenticationType') }}</strong>
          <ul class="list-disc pl-5 mt-1">
            <li>{{ t('ui.apiKeyTraditionalKey') }}</li>
            <li>{{ t('ui.vertexAiForGoogleCloud') }}</li>
          </ul>
        </li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.supportedApiFormats') }}</strong>{{ t('ui.selectTheApiFormatsThisKeyCanUseAndSet') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.prioritye097ced') }}</strong>{{ t('ui.inProviderFirstSchedulingModeKeysWithDifferentPrioritiesFail') }}</li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.rpmLimit') }}</strong>{{ t('ui.keyUsageRateLimitTenPercentIsReservedForExisting') }} <ul class="list-disc pl-5 mt-1">
            <li>{{ t('ui.enterAFixedValue') }}</li>
            <li>{{ t('ui.leaveEmptyToLetTheAdaptivePolicyLearnAutomatically') }}</li>
          </ul>
        </li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.cacheTtl') }}</strong>
          <ul class="list-disc pl-5 mt-1">
            <li>{{ t('ui.0DoNotPrioritizeCacheUsage') }}</li>
            <li>{{ t('ui.nWithinNMinutesTheSameUserKeyPrefersThe') }}</li>
          </ul>
        </li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.circuitBreakerProbes') }}</strong><br> {{ t('ui.whenTheSameProviderKeyFailsSeveralConsecutiveRequestsIt') }}
        </li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('ui.automaticallyFetchUpstreamModels98ab0e1') }}</strong><br> {{ t('ui.whenTheUpstreamModelListEndpointIsSupportedFetchAvailable') }}
        </li>
      </ul>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-add-key-1.webp"
          :alt="t('ui.configureKeyAuthentication')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <img
          loading="lazy"
          src="/guide/concepts-add-key-2.webp"
          :alt="t('ui.keyMultiplierAndPriority')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
      </div>
    </section>

    <!-- 5. 模型权限 -->
    <section
      id="model-permission"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.5ModelPermissions') }}</h2>
      <ul class="list-decimal pl-5 space-y-2 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <li>{{ t('ui.enableAutomaticFetchRefreshOfAllUpstreamModelsWhileEditing') }}</li>
        <li>{{ t('ui.manuallyRestrictOrCreateAvailableModelsForThisKey') }}</li>
        <li>{{ t('ui.ifAutomaticFetchIsDisabledAndNoSpecificModelIs') }}</li>
        <li>{{ t('ui.regexModelMappingCanBeUsedOnlyWhenModelPermissions') }}</li>
      </ul>
      <div class="grid grid-cols-1 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-model-perms-1.webp"
          :alt="t('ui.modelPermissionsStep1')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <img
            loading="lazy"
            src="/guide/concepts-model-perms-2.webp"
            :alt="t('ui.modelPermissionsStep2')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
          <img
            loading="lazy"
            src="/guide/concepts-model-perms-3.webp"
            :alt="t('ui.modelPermissionsStep3')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
        </div>
      </div>
    </section>

    <!-- 6. 关联模型 -->
    <section
      id="link-model"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.6LinkModels') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.linkProviderModelsFromGlobalModelsToDefineWhichModels') }}
      </p>
      <div class="grid grid-cols-1 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-link-model-1.webp"
          :alt="t('ui.linkModelsStep1')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <img
            loading="lazy"
            src="/guide/concepts-link-model-2.webp"
            :alt="t('ui.linkModelsStep2')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
          <img
            loading="lazy"
            src="/guide/concepts-link-model-3.webp"
            :alt="t('ui.linkModelsStep3')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
        </div>
      </div>
    </section>

    <!-- 7. 模型映射 -->
    <section
      id="model-mapping"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.7ModelMapping') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.ifTheProviderRequestNameIsNotTheStandardName') }}
      </p>
      <div class="bg-[#f5f5f0]/80 dark:bg-[rgba(227,224,211,0.05)] p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] text-sm mb-6">
        <p class="mb-2">
          <span class="text-[#cc785c] font-medium mr-2">{{ t('ui.nameCorrectionExample2ef6565') }}</span>{{ t('ui.officialStandardName') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-opus-4-6</code> {{ t('ui.whileTheProviderCallsIt') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-opus-4-6-last</code>。
        </p>
        <p><span class="text-[#cc785c] font-medium mr-2">{{ t('ui.modelSubstitutionExample') }}</span>{{ t('ui.officialStandardName') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-opus-4-6</code> {{ t('ui.whileTheUpstreamRequestUses') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-sonnet-4-6</code>。</p>
      </div>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-model-mapping-1.webp"
          :alt="t('ui.modelMappingStep1')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <img
          loading="lazy"
          src="/guide/concepts-model-mapping-2.webp"
          :alt="t('ui.modelMappingStep2')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
      </div>
    </section>

    <!-- 8. 反向代理 -->
    <section
      id="reverse-proxy"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('ui.8ReverseProxy') }}</h2>
      <div class="space-y-6 mt-6">
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8] mb-3">
            1. Codex
          </h4>
          <img
            loading="lazy"
            src="/guide/concepts-reverse-proxy-codex.webp"
            :alt="t('ui.codexReverseProxy')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full max-w-2xl"
          >
        </div>
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8] mb-3">
            2. Krio
          </h4>
          <img
            loading="lazy"
            src="/guide/concepts-reverse-proxy-kiro.webp"
            :alt="t('ui.krioReverseProxy')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full max-w-2xl"
          >
        </div>
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8] mb-3">
            3. Antigravity
          </h4>
          <img
            loading="lazy"
            src="/guide/concepts-reverse-proxy-antigravity.webp"
            :alt="t('ui.antigravityReverseProxy')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full max-w-2xl"
          >
        </div>
      </div>
    </section>

    <!-- 9. 优先级管理 -->
    <section
      id="priority-management"
      class="scroll-mt-24 lg:scroll-mt-20 pb-8"
    >
      <h2>{{ t('ui.9PriorityManagement') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('ui.dragOrClickTheNumberInputDirectlyToAdjustPriority') }}
      </p>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        <div class="space-y-4">
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('ui.1ProviderPriority') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('ui.scheduleByProviderOrderWithLoadBalancingAtTheSame') }}
            </p>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('ui.2KeyPriority') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('ui.scheduleGlobalKeysTogetherWithLoadBalancingAtTheSame') }}
            </p>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('ui.3CacheAffinityMode') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('ui.withinTheKeyTtlWindowPreferTheKeyUsedBy') }}
            </p>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('ui.4LoadBalancingMode') }}
            </h4>
            <ul class="list-disc pl-5 mt-1 text-sm text-[#666663] dark:text-[#a3a094] space-y-1">
              <li>{{ t('ui.cancelGlobalProviderPriorityWhileKeepingPriorityAmongKeysInside') }}</li>
              <li>{{ t('ui.cancelGlobalKeyPriorityAndUseKeysInShuffledOrder') }}</li>
            </ul>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('ui.5FixedOrderMode') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('ui.cancelCacheAffinityAndAlwaysRequestKeysInFixedOrder') }}
            </p>
          </div>
        </div>
        
        <div class="space-y-4">
          <img
            loading="lazy"
            src="/guide/concepts-priority-1.webp"
            :alt="t('ui.priorityManagementStep1')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
          <img
            loading="lazy"
            src="/guide/concepts-priority-2.webp"
            :alt="t('ui.priorityManagementStep2')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
        </div>
      </div>
    </section>
  </div>
</template>
