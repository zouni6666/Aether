<template>
  <!-- Codex 上游额度信息（仅当有元数据时显示） -->
  <div
    v-if="hasCodexQuotaDisplayData(key)"
    class="mt-2 p-2 bg-muted/30 rounded-md"
  >
    <ProviderQuotaSectionHeader
      :title="t('ui.accountQuota')"
      :loading="refreshing"
      :updated-text="getCodexQuotaDisplay(key)?.updated_at ? formatUpdatedAt(getCodexQuotaDisplay(key)?.updated_at || 0) : null"
    />
    <!-- 普通 Codex 限额并排显示：Team/Plus/Enterprise 账号 2列, Free 账号 1列 -->
    <div
      class="grid gap-3"
      :class="isCodexTeamPlan(key) ? 'grid-cols-2' : 'grid-cols-1'"
    >
      <!-- 主限额 -->
      <ProviderQuotaProgressRow
        v-if="getCodexQuotaDisplay(key)?.primary_used_percent !== undefined"
        :label="getCodexPrimaryQuotaLabel(key)"
        :used-percent="getCodexQuotaDisplay(key)?.primary_used_percent || 0"
        :remaining-percent="toCodexRemainingPercent(getCodexQuotaDisplay(key)?.primary_used_percent)"
        :meter-class="getQuotaRemainingClass(getCodexQuotaDisplay(key)?.primary_used_percent || 0)"
        :bar-class="getQuotaRemainingBarColor(getCodexQuotaDisplay(key)?.primary_used_percent || 0)"
        :reset-text="(getCodexQuotaDisplay(key)?.primary_reset_at || getCodexQuotaDisplay(key)?.primary_reset_seconds) && shouldStartCodexResetCountdown(getCodexQuotaDisplay(key)?.primary_used_percent || 0)
          ? getResetCountdownText(
            getCodexQuotaDisplay(key)?.primary_reset_at,
            getCodexQuotaDisplay(key)?.primary_reset_seconds,
            getCodexQuotaDisplay(key)?.updated_at,
            getCodexQuotaDisplay(key)?.primary_used_percent
          )
          : null"
        :footer-class="getResetCountdownClass(
          getCodexQuotaDisplay(key)?.primary_reset_at,
          getCodexQuotaDisplay(key)?.primary_reset_seconds,
          getCodexQuotaDisplay(key)?.updated_at,
          getCodexQuotaDisplay(key)?.primary_used_percent
        )"
      />
      <!-- 5H限额（仅 Team/Plus/Enterprise 显示） -->
      <ProviderQuotaProgressRow
        v-if="isCodexTeamPlan(key) && getCodexQuotaDisplay(key)?.secondary_used_percent !== undefined"
        :label="t('ui.5hLimit')"
        :used-percent="getCodexQuotaDisplay(key)?.secondary_used_percent || 0"
        :remaining-percent="toCodexRemainingPercent(getCodexQuotaDisplay(key)?.secondary_used_percent)"
        :meter-class="getQuotaRemainingClass(getCodexQuotaDisplay(key)?.secondary_used_percent || 0)"
        :bar-class="getQuotaRemainingBarColor(getCodexQuotaDisplay(key)?.secondary_used_percent || 0)"
        :reset-text="shouldStartCodexResetCountdown(getCodexQuotaDisplay(key)?.secondary_used_percent || 0)
          ? getCodexQuotaResetText(
            getCodexQuotaDisplay(key)?.secondary_reset_at,
            getCodexQuotaDisplay(key)?.secondary_reset_seconds,
            getCodexQuotaDisplay(key)?.updated_at,
            getCodexQuotaDisplay(key)?.secondary_used_percent
          )
          : null"
        :footer-class="getResetCountdownClass(
          getCodexQuotaDisplay(key)?.secondary_reset_at,
          getCodexQuotaDisplay(key)?.secondary_reset_seconds,
          getCodexQuotaDisplay(key)?.updated_at,
          getCodexQuotaDisplay(key)?.secondary_used_percent
        )"
      />
    </div>
    <!-- Spark 限额独立一行展示，避免与普通 Codex 周/5H 混淆 -->
    <div
      v-if="hasCodexSparkQuotaDisplayData(key)"
      class="mt-3 border-t border-border/60 pt-2"
    >
      <div class="mb-1 text-[10px] text-muted-foreground">
        GPT-5.3 Codex Spark
      </div>
      <div class="grid gap-3 grid-cols-2">
        <ProviderQuotaProgressRow
          v-if="getCodexQuotaDisplay(key)?.spark_secondary_used_percent !== undefined"
          :label="t('ui.sparkWeeklyd2a6127')"
          :used-percent="getCodexQuotaDisplay(key)?.spark_secondary_used_percent || 0"
          :remaining-percent="toCodexRemainingPercent(getCodexQuotaDisplay(key)?.spark_secondary_used_percent)"
          :meter-class="getQuotaRemainingClass(getCodexQuotaDisplay(key)?.spark_secondary_used_percent || 0)"
          :bar-class="getQuotaRemainingBarColor(getCodexQuotaDisplay(key)?.spark_secondary_used_percent || 0)"
          :reset-text="shouldStartCodexResetCountdown(getCodexQuotaDisplay(key)?.spark_secondary_used_percent || 0)
            ? getCodexQuotaResetText(
              getCodexQuotaDisplay(key)?.spark_secondary_reset_at,
              getCodexQuotaDisplay(key)?.spark_secondary_reset_seconds,
              getCodexQuotaDisplay(key)?.updated_at,
              getCodexQuotaDisplay(key)?.spark_secondary_used_percent
            )
            : null"
          :footer-class="getResetCountdownClass(
            getCodexQuotaDisplay(key)?.spark_secondary_reset_at,
            getCodexQuotaDisplay(key)?.spark_secondary_reset_seconds,
            getCodexQuotaDisplay(key)?.updated_at,
            getCodexQuotaDisplay(key)?.spark_secondary_used_percent
          )"
        />
        <ProviderQuotaProgressRow
          v-if="getCodexQuotaDisplay(key)?.spark_primary_used_percent !== undefined"
          label="Spark 5H"
          :used-percent="getCodexQuotaDisplay(key)?.spark_primary_used_percent || 0"
          :remaining-percent="toCodexRemainingPercent(getCodexQuotaDisplay(key)?.spark_primary_used_percent)"
          :meter-class="getQuotaRemainingClass(getCodexQuotaDisplay(key)?.spark_primary_used_percent || 0)"
          :bar-class="getQuotaRemainingBarColor(getCodexQuotaDisplay(key)?.spark_primary_used_percent || 0)"
          :reset-text="shouldStartCodexResetCountdown(getCodexQuotaDisplay(key)?.spark_primary_used_percent || 0)
            ? getCodexQuotaResetText(
              getCodexQuotaDisplay(key)?.spark_primary_reset_at,
              getCodexQuotaDisplay(key)?.spark_primary_reset_seconds,
              getCodexQuotaDisplay(key)?.updated_at,
              getCodexQuotaDisplay(key)?.spark_primary_used_percent
            )
            : null"
          :footer-class="getResetCountdownClass(
            getCodexQuotaDisplay(key)?.spark_primary_reset_at,
            getCodexQuotaDisplay(key)?.spark_primary_reset_seconds,
            getCodexQuotaDisplay(key)?.updated_at,
            getCodexQuotaDisplay(key)?.spark_primary_used_percent
          )"
        />
      </div>
    </div>
    <div
      v-if="getCodexResetCreditAvailableCount(key) !== null || hasPendingCodexResetCredit(key)"
      class="mt-3 border-t border-border/60 pt-2"
    >
      <div class="flex flex-wrap items-center gap-x-1 gap-y-1 text-[10px] leading-4 text-muted-foreground">
        <button
          v-if="canConsumeCodexResetCredit(key)"
          type="button"
          class="font-medium text-primary underline-offset-2 transition-colors hover:text-primary/80 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 disabled:pointer-events-none disabled:opacity-60"
          :disabled="consumingResetCredit"
          @click="$emit('consumeResetCredit')"
        >
          {{ consumingResetCredit
            ? t('ui.resetting')
            : hasPendingCodexResetCredit(key) ? t('ui.continueCheckingReset') : t('ui.clickToReset') }}
        </button>
        <span
          v-else
          class="font-medium"
        >
          {{ t('ui.clickToReset') }}
        </span>
        <span>{{ formatCodexResetCreditCount(key) }}</span>
        <template v-if="getVisibleCodexResetCreditItems(key).length > 0">
          <span aria-hidden="true">|</span>
          <span>{{ t('ui.expiringSoon1b588db') }}</span>
          <template
            v-for="(item, itemIndex) in getVisibleCodexResetCreditItems(key)"
            :key="item.id || `${item.displayKey}-${item.expiresAt}`"
          >
            <span
              :title="item.title"
              class="tabular-nums"
            >
              {{ item.displayKey }} {{ formatCodexResetCreditExpiresAt(item.expiresAt) }}
            </span>
            <span
              v-if="itemIndex < getVisibleCodexResetCreditItems(key).length - 1"
              aria-hidden="true"
            >·</span>
          </template>
        </template>
      </div>
    </div>
  </div>
  <!-- Antigravity 上游额度摘要（按家族分组展示关键配额） -->
  <div
    v-if="providerType === 'antigravity' && (hasAntigravityQuotaDisplayData(key) || isAntigravityForbiddenKey(key))"
    class="mt-2 p-2 rounded-md"
    :class="isAntigravityForbiddenKey(key) ? 'bg-destructive/10 border border-destructive/30' : 'bg-muted/30'"
  >
    <!-- 封禁状态显示 -->
    <div
      v-if="isAntigravityForbiddenKey(key)"
      class="flex items-center gap-2 text-destructive"
    >
      <ShieldX class="w-4 h-4 shrink-0" />
      <div class="flex-1 min-w-0">
        <div class="text-[11px] font-medium">
          {{ t('ui.accountAccessForbidden') }}
        </div>
        <div
          v-if="getAntigravityForbiddenReason(key)"
          class="text-[10px] text-destructive/80 truncate"
          :title="getAntigravityForbiddenReason(key)"
        >
          {{ getAntigravityForbiddenReason(key) }}
        </div>
      </div>
      <span
        v-if="getAntigravityForbiddenAt(key)"
        class="text-[9px] text-destructive/60 shrink-0"
      >
        {{ formatBanTimestamp(getAntigravityForbiddenAt(key)) }}
      </span>
    </div>
    <!-- 正常配额显示 -->
    <template v-else>
      <ProviderQuotaSectionHeader
        :title="t('ui.modelQuota')"
        :loading="refreshing"
        :updated-text="getAntigravityQuotaUpdatedAt(key) ? formatUpdatedAt(getAntigravityQuotaUpdatedAt(key) || 0) : null"
      />
      <div class="grid grid-cols-2 gap-3">
        <ProviderQuotaProgressRow
          v-for="item in getAntigravityQuotaGroupItems(key)"
          :key="item.model"
          :label="item.label"
          :title="item.label"
          :used-percent="item.usedPercent"
          :remaining-percent="item.remainingPercent"
          :meter-text="item.detail"
          :meter-class="getQuotaRemainingClass(item.usedPercent)"
          :bar-class="getQuotaRemainingBarColor(item.usedPercent)"
        >
          <template #footer>
            <div class="mt-0.5 space-y-0.5">
              <div
                v-if="item.resetSeconds !== null"
                class="text-[9px] text-muted-foreground/70"
              >
                <template v-if="item.resetSeconds !== null && item.resetSeconds > 0">
                  {{ formatResetTime(item.resetSeconds) }}{{ t('ui.untilReset') }}
                </template>
                <template v-else-if="item.resetSeconds !== null && item.resetSeconds <= 0">
                  {{ t('ui.resetd2cd2e2') }}
                </template>
              </div>
            </div>
          </template>
        </ProviderQuotaProgressRow>
      </div>
    </template>
  </div>
  <!-- Claude Code 账号额度（5H / 周窗口） -->
  <div
    v-if="providerType === 'claude_code' && hasClaudeCodeQuotaDisplayData(key)"
    class="mt-2 p-2 rounded-md bg-muted/30"
  >
    <ProviderQuotaSectionHeader
      :title="t('ui.accountQuota')"
      :loading="refreshing"
      :updated-text="getClaudeCodeQuotaUpdatedAt(key) ? formatUpdatedAt(getClaudeCodeQuotaUpdatedAt(key) || 0) : null"
    />
    <div class="grid grid-cols-2 gap-3">
      <ProviderQuotaProgressRow
        v-for="item in getClaudeCodeQuotaItems(key)"
        :key="item.code"
        :label="item.label"
        :used-percent="item.usedPercent"
        :remaining-percent="item.remainingPercent"
        :meter-class="getQuotaRemainingClass(item.usedPercent)"
        :bar-class="getQuotaRemainingBarColor(item.usedPercent)"
      >
        <template #footer>
          <div
            v-if="item.resetSeconds !== null && item.remainingPercent < 100"
            class="text-[9px] text-muted-foreground/70 mt-0.5"
          >
            <template v-if="item.resetSeconds > 0">
              {{ formatResetTime(item.resetSeconds) }}{{ t('ui.untilReset') }}
            </template>
            <template v-else>
              {{ t('ui.resetd2cd2e2') }}
            </template>
          </div>
        </template>
      </ProviderQuotaProgressRow>
    </div>
    <div
      v-if="getClaudeCodeResetCreditAvailableCount(key) !== null"
      class="mt-3 border-t border-border/60 pt-2"
    >
      <div class="flex flex-wrap items-center gap-x-1 gap-y-1 text-[10px] leading-4 text-muted-foreground">
        <span>{{ formatCodexResetCreditCountLabel(getClaudeCodeResetCreditAvailableCount(key)) }}</span>
        <template v-if="getVisibleClaudeCodeResetCreditItems(key).length > 0">
          <span aria-hidden="true">|</span>
          <span>{{ t('ui.expiringSoon1b588db') }}</span>
          <template
            v-for="(item, itemIndex) in getVisibleClaudeCodeResetCreditItems(key)"
            :key="`${item.displayKey}-${item.expiresAt}`"
          >
            <span
              :title="item.title"
              class="tabular-nums"
            >
              {{ item.displayKey }} {{ formatCodexResetCreditExpiresAt(item.expiresAt) }}
            </span>
            <span
              v-if="itemIndex < getVisibleClaudeCodeResetCreditItems(key).length - 1"
              aria-hidden="true"
            >·</span>
          </template>
        </template>
      </div>
    </div>
  </div>
  <!-- Gemini CLI 上游模型配额 -->
  <div
    v-if="providerType === 'gemini_cli' && hasGeminiCliQuotaDisplayData(key)"
    class="mt-2 p-2 rounded-md bg-muted/30"
  >
    <ProviderQuotaSectionHeader
      :title="t('ui.modelQuota')"
      :loading="refreshing"
      :updated-text="getGeminiCliQuotaUpdatedAt(key) ? formatUpdatedAt(getGeminiCliQuotaUpdatedAt(key) || 0) : null"
    />
    <div
      v-if="geminiCliCreditsText"
      class="mb-2 text-[10px] font-medium text-foreground/90"
    >
      {{ geminiCliCreditsText }}
    </div>
    <div
      v-if="getGeminiCliQuotaItems(key).length > 0"
      class="grid grid-cols-2 gap-3"
    >
      <ProviderQuotaProgressRow
        v-for="item in getGeminiCliQuotaItems(key)"
        :key="item.model"
        :label="item.label"
        :title="item.model"
        :used-percent="item.usedPercent"
        :remaining-percent="item.remainingPercent"
        :meter-class="getQuotaRemainingClass(item.usedPercent)"
        :bar-class="getQuotaRemainingBarColor(item.usedPercent)"
      >
        <template #footer>
          <div
            v-if="item.resetSeconds !== null && item.remainingPercent < 100"
            class="text-[9px] text-muted-foreground/70 mt-0.5"
          >
            <template v-if="item.resetSeconds > 0">
              {{ formatResetTime(item.resetSeconds) }}{{ t('ui.untilReset') }}
            </template>
            <template v-else>
              {{ t('ui.resetd2cd2e2') }}
            </template>
          </div>
        </template>
      </ProviderQuotaProgressRow>
    </div>
  </div>
  <!-- Kiro 上游额度信息（仅当有元数据时显示） -->
  <div
    v-if="providerType === 'kiro' && (hasKiroQuotaDisplayData(key) || isKiroBannedKey(key))"
    class="mt-2 p-2 rounded-md"
    :class="isKiroBannedKey(key) ? 'bg-destructive/10 border border-destructive/30' : 'bg-muted/30'"
  >
    <!-- 封禁状态显示 -->
    <div
      v-if="isKiroBannedKey(key)"
      class="flex items-center gap-2 text-destructive"
    >
      <ShieldX class="w-4 h-4 shrink-0" />
      <div class="flex-1 min-w-0">
        <div class="text-[11px] font-medium">
          {{ t('ui.accountBannedb1ae4c0') }}
        </div>
        <div
          v-if="getKiroQuotaDisplay(key)?.ban_reason"
          class="text-[10px] text-destructive/80 truncate"
          :title="getKiroQuotaDisplay(key)?.ban_reason"
        >
          {{ getKiroQuotaDisplay(key)?.ban_reason }}
        </div>
      </div>
      <span
        v-if="getKiroQuotaDisplay(key)?.banned_at"
        class="text-[9px] text-destructive/60 shrink-0"
      >
        {{ formatBanTimestamp(getKiroQuotaDisplay(key)?.banned_at) }}
      </span>
    </div>
    <!-- 正常配额显示 -->
    <template v-else>
      <ProviderQuotaSectionHeader
        :title="t('ui.accountQuota')"
        :loading="refreshing"
        :updated-text="getKiroQuotaDisplay(key)?.updated_at ? formatUpdatedAt(getKiroQuotaDisplay(key)?.updated_at || 0) : null"
      />
      <!-- Kiro 额度显示：使用进度 -->
      <div>
        <!-- 使用额度进度条 -->
        <ProviderQuotaProgressRow
          :label="t('ui.usageQuota')"
          :used-percent="getKiroQuotaDisplay(key)?.usage_percentage || 0"
          :meter-class="getQuotaRemainingClass(getKiroQuotaDisplay(key)?.usage_percentage || 0)"
          :bar-class="getQuotaRemainingBarColor(getKiroQuotaDisplay(key)?.usage_percentage || 0)"
        >
          <template #footer>
            <div class="flex items-center justify-between text-[9px] text-muted-foreground/70 mt-0.5">
              <span>
                {{ formatKiroUsage(getKiroQuotaDisplay(key)?.current_usage) }} /
                {{ formatKiroUsage(getKiroQuotaDisplay(key)?.usage_limit) }}
              </span>
              <span v-if="getKiroQuotaDisplay(key)?.next_reset_at">
                {{ formatKiroResetTime(getKiroQuotaDisplay(key)?.next_reset_at) }}{{ t('ui.reset') }}
              </span>
            </div>
          </template>
        </ProviderQuotaProgressRow>
      </div>
    </template>
  </div>
  <!-- xAI / Grok Build 订阅额度 -->
  <div
    v-if="providerType === 'xai' && hasXaiQuotaDisplayData(key)"
    class="mt-2 p-2 rounded-md bg-muted/30"
  >
    <ProviderQuotaSectionHeader
      :title="t('ui.accountQuota')"
      :loading="refreshing"
      :updated-text="getXaiQuotaDisplay(key)?.updated_at ? formatUpdatedAt(getXaiQuotaDisplay(key)?.updated_at || 0) : null"
    />
    <div class="space-y-2">
      <ProviderQuotaProgressRow
        v-if="getXaiQuotaDisplay(key)?.usage_percentage !== undefined || getXaiQuotaDisplay(key)?.remaining_percentage !== undefined"
        :label="getXaiUsageLabel(key)"
        :used-percent="getXaiUsedPercent(key)"
        :remaining-percent="getXaiRemainingPercent(key)"
        :meter-class="getQuotaRemainingClass(getXaiUsedPercent(key))"
        :bar-class="getQuotaRemainingBarColor(getXaiUsedPercent(key))"
        :reset-text="getXaiQuotaDisplay(key)?.next_reset_at
          ? `${formatKiroResetTime(getXaiQuotaDisplay(key)?.next_reset_at)}${t('ui.reset')}`
          : null"
      >
        <template
          v-if="getXaiQuotaDisplay(key)?.usage_limit != null"
          #footer
        >
          <div class="flex items-center justify-between text-[9px] text-muted-foreground/70 mt-0.5">
            <span>
              {{ formatKiroUsage(getXaiQuotaDisplay(key)?.current_usage) }} /
              {{ formatKiroUsage(getXaiQuotaDisplay(key)?.usage_limit) }}
            </span>
            <span v-if="getXaiQuotaDisplay(key)?.next_reset_at">
              {{ formatKiroResetTime(getXaiQuotaDisplay(key)?.next_reset_at) }}{{ t('ui.reset') }}
            </span>
          </div>
        </template>
      </ProviderQuotaProgressRow>
      <div
        v-if="getXaiQuotaDisplay(key)?.prepaid_balance != null"
        class="text-[10px] text-muted-foreground"
      >
        {{ t('ui.prepaidCredits') }}: {{ formatKiroUsage(getXaiQuotaDisplay(key)?.prepaid_balance) }}
      </div>
      <ProviderQuotaProgressRow
        v-if="getXaiQuotaDisplay(key)?.on_demand_cap"
        :label="t('ui.onDemandCredits')"
        :used-percent="getXaiOnDemandUsedPercent(key)"
        :meter-class="getQuotaRemainingClass(getXaiOnDemandUsedPercent(key))"
        :bar-class="getQuotaRemainingBarColor(getXaiOnDemandUsedPercent(key))"
      >
        <template #footer>
          <div class="text-[9px] text-muted-foreground/70 mt-0.5">
            {{ formatKiroUsage(getXaiQuotaDisplay(key)?.on_demand_used) }} /
            {{ formatKiroUsage(getXaiQuotaDisplay(key)?.on_demand_cap) }}
          </div>
        </template>
      </ProviderQuotaProgressRow>
    </div>
  </div>
  <!-- Windsurf 上游额度信息 -->
  <div
    v-if="providerType === 'windsurf' && (hasWindsurfQuotaDisplayData(key) || isWindsurfUnavailableKey(key) || isWindsurfExhaustedKey(key))"
    class="mt-2 p-2 rounded-md"
    :class="isWindsurfUnavailableKey(key) ? 'bg-destructive/10 border border-destructive/30' : (isWindsurfExhaustedKey(key) ? 'bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-900/50' : 'bg-muted/30')"
  >
    <div
      v-if="isWindsurfUnavailableKey(key)"
      class="flex items-center gap-2 text-destructive"
    >
      <ShieldX class="w-4 h-4 shrink-0" />
      <div class="flex-1 min-w-0">
        <div class="text-[11px] font-medium">
          {{ t('ui.accountUnavailable') }}
        </div>
        <div
          v-if="getWindsurfQuotaDisplay(key)?.last_error"
          class="text-[10px] text-destructive/80 truncate"
          :title="getWindsurfQuotaDisplay(key)?.last_error || ''"
        >
          {{ getWindsurfQuotaDisplay(key)?.last_error }}
        </div>
      </div>
    </div>
    <template v-else>
      <div
        v-if="isWindsurfExhaustedKey(key)"
        class="mb-2 flex items-center gap-2 text-amber-700 dark:text-amber-300"
      >
        <ShieldX class="w-4 h-4 shrink-0" />
        <div class="flex-1 min-w-0">
          <div class="text-[11px] font-medium">
            {{ getWindsurfQuotaStatusLabel(key) }}
          </div>
          <div
            v-if="getWindsurfQuotaDisplay(key)?.last_error"
            class="text-[10px] text-amber-700/80 dark:text-amber-300/80 truncate"
            :title="getWindsurfQuotaDisplay(key)?.last_error || ''"
          >
            {{ getWindsurfQuotaDisplay(key)?.last_error }}
          </div>
        </div>
      </div>
      <ProviderQuotaSectionHeader
        :title="t('ui.accountQuota')"
        :loading="refreshing"
        :updated-text="getWindsurfQuotaDisplay(key)?.updated_at ? formatUpdatedAt(getWindsurfQuotaDisplay(key)?.updated_at || 0) : null"
      />
      <div class="grid grid-cols-2 gap-3">
        <ProviderQuotaProgressRow
          v-if="getWindsurfQuotaDisplay(key)?.daily_remaining_percent !== undefined"
          :label="t('ui.dailyQuotaa644d7c')"
          :used-percent="getWindsurfQuotaDisplay(key)?.daily_used_percent || 0"
          :remaining-percent="getWindsurfQuotaDisplay(key)?.daily_remaining_percent || 0"
          :meter-class="getQuotaRemainingClass(getWindsurfQuotaDisplay(key)?.daily_used_percent || 0)"
          :bar-class="getQuotaRemainingBarColor(getWindsurfQuotaDisplay(key)?.daily_used_percent || 0)"
          :reset-text="getWindsurfQuotaDisplay(key)?.daily_reset_at
            ? `${formatKiroResetTime(getWindsurfQuotaDisplay(key)?.daily_reset_at || 0)}${t('ui.reset')}`
            : null"
        />
        <ProviderQuotaProgressRow
          v-if="getWindsurfQuotaDisplay(key)?.weekly_remaining_percent !== undefined"
          :label="t('ui.weeklyQuota')"
          :used-percent="getWindsurfQuotaDisplay(key)?.weekly_used_percent || 0"
          :remaining-percent="getWindsurfQuotaDisplay(key)?.weekly_remaining_percent || 0"
          :meter-class="getQuotaRemainingClass(getWindsurfQuotaDisplay(key)?.weekly_used_percent || 0)"
          :bar-class="getQuotaRemainingBarColor(getWindsurfQuotaDisplay(key)?.weekly_used_percent || 0)"
          :reset-text="getWindsurfQuotaDisplay(key)?.weekly_reset_at
            ? `${formatKiroResetTime(getWindsurfQuotaDisplay(key)?.weekly_reset_at || 0)}${t('ui.reset')}`
            : null"
        />
      </div>
      <div
        v-if="hasWindsurfPromptQuota(key) || hasWindsurfFlexQuota(key)"
        class="mt-2 flex items-center gap-3 text-[9px] text-muted-foreground/70"
      >
        <span v-if="hasWindsurfPromptQuota(key)">
          Prompt {{ formatKiroUsage(getWindsurfQuotaDisplay(key)?.prompt_used || 0) }} /
          {{ formatKiroUsage(getWindsurfQuotaDisplay(key)?.prompt_limit || 0) }}
        </span>
        <span v-if="hasWindsurfFlexQuota(key)">
          Flex {{ formatKiroUsage(getWindsurfQuotaDisplay(key)?.flex_used || 0) }} /
          {{ formatKiroUsage(getWindsurfQuotaDisplay(key)?.flex_limit || 0) }}
        </span>
      </div>
      <div
        v-if="hasWindsurfModelCount(key) || hasWindsurfModelPreview(key)"
        class="mt-2 flex items-center justify-between gap-2 text-[9px] text-muted-foreground/70"
      >
        <span>
          {{ t('ui.model') }} {{ getWindsurfQuotaDisplay(key)?.allowed_models_count ?? getWindsurfQuotaDisplay(key)?.models?.length }} {{ t('ui.items3a11681') }}
        </span>
        <span
          v-if="getWindsurfModelPreview(key)"
          class="truncate"
          :title="getWindsurfModelPreview(key) || ''"
        >
          {{ getWindsurfModelPreview(key) }}
        </span>
      </div>
    </template>
  </div>
  <!-- ChatGPT Web 上游额度信息（生图配额） -->
  <div
    v-if="providerType === 'chatgpt_web' && hasChatGPTWebQuotaDisplayData(key)"
    class="mt-2 p-2 rounded-md bg-muted/30"
  >
    <ProviderQuotaSectionHeader
      :title="t('ui.accountQuota')"
      :loading="refreshing"
      :updated-text="getChatGPTWebQuotaDisplay(key)?.updated_at ? formatUpdatedAt(getChatGPTWebQuotaDisplay(key)?.updated_at || 0) : null"
    />
    <div>
      <ProviderQuotaProgressRow
        :label="t('ui.remainingQuota')"
        :used-percent="getChatGPTWebQuotaUsedPercent(key)"
        :remaining-percent="getChatGPTWebQuotaRemainingPercent(key)"
        :meter-class="getQuotaRemainingClass(getChatGPTWebQuotaUsedPercent(key))"
        :bar-class="getQuotaRemainingBarColor(getChatGPTWebQuotaUsedPercent(key))"
      >
        <template #footer>
          <div class="flex items-center justify-between text-[9px] text-muted-foreground/70 mt-0.5">
            <span>
              {{ formatChatGPTWebUsage(getChatGPTWebQuotaDisplay(key)?.image_quota_remaining) }} /
              {{ formatChatGPTWebUsage(getChatGPTWebQuotaDisplay(key)?.image_quota_total) }}
            </span>
            <span v-if="getChatGPTWebQuotaDisplay(key)?.image_quota_reset_at">
              {{ formatKiroResetTime(getChatGPTWebQuotaDisplay(key)?.image_quota_reset_at) }}{{ t('ui.reset') }}
            </span>
          </div>
        </template>
      </ProviderQuotaProgressRow>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ShieldX } from 'lucide-vue-next'
import { useI18n } from '@/i18n'
import type { EndpointAPIKey } from '@/api/endpoints'
import ProviderQuotaProgressRow from './ProviderQuotaProgressRow.vue'
import ProviderQuotaSectionHeader from './ProviderQuotaSectionHeader.vue'
import {
  getCodexQuotaDisplay,
  getCodexPrimaryQuotaLabel,
  hasCodexQuotaDisplayData,
  hasCodexSparkQuotaDisplayData,
  getCodexResetCreditAvailableCount,
  formatCodexResetCreditCount,
  getVisibleCodexResetCreditItems,
  hasPendingCodexResetCredit,
  getKiroQuotaDisplay,
  hasKiroQuotaDisplayData,
  getXaiQuotaDisplay,
  hasXaiQuotaDisplayData,
  getXaiUsageLabel,
  getXaiUsedPercent,
  getXaiRemainingPercent,
  getXaiOnDemandUsedPercent,
  getWindsurfQuotaDisplay,
  hasWindsurfQuotaDisplayData,
  isWindsurfUnavailableKey,
  isWindsurfExhaustedKey,
  getWindsurfModelPreview,
  hasWindsurfPromptQuota,
  hasWindsurfFlexQuota,
  hasWindsurfModelCount,
  hasWindsurfModelPreview,
  getChatGPTWebQuotaDisplay,
  hasChatGPTWebQuotaDisplayData,
  getChatGPTWebQuotaUsedPercent,
  getChatGPTWebQuotaRemainingPercent,
  formatChatGPTWebUsage,
  isKiroBannedKey,
  formatBanTimestamp,
  isAntigravityForbiddenKey,
  getAntigravityForbiddenReason,
  getAntigravityForbiddenAt,
  getAntigravityQuotaUpdatedAt,
  formatKiroUsage,
} from '@/features/providers/utils/providerKeyQuotaDisplay'
import { getGeminiCliAccountCreditsText } from '@/features/providers/utils/providerKeyQuota'
import {
  formatCodexResetCreditCount as formatCodexResetCreditCountLabel,
  formatCodexResetCreditExpiresAt,
} from './codex-reset-credit-display'
import {
  getClaudeCodeQuotaUpdatedAt,
  getClaudeCodeResetCreditAvailableCount,
  getGeminiCliQuotaItems,
  getGeminiCliQuotaUpdatedAt,
  getQuotaRemainingBarColor,
  getQuotaRemainingClass,
  hasGeminiCliQuotaDisplayData,
  isCodexTeamPlan,
  shouldStartCodexResetCountdown,
  toCodexRemainingPercent,
  useProviderKeyQuotaPresentation,
} from '../composables/useProviderKeyQuotaPresentation'

/**
 * 单个密钥的上游配额摘要：按提供商类型展示 Codex / Antigravity / Claude Code /
 * Gemini CLI / Kiro / xAI / Windsurf / ChatGPT Web 的额度、重置倒计时与封禁状态。
 */
const props = withDefaults(defineProps<{
  providerKey: EndpointAPIKey
  providerType?: string | null
  refreshing?: boolean
  consumingResetCredit?: boolean
  canConsumeResetCredit?: boolean
  countdownTick: number
}>(), {
  providerType: null,
  refreshing: false,
  consumingResetCredit: false,
  canConsumeResetCredit: false,
})

defineEmits<{
  consumeResetCredit: []
}>()

const { t } = useI18n()

const key = computed(() => props.providerKey)
const providerType = computed(() => props.providerType)
const refreshing = computed(() => props.refreshing)
const consumingResetCredit = computed(() => props.consumingResetCredit)
const canConsumeCodexResetCredit = (_key: EndpointAPIKey) => props.canConsumeResetCredit
const geminiCliCreditsText = computed(() => getGeminiCliAccountCreditsText(key.value, 'gemini_cli'))
const tick = computed(() => props.countdownTick)

const {
  formatUpdatedAt,
  formatKiroResetTime,
  formatResetTime,
  getResetCountdownText,
  getCodexQuotaResetText,
  getResetCountdownClass,
  getWindsurfQuotaStatusLabel,
  hasAntigravityQuotaDisplayData,
  getAntigravityQuotaGroupItems,
  hasClaudeCodeQuotaDisplayData,
  getClaudeCodeQuotaItems,
  getVisibleClaudeCodeResetCreditItems,
} = useProviderKeyQuotaPresentation(tick)
</script>
