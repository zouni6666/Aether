<template>
  <TableCard
    :title="t('nav.usageRecords')"
    class="relative"
  >
    <template #actions>
      <!-- 时间范围筛选 -->
      <TimeRangePicker
        v-model="timeRangeModel"
        :show-granularity="false"
        class="hidden shrink-0 md:flex"
      />

      <!-- 分隔线 -->
      <div class="hidden sm:block h-4 w-px bg-border" />

      <!-- 通用搜索 -->
      <div class="order-1 flex w-full items-center gap-2 md:order-none md:w-auto">
        <div class="relative min-w-0 flex-1 md:w-48 md:flex-none">
          <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
          <Input
            id="usage-records-search"
            v-model="localSearch"
            :placeholder="isAdmin ? t('ui.searchUsersKeys') : t('ui.searchKeysModels')"
            class="h-8 w-full text-xs border-border/60 pl-8"
          />
        </div>
      </div>

      <Button
        variant="ghost"
        size="icon"
        data-usage-hide-unknown-toggle="mobile"
        class="absolute right-12 top-2.5 h-8 w-8 shrink-0 md:hidden"
        :class="hideUnknownRecords ? 'text-primary' : ''"
        :title="hideUnknownRecords ? t('ui.showUnknownRequests') : t('ui.hideUnknownRequests')"
        :aria-label="t('ui.hideRequestsWithUnknownModelsOrProviders')"
        :aria-pressed="hideUnknownRecords"
        @click="$emit('update:hideUnknownRecords', !hideUnknownRecords)"
      >
        <EyeOff class="w-3.5 h-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="absolute right-4 top-2.5 h-8 w-8 shrink-0 md:hidden"
        :class="autoRefresh ? 'text-primary' : ''"
        :title="autoRefresh ? t('ui.disableAutomaticRefresh') : t('ui.enableAutomaticRefresh')"
        @click="$emit('update:autoRefresh', !autoRefresh)"
      >
        <RefreshCcw
          class="w-3.5 h-3.5"
          :class="autoRefresh ? 'animate-spin' : ''"
        />
      </Button>

      <div class="order-3 grid w-full grid-cols-2 gap-2 md:hidden">
        <!-- 时间范围筛选 -->
        <TimeRangePicker
          v-model="timeRangeModel"
          :show-granularity="false"
          class="min-w-0"
          preset-trigger-class="!w-full"
        />

        <!-- 用户筛选（仅管理员可见） -->
        <ServerUserSelector
          v-if="isAdmin"
          class="min-w-0"
          :model-value="filterUser"
          :initial-users="availableUsers"
          dropdown
          @update:model-value="$emit('update:filterUser', $event)"
        />

        <!-- 模型筛选 -->
        <Select
          :model-value="filterModel"
          @update:model-value="$emit('update:filterModel', $event)"
        >
          <SelectTrigger class="h-8 w-full min-w-0 text-xs border-border/60">
            <SelectValue :placeholder="t('ui.model')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__all__">
              {{ t('ui.allModels') }}
            </SelectItem>
            <SelectItem
              v-for="model in availableModels"
              :key="model"
              :value="model"
            >
              {{ model.replace('claude-', '') }}
            </SelectItem>
          </SelectContent>
        </Select>

        <!-- 提供商筛选（仅管理员可见） -->
        <Select
          v-if="isAdmin"
          :model-value="filterProvider"
          @update:model-value="$emit('update:filterProvider', $event)"
        >
          <SelectTrigger class="h-8 w-full min-w-0 text-xs border-border/60">
            <SelectValue :placeholder="t('nav.providers')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__all__">
              {{ t('ui.allProviders') }}
            </SelectItem>
            <SelectItem
              v-for="provider in availableProviders"
              :key="provider"
              :value="provider"
            >
              {{ provider }}
            </SelectItem>
          </SelectContent>
        </Select>

        <!-- API格式筛选 -->
        <Select
          :model-value="filterApiFormat"
          @update:model-value="$emit('update:filterApiFormat', $event)"
        >
          <SelectTrigger class="h-8 w-full min-w-0 text-xs border-border/60">
            <SelectValue :placeholder="t('ui.format')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__all__">
              {{ t('ui.allFormats') }}
            </SelectItem>
            <SelectItem
              v-for="format in availableApiFormats"
              :key="format.value"
              :value="format.value"
            >
              {{ format.label }}
            </SelectItem>
          </SelectContent>
        </Select>

        <!-- 状态筛选 -->
        <Select
          :model-value="filterStatus"
          @update:model-value="$emit('update:filterStatus', $event)"
        >
          <SelectTrigger class="h-8 w-full min-w-0 text-xs border-border/60">
            <SelectValue :placeholder="t('ui.status6320b4a')" />
          </SelectTrigger>
          <SelectContent>
            <!-- 手机与桌面共用状态选项，避免新增筛选时漏掉手机入口。 -->
            <SelectItem
              v-for="option in statusFilterOptions"
              :key="option.value"
              :value="option.value"
              :disabled="option.disabled"
            >
              {{ option.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <!-- 分隔线 -->
      <div class="hidden sm:block h-4 w-px bg-border" />

      <!-- 列显示配置（桌面端） -->
      <MultiSelect
        v-model="visibleColumnIds"
        class="hidden md:block"
        :options="columnSelectOptions"
        :placeholder="t('ui.visibleColumns')"
        trigger-class="w-40 h-8 text-xs border-border/60"
        dropdown-min-width="14rem"
        :searchable="false"
      />

      <!-- 分隔线 -->
      <div class="hidden sm:block h-4 w-px bg-border" />

      <!-- 自动刷新按钮 -->
      <Button
        variant="ghost"
        size="icon"
        data-usage-hide-unknown-toggle="desktop"
        class="hidden h-8 w-8 shrink-0 md:inline-flex"
        :class="hideUnknownRecords ? 'text-primary' : ''"
        :title="hideUnknownRecords ? t('ui.showUnknownRequests') : t('ui.hideUnknownRequests')"
        :aria-label="t('ui.hideRequestsWithUnknownModelsOrProviders')"
        :aria-pressed="hideUnknownRecords"
        @click="$emit('update:hideUnknownRecords', !hideUnknownRecords)"
      >
        <EyeOff class="w-3.5 h-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="hidden h-8 w-8 shrink-0 md:inline-flex"
        :class="autoRefresh ? 'text-primary' : ''"
        :title="autoRefresh ? t('ui.disableAutomaticRefresh') : t('ui.enableAutomaticRefresh')"
        @click="$emit('update:autoRefresh', !autoRefresh)"
      >
        <RefreshCcw
          class="w-3.5 h-3.5"
          :class="autoRefresh ? 'animate-spin' : ''"
        />
      </Button>
    </template>

    <!-- 移动端卡片视图 -->
    <div class="md:hidden">
      <div
        v-if="records.length === 0"
        class="text-center py-12 text-muted-foreground"
      >
        {{ t('ui.noRequestRecords') }}
      </div>
      <UsageRecordMobileCard
        v-for="record in records"
        v-else
        :key="record.id"
        :record="record"
        :is-admin="isAdmin"
        :show-actual-cost="showActualCost"
        @show-detail="emit('showDetail', $event)"
      />
    </div>

    <!-- 桌面端表格视图 -->
    <Table
      class="hidden md:table table-fixed w-full"
      :class="[desktopTableMinWidthClass]"
    >
      <colgroup v-if="isAdmin">
        <col
          v-if="isColumnVisible('time')"
          class="w-[8%]"
        >
        <col
          v-if="isColumnVisible('user')"
          class="w-[12%]"
        >
        <col
          v-if="isColumnVisible('model')"
          class="w-[14%]"
        >
        <col
          v-if="isColumnVisible('provider')"
          class="w-[16%]"
        >
        <col
          v-if="isColumnVisible('api_format')"
          class="w-[15%]"
        >
        <col
          v-if="isColumnVisible('status')"
          class="w-[10%]"
        >
        <col
          v-if="isColumnVisible('tokens')"
          class="w-[10%]"
        >
        <col
          v-if="isColumnVisible('cost')"
          class="w-[6%]"
        >
        <col
          v-if="isColumnVisible('performance')"
          class="w-[9%]"
        >
        <col
          v-if="isColumnVisible('client_family')"
          class="w-[12%]"
        >
        <col
          v-if="isColumnVisible('client_ip')"
          class="w-[10%]"
        >
        <col
          v-if="isColumnVisible('user_agent')"
          class="w-[13%]"
        >
      </colgroup>
      <colgroup v-else>
        <col
          v-if="isColumnVisible('time')"
          class="w-[9%]"
        >
        <col
          v-if="isColumnVisible('key')"
          class="w-[17%]"
        >
        <col
          v-if="isColumnVisible('model')"
          class="w-[22%]"
        >
        <col
          v-if="isColumnVisible('api_format')"
          class="w-[14%]"
        >
        <col
          v-if="isColumnVisible('status')"
          class="w-[10%]"
        >
        <col
          v-if="isColumnVisible('tokens')"
          class="w-[11%]"
        >
        <col
          v-if="isColumnVisible('cost')"
          class="w-[7%]"
        >
        <col
          v-if="isColumnVisible('performance')"
          class="w-[10%]"
        >
        <col
          v-if="isColumnVisible('client_family')"
          class="w-[12%]"
        >
        <col
          v-if="isColumnVisible('client_ip')"
          class="w-[10%]"
        >
        <col
          v-if="isColumnVisible('user_agent')"
          class="w-[13%]"
        >
      </colgroup>
      <TableHeader>
        <TableRow class="border-b border-border/60 hover:bg-transparent">
          <TableHead
            v-if="isColumnVisible('time')"
            class="h-12 font-semibold w-[8%]"
          >
            {{ t('ui.time') }}
          </TableHead>
          <SortableTableHead
            v-if="isAdmin && isColumnVisible('user')"
            class="h-12 font-semibold w-[12%]"
            column-key="user"
            :sortable="false"
            :filter-active="filterUser !== '__all__'"
            :filter-title="t('ui.filterUser')"
            filter-content-class="w-64 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ t('auth.role.user') }} <template #filter="{ close }">
              <ServerUserSelector
                :model-value="filterUser"
                :initial-users="availableUsers"
                @update:model-value="$emit('update:filterUser', $event)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <TableHead
            v-if="!isAdmin && isColumnVisible('key')"
            class="h-12 font-semibold w-[17%]"
          >
            {{ t('ui.key') }}
          </TableHead>
          <SortableTableHead
            v-if="isColumnVisible('model')"
            class="h-12 font-semibold"
            :class="[isAdmin ? 'w-[14%]' : 'w-[22%]']"
            column-key="model"
            :sortable="false"
            :filter-active="filterModel !== '__all__'"
            :filter-title="t('ui.filterModels')"
            filter-content-class="w-64 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ t('ui.model') }} <template #filter="{ close }">
              <TableFilterMenu
                :model-value="filterModel"
                :options="modelFilterOptions"
                @update:model-value="$emit('update:filterModel', $event)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <SortableTableHead
            v-if="isAdmin && isColumnVisible('provider')"
            class="h-12 font-semibold w-[16%]"
            column-key="provider"
            :sortable="false"
            :filter-active="filterProvider !== '__all__'"
            :filter-title="t('ui.filterProvider')"
            filter-content-class="w-48 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ t('nav.providers') }} <template #filter="{ close }">
              <TableFilterMenu
                :model-value="filterProvider"
                :options="providerFilterOptions"
                @update:model-value="$emit('update:filterProvider', $event)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <SortableTableHead
            v-if="isColumnVisible('api_format')"
            class="h-12 font-semibold"
            :class="[isAdmin ? 'w-[15%]' : 'w-[14%]']"
            column-key="api_format"
            :sortable="false"
            :filter-active="filterApiFormat !== '__all__'"
            :filter-title="t('ui.filterApiFormat')"
            filter-content-class="w-72 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ t('ui.apiFormat55eba68') }} <template #filter="{ close }">
              <TableFilterMenu
                :model-value="filterApiFormat"
                :options="apiFormatFilterOptions"
                @update:model-value="$emit('update:filterApiFormat', $event)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <SortableTableHead
            v-if="isColumnVisible('status')"
            class="h-12 font-semibold w-[10%] text-center"
            column-key="status"
            :sortable="false"
            align="center"
            :filter-active="filterStatus !== '__all__'"
            :filter-title="t('ui.filterType')"
            filter-content-class="w-44 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ t('ui.typeba40014') }} <template #filter="{ close }">
              <TableFilterMenu
                :model-value="filterStatus"
                :options="statusFilterOptions"
                @update:model-value="$emit('update:filterStatus', $event)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <TableHead
            v-if="isColumnVisible('tokens')"
            class="h-12 font-semibold w-[10%] text-center"
          >
            Tokens
          </TableHead>
          <TableHead
            v-if="isColumnVisible('cost')"
            class="h-12 font-semibold w-[6%] text-right"
          >
            {{ t('ui.cost7fe8603') }}
          </TableHead>
          <TableHead
            v-if="isColumnVisible('performance')"
            class="h-12 font-semibold w-[9%] text-right"
          >
            <div class="flex flex-col items-end text-[11px] leading-3">
              <span class="whitespace-nowrap">{{ t('ui.endToEnd') }}</span>
              <span class="whitespace-nowrap">{{ t('ui.firstTokenTotalTime') }}</span>
              <span class="text-muted-foreground font-normal">{{ t('ui.outputSpeed') }}</span>
            </div>
          </TableHead>
          <SortableTableHead
            v-if="isColumnVisible('client_family')"
            class="h-12 font-semibold w-[12%]"
            column-key="client_family"
            :sortable="false"
            :filter-active="filterClientFamily !== '__all__'"
            :filter-title="t('ui.filterClients')"
            filter-content-class="w-44 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ t('ui.client') }} <template #filter="{ close }">
              <TableFilterMenu
                :model-value="filterClientFamily"
                :options="clientFamilyFilterOptions"
                @update:model-value="$emit('update:filterClientFamily', $event)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <TableHead
            v-if="isColumnVisible('client_ip')"
            class="h-12 font-semibold w-[10%]"
          >
            {{ t('ui.ipAddress') }}
          </TableHead>
          <TableHead
            v-if="isColumnVisible('user_agent')"
            class="h-12 font-semibold w-[13%]"
          >
            User-Agent
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-if="records.length === 0">
          <TableCell
            :colspan="visibleColumnCount"
            class="text-center py-12 text-muted-foreground"
          >
            {{ t('ui.noRequestRecords') }}
          </TableCell>
        </TableRow>
        <TableRow
          v-for="record in records"
          v-else
          :key="record.id"
          :class="isAdmin ? 'cursor-pointer border-b border-border/40 hover:bg-muted/30 transition-colors h-[72px]' : 'border-b border-border/40 hover:bg-muted/30 transition-colors h-[72px]'"
          @mousedown="handleRowMouseDown($event, record.id)"
          @click="handleRowClick($event, record.id)"
        >
          <TableCell
            v-if="isColumnVisible('time')"
            class="py-4 w-[8%] align-top"
          >
            <div class="flex flex-col gap-0.5 leading-tight">
              <span class="text-xs text-foreground tabular-nums whitespace-nowrap">
                {{ formatRecordTime(record.created_at) }}
              </span>
              <span class="text-[11px] text-muted-foreground tabular-nums whitespace-nowrap">
                {{ formatRecordDate(record.created_at) }}
              </span>
            </div>
          </TableCell>
          <TableCell
            v-if="isAdmin && isColumnVisible('user')"
            class="py-4 w-[12%] truncate"
            :title="record.username || record.user_email || (record.user_id ? `User ${record.user_id}` : t('ui.deletedUser'))"
          >
            <div class="flex flex-col text-xs gap-0.5">
              <span class="truncate">
                {{ record.username || record.user_email || (record.user_id ? `User ${record.user_id}` : t('ui.deletedUser')) }}
              </span>
              <span
                v-if="record.api_key?.name"
                class="text-muted-foreground truncate"
                :title="record.api_key.name"
              >
                {{ record.api_key.name }}
              </span>
            </div>
          </TableCell>
          <!-- 用户页面的密钥列 -->
          <TableCell
            v-if="!isAdmin && isColumnVisible('key')"
            class="py-4 w-[17%]"
            :title="record.api_key?.name || '-'"
          >
            <div class="flex flex-col text-xs gap-0.5">
              <span class="truncate">{{ record.api_key?.name || '-' }}</span>
              <span
                v-if="record.api_key?.display"
                class="text-muted-foreground truncate"
              >
                {{ record.api_key.display }}
              </span>
            </div>
          </TableCell>
          <TableCell
            v-if="isColumnVisible('model')"
            class="font-medium py-4"
            :class="[isAdmin ? 'w-[14%]' : 'w-[22%]']"
            :title="getModelTooltip(record)"
          >
            <div class="flex min-w-0 items-start gap-1">
              <UsageModelDisplay
                :record="record"
                class="text-xs"
              />
              <ShieldCheck
                v-if="record.chat_pii_redaction"
                data-usage-attempt-marker="pii-redaction"
                class="mt-0.5 h-3.5 w-3.5 shrink-0 text-emerald-600 dark:text-emerald-400"
                :title="t('ui.thisRequestTriggeredSensitiveInformationProtectionTheContentSentUpstream')"
                :aria-label="t('ui.sensitiveInformationProtectionTriggered')"
              />
            </div>
          </TableCell>
          <TableCell
            v-if="isAdmin && isColumnVisible('provider')"
            class="py-4 w-[16%]"
          >
            <div class="flex min-w-0 items-center gap-1">
              <UsageProviderDisplay
                :record="record"
                class="text-xs"
              />
              <Shuffle
                v-if="record.has_fallback"
                data-usage-attempt-marker="fallback"
                class="w-3.5 h-3.5 text-amber-600 dark:text-amber-400 flex-shrink-0"
                :title="t('ui.thisRequestFailedOverToAnotherProvider')"
                :aria-label="t('ui.providerFailover')"
              />
              <!--
                被跳过的更高优先级候选：与"故障转移"区分开。
                它表示候选在调度阶段就被排除、请求从未发出，因此不会有任何上游报错，
                是"无报错却换了提供商"这一现象的直接解释。
              -->
              <Ban
                v-if="record.has_skipped_candidate && !record.has_fallback"
                data-usage-attempt-marker="skipped-candidate"
                class="w-3.5 h-3.5 text-slate-500 dark:text-slate-400 flex-shrink-0"
                :title="skippedCandidateTooltip(record)"
                :aria-label="skippedCandidateAriaLabel(record)"
              />
              <RefreshCcw
                v-if="record.has_retry"
                data-usage-attempt-marker="retry"
                class="w-3.5 h-3.5 text-blue-600 dark:text-blue-400 flex-shrink-0"
                :title="t('ui.thisRequestWasRetried')"
                :aria-label="t('ui.retried')"
              />
            </div>
          </TableCell>
          <TableCell
            v-if="isColumnVisible('api_format')"
            class="py-4"
            :class="[isAdmin ? 'w-[15%]' : 'w-[14%]']"
            :title="getApiFormatTooltip(record)"
          >
            <!-- 有格式转换或同族格式差异：两行显示 -->
            <div
              v-if="shouldShowFormatConversion(record)"
              class="flex flex-col text-xs gap-0.5"
            >
              <div class="flex items-center gap-1 whitespace-nowrap">
                <span>{{ formatApiFormat(record.api_format!) }}</span>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  class="w-3 h-3 text-muted-foreground flex-shrink-0"
                >
                  <path
                    fill-rule="evenodd"
                    d="M3 10a.75.75 0 01.75-.75h10.638L10.23 5.29a.75.75 0 111.04-1.08l5.5 5.25a.75.75 0 010 1.08l-5.5 5.25a.75.75 0 11-1.04-1.08l4.158-3.96H3.75A.75.75 0 013 10z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
              <span class="text-muted-foreground whitespace-nowrap">{{ formatApiFormat(record.endpoint_api_format!) }}</span>
            </div>
            <!-- 无格式转换：单行显示 -->
            <span
              v-else-if="record.api_format"
              class="text-xs whitespace-nowrap"
            >{{ formatApiFormat(record.api_format) }}</span>
            <span
              v-else
              class="text-muted-foreground text-xs"
            >-</span>
          </TableCell>
          <TableCell
            v-if="isColumnVisible('status')"
            class="text-center py-4 w-[10%]"
          >
            <!-- 优先显示请求状态 -->
            <Badge
              v-if="isUsageRecordFailed(record)"
              variant="destructive"
              class="whitespace-nowrap"
            >
              {{ t('ui.failed28384d7') }}
            </Badge>
            <Badge
              v-else-if="getDisplayStatus(record) === 'pending'"
              variant="outline"
              class="whitespace-nowrap animate-pulse border-muted-foreground/30 text-muted-foreground"
            >
              {{ t('ui.waiting') }}
            </Badge>
            <Badge
              v-else-if="getDisplayStatus(record) === 'streaming'"
              variant="outline"
              class="whitespace-nowrap animate-pulse border-primary/50 text-primary"
            >
              {{ t('ui.transferring') }}
            </Badge>
            <Badge
              v-else-if="record.status === 'cancelled'"
              variant="outline"
              class="whitespace-nowrap border-amber-500/50 text-amber-600 dark:text-amber-400"
            >
              {{ t('ui.canceled') }}
            </Badge>
            <Badge
              v-else-if="isUsageWebSocket(record)"
              variant="outline"
              data-usage-transport="websocket"
              :title="getWebSocketTransportTitle(record)"
              class="whitespace-nowrap border-sky-500/50 text-sky-600 dark:text-sky-400"
            >
              WS
            </Badge>
            <Badge
              v-else-if="getStreamModeSegments(record).hasConversion"
              :variant="streamBadgeVariant(getStreamModeSegments(record).client === '流式')"
              :class="(streamBadgeVariant(getStreamModeSegments(record).client === '流式') === 'secondary')
                ? 'whitespace-nowrap inline-flex items-center gap-1'
                : 'whitespace-nowrap border-border/60 text-muted-foreground inline-flex items-center gap-1'"
            >
              <span>{{ getStreamModeSegments(record).client }}</span>
              <span class="opacity-60">→</span>
              <span>{{ getStreamModeSegments(record).upstream }}</span>
            </Badge>
            <Badge
              v-else
              :variant="streamBadgeVariant(getUpstreamStream(record))"
              :class="(streamBadgeVariant(getUpstreamStream(record)) === 'secondary')
                ? 'whitespace-nowrap'
                : 'whitespace-nowrap border-border/60 text-muted-foreground'"
            >
              {{ getStreamModeLabel(record) }}
            </Badge>
          </TableCell>
          <TableCell
            v-if="isColumnVisible('tokens')"
            class="py-4 w-[10%]"
          >
            <template v-if="record.usage_available !== false">
              <div class="grid w-full min-w-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] gap-x-1 text-xs leading-tight tabular-nums">
                <span class="justify-self-end whitespace-nowrap text-right">
                  {{ formatTokens(getRecordEffectiveInputTokens(record)) }}
                </span>
                <span class="justify-self-center text-muted-foreground">
                  /
                </span>
                <span class="justify-self-start whitespace-nowrap text-left">
                  {{ formatTokens(record.output_tokens || 0) }}
                </span>
              </div>
              <div class="mt-0.5 grid w-full min-w-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] gap-x-1 text-xs leading-tight tabular-nums text-muted-foreground">
                <span
                  class="justify-self-end whitespace-nowrap text-right"
                  :class="[
                    hasPositiveTokens(getRecordCacheReadTokens(record)) ? 'text-foreground/70' : ''
                  ]"
                >
                  {{ formatOptionalTokens(getRecordCacheReadTokens(record)) }}
                </span>
                <span class="justify-self-center">
                  /
                </span>
                <span
                  class="justify-self-start whitespace-nowrap text-left"
                  :class="[
                    hasPositiveTokens(getRecordCacheCreationTokens(record)) ? 'text-foreground/70' : ''
                  ]"
                >
                  {{ formatOptionalTokens(getRecordCacheCreationTokens(record)) }}
                </span>
              </div>
              <div
                v-if="(record.input_audio_tokens || 0) > 0 || (record.output_audio_tokens || 0) > 0"
                class="mt-0.5 text-right text-[10px] leading-tight tabular-nums text-muted-foreground"
              >
                {{ t('ui.audio296c632') }} {{ formatOptionalTokens(record.input_audio_tokens) }} / {{ formatOptionalTokens(record.output_audio_tokens) }}
              </div>
            </template>
            <div
              v-else
              data-usage-unavailable="tokens"
              class="text-right text-xs text-muted-foreground"
              :title="t('ui.theUpstreamDidNotProvideVerifiableTokenOrCostUsage')"
            >
              {{ t('ui.unavailable460b357') }}
            </div>
          </TableCell>
          <TableCell
            v-if="isColumnVisible('cost')"
            class="text-right py-4 w-[6%]"
          >
            <UsageCostDisplay
              :record="record"
              :show-actual-cost="showActualCost"
            />
          </TableCell>
          <TableCell
            v-if="isColumnVisible('performance')"
            class="text-right py-4 w-[9%]"
          >
            <!-- pending/streaming 状态：首字与动态总耗时保留在同一行 -->
            <div
              v-if="getDisplayStatus(record) === 'pending' || getDisplayStatus(record) === 'streaming'"
              class="flex flex-col items-end text-xs gap-0.5"
            >
              <span class="tabular-nums whitespace-nowrap">
                <span>{{ formatRecordDurationSeconds(record.first_byte_time_ms) }}</span>
                <span class="text-muted-foreground"> / </span>
                <ElapsedTimeText
                  class="text-primary"
                  :created-at="record.created_at"
                  :response-time-updated-at="record.response_time_updated_at ?? null"
                  :status="getDisplayStatus(record)"
                  :response-time-ms="record.response_time_ms ?? null"
                />
              </span>
            </div>
            <!-- 已完成状态：首字 + 总耗时 -->
            <div
              v-else-if="hasRecordDisplayLatency(record)"
              class="flex flex-col items-end text-xs gap-0.5"
              :title="getRecordPerformanceTitle(record)"
            >
              <span class="tabular-nums whitespace-nowrap">{{ formatRecordLatencyPair(record) }}</span>
              <span class="text-muted-foreground tabular-nums whitespace-nowrap">
                {{ formatOutputRate(getRecordDisplayOutputRate(record)) }}
              </span>
            </div>
            <span
              v-else
              class="text-muted-foreground"
            >-</span>
          </TableCell>
          <TableCell
            v-if="isColumnVisible('client_family')"
            class="py-4 w-[12%] text-xs"
            :title="formatClientFamily(record.client_family)"
          >
            <Badge
              variant="outline"
              class="w-fit max-w-full border-border/60 text-muted-foreground"
            >
              <span class="truncate">{{ formatClientFamily(record.client_family) }}</span>
            </Badge>
          </TableCell>
          <TableCell
            v-if="isColumnVisible('client_ip')"
            class="py-4 w-[10%] text-xs truncate"
            :title="record.client_ip || '-'"
          >
            {{ record.client_ip || '-' }}
          </TableCell>
          <TableCell
            v-if="isColumnVisible('user_agent')"
            class="py-4 w-[13%] text-xs truncate"
            :title="record.user_agent || '-'"
          >
            {{ formatUserAgent(record.user_agent) }}
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>

    <!-- 分页控件 -->
    <template #pagination>
      <Pagination
        v-if="totalRecords > 0"
        :current="currentPage"
        :total="totalRecords"
        :page-size="pageSize"
        :page-size-options="pageSizeOptions"
        cache-key="usage-records-page-size"
        @update:current="$emit('update:currentPage', $event)"
        @update:page-size="$emit('update:pageSize', $event)"
      />
    </template>
  </TableCard>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ref, computed, onBeforeUnmount, watch } from 'vue'
import { useLocalStorage } from '@vueuse/core'
import {
  TableCard,
  Badge,
  Button,
  Input,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  Pagination,
  SortableTableHead,
  TableFilterMenu,
} from '@/components/ui'
import { Ban, EyeOff, RefreshCcw, Search, ShieldCheck, Shuffle } from 'lucide-vue-next'
import { formatTokens } from '@/utils/format'
import {
  formatOutputRate,
} from '../performance'
import {
  isUsageRecordFailed,
  isUsageWebSocket,
} from '../utils/status'
import { useRowClick } from '@/composables/useRowClick'
import { useDarkMode } from '@/composables/useDarkMode'
import { API_FORMAT_ORDER, formatApiFormat } from '@/api/endpoints/types/api-format'
import { formatClientFamily } from '@/features/usage/utils/clientFamily'
import { formatUsageWebSocketTransportTitle as getWebSocketTransportTitle } from '../utils/websocketTransport'
import {
  getDisplayStatus,
  getStreamModeLabel,
  getStreamModeSegments,
  getUpstreamStream,
  formatRecordDate,
  formatRecordTime,
  getRecordEffectiveInputTokens,
  getRecordCacheReadTokens,
  getRecordCacheCreationTokens,
  hasPositiveTokens,
  formatOptionalTokens,
  formatRecordLatencyPair,
  formatRecordDurationSeconds,
  hasRecordDisplayLatency,
  getRecordDisplayOutputRate,
  getRecordPerformanceTitle,
  formatUserAgent,
  shouldShowFormatConversion,
  skippedCandidateTooltip,
  skippedCandidateAriaLabel,
  getApiFormatTooltip,
  getModelTooltip,
} from '../utils/usageRecordDisplay'
import type { DateRangeParams, UsageRecord } from '../types'
import { MultiSelect, TimeRangePicker } from '@/components/common'
import type { MultiSelectOption } from '@/components/common/MultiSelect.vue'
import ElapsedTimeText from './ElapsedTimeText.vue'
import ServerUserSelector from './ServerUserSelector.vue'
import UsageModelDisplay from './UsageModelDisplay.vue'
import UsageCostDisplay from './UsageCostDisplay.vue'
import UsageProviderDisplay from './UsageProviderDisplay.vue'
import UsageRecordMobileCard from './UsageRecordMobileCard.vue'

export interface UserOption {
  id: string
  username: string
  email: string
}

interface FilterOption {
  value: string
  label: string
  disabled?: boolean
}

type UsageRecordColumnId =
  | 'time'
  | 'user'
  | 'key'
  | 'model'
  | 'provider'
  | 'api_format'
  | 'status'
  | 'tokens'
  | 'cost'
  | 'performance'
  | 'client_family'
  | 'client_ip'
  | 'user_agent'

interface UsageRecordColumnOption {
  id: UsageRecordColumnId
  label: string
  adminOnly?: boolean
  userOnly?: boolean
}

const props = defineProps<{
  records: UsageRecord[]
  isAdmin: boolean
  showActualCost: boolean
  loading: boolean
  // 时间范围
  timeRange: DateRangeParams
  // 筛选
  filterSearch: string
  filterUser: string
  filterModel: string
  filterProvider: string
  filterApiFormat: string
  filterStatus: string
  filterClientFamily: string
  availableUsers: UserOption[]
  availableModels: string[]
  availableProviders: string[]
  availableClientFamilies: string[]
  // 分页
  currentPage: number
  pageSize: number
  totalRecords: number
  pageSizeOptions: number[]
  // 自动刷新
  autoRefresh: boolean
  hideUnknownRecords: boolean
}>()

const emit = defineEmits<{
  'update:timeRange': [value: DateRangeParams]
  'update:filterSearch': [value: string]
  'update:filterUser': [value: string]
  'update:filterModel': [value: string]
  'update:filterProvider': [value: string]
  'update:filterApiFormat': [value: string]
  'update:filterStatus': [value: string]
  'update:filterClientFamily': [value: string]
  'update:currentPage': [value: number]
  'update:pageSize': [value: number]
  'update:autoRefresh': [value: boolean]
  'update:hideUnknownRecords': [value: boolean]
  'refresh': []
  'showDetail': [id: string]
  'prefetchDetail': [id: string]
}>()

const { t } = useI18n()

const USAGE_RECORD_COLUMN_OPTIONS: UsageRecordColumnOption[] = [
  { id: 'time', get label() { return t('ui.time') } },
  { id: 'user', get label() { return t('auth.role.user') }, adminOnly: true },
  { id: 'key', get label() { return t('ui.key') }, userOnly: true },
  { id: 'model', get label() { return t('ui.model') } },
  { id: 'provider', get label() { return t('nav.providers') }, adminOnly: true },
  { id: 'api_format', get label() { return t('ui.apiFormat55eba68') } },
  { id: 'status', get label() { return t('ui.typeStatus') } },
  { id: 'tokens', label: 'Tokens' },
  { id: 'cost', get label() { return t('ui.cost7fe8603') } },
  { id: 'performance', get label() { return t('ui.durationSpeedf3ebf76') } },
  { id: 'client_family', get label() { return t('ui.clientType') } },
  { id: 'client_ip', get label() { return t('ui.ipAddress') } },
  { id: 'user_agent', label: 'User-Agent' },
]

const DEFAULT_ADMIN_COLUMNS: UsageRecordColumnId[] = [
  'time',
  'user',
  'model',
  'provider',
  'api_format',
  'status',
  'tokens',
  'cost',
  'performance',
]

const DEFAULT_USER_COLUMNS: UsageRecordColumnId[] = [
  'time',
  'key',
  'model',
  'api_format',
  'status',
  'tokens',
  'cost',
  'performance',
]

// 使用统一 API 格式枚举，避免使用记录筛选项和系统格式列表漂移。
const availableApiFormats = API_FORMAT_ORDER.map((value) => ({
  value,
  label: formatApiFormat(value),
}))

const adminVisibleColumnIds = useLocalStorage<UsageRecordColumnId[]>(
  'usage-records-visible-columns-admin',
  DEFAULT_ADMIN_COLUMNS,
)
const userVisibleColumnIds = useLocalStorage<UsageRecordColumnId[]>(
  'usage-records-visible-columns-user',
  DEFAULT_USER_COLUMNS,
)

const roleColumnOptions = computed(() => USAGE_RECORD_COLUMN_OPTIONS.filter((column) => {
  if (column.adminOnly && !props.isAdmin) return false
  if (column.userOnly && props.isAdmin) return false
  return true
}))

const roleColumnIds = computed(() => new Set(roleColumnOptions.value.map(column => column.id)))

function sanitizeColumnIds(
  ids: readonly string[],
  fallback: readonly UsageRecordColumnId[],
): UsageRecordColumnId[] {
  const seen = new Set<UsageRecordColumnId>()
  const sanitized = ids.filter((id): id is UsageRecordColumnId => {
    if (!roleColumnIds.value.has(id as UsageRecordColumnId)) return false
    if (seen.has(id as UsageRecordColumnId)) return false
    seen.add(id as UsageRecordColumnId)
    return true
  })
  if (sanitized.length === 0) return [...fallback]
  // Add newly introduced feature column to existing saved layouts, keeping it
  // immediately before Tokens as the default presentation order.
  return sanitized
}

const visibleColumnIds = computed<UsageRecordColumnId[]>({
  get: () => sanitizeColumnIds(
    props.isAdmin ? adminVisibleColumnIds.value : userVisibleColumnIds.value,
    props.isAdmin ? DEFAULT_ADMIN_COLUMNS : DEFAULT_USER_COLUMNS,
  ),
  set: (value) => {
    const sanitized = sanitizeColumnIds(value, props.isAdmin ? DEFAULT_ADMIN_COLUMNS : DEFAULT_USER_COLUMNS)
    if (props.isAdmin) {
      adminVisibleColumnIds.value = sanitized
    } else {
      userVisibleColumnIds.value = sanitized
    }
  },
})

const visibleColumnSet = computed(() => new Set<UsageRecordColumnId>(visibleColumnIds.value))
const visibleColumnCount = computed(() => visibleColumnIds.value.length)
const desktopTableMinWidthClass = computed(() => {
  const metadataColumnCount = visibleColumnIds.value.filter(column => (
    column === 'client_family' ||
    column === 'client_ip' ||
    column === 'user_agent'
  )).length
  if (metadataColumnCount >= 3) return 'min-w-[1520px]'
  if (metadataColumnCount > 0) return 'min-w-[1320px]'
  return props.isAdmin ? 'min-w-[1120px]' : 'min-w-[960px]'
})

const columnSelectOptions = computed<MultiSelectOption[]>(() => roleColumnOptions.value.map(column => ({
  value: column.id,
  label: column.label,
})))

function isColumnVisible(column: UsageRecordColumnId): boolean {
  return visibleColumnSet.value.has(column)
}

const modelFilterOptions = computed<FilterOption[]>(() => [
  { value: '__all__', get label() { return t('ui.allModels') } },
  ...props.availableModels.map((model) => ({
    value: model,
    label: model.replace('claude-', ''),
  })),
])

const providerFilterOptions = computed<FilterOption[]>(() => [
  { value: '__all__', get label() { return t('ui.allProviders') } },
  ...props.availableProviders.map((provider) => ({
    value: provider,
    label: provider,
  })),
])

const clientFamilyFilterOptions = computed<FilterOption[]>(() => {
  const families = new Set<string>(props.availableClientFamilies)
  props.records.forEach((record) => {
    const family = record.client_family?.trim()
    if (family) families.add(family)
  })
  return [
    { value: '__all__', get label() { return t('ui.allClients') } },
    ...Array.from(families).sort().map((family) => ({
      value: family,
      label: formatClientFamily(family),
    })),
  ]
})

const apiFormatFilterOptions = computed<FilterOption[]>(() => [
  { value: '__all__', get label() { return t('ui.allFormats') } },
  ...availableApiFormats.map((format) => ({
    value: format.value,
    label: format.label,
  })),
])

const statusFilterOptions: FilterOption[] = [
  { value: '__all__', get label() { return t('ui.allTypes') } },
  { value: 'stream', get label() { return t('ui.httpStreaming') } },
  { value: 'standard', get label() { return t('ui.httpStandard') } },
  { value: 'websocket', label: 'WebSocket (WS)' },
  { value: 'active', get label() { return t('ui.activee85e590') } },
  { value: 'failed', get label() { return t('ui.failed28384d7') } },
  { value: 'cancelled', get label() { return t('ui.canceled') } },
  { value: 'has_retry', get label() { return t('ui.retried') } },
  { value: 'has_fallback', get label() { return t('ui.failedOver') } },
  // 与"发生转移"区分：候选在调度阶段就被排除，请求从未发出（无上游报错）
  { value: 'has_skipped_candidate', label: '有候选被调度跳过' },
]

const timeRangeModel = computed({
  get: () => props.timeRange,
  set: (value: DateRangeParams) => emit('update:timeRange', value)
})

// 通用搜索（输入防抖）
const SEARCH_EMIT_DEBOUNCE_MS = 300
const localSearch = ref(props.filterSearch)
let searchEmitTimer: ReturnType<typeof setTimeout> | null = null

function cancelPendingSearchEmit() {
  if (searchEmitTimer !== null) {
    clearTimeout(searchEmitTimer)
    searchEmitTimer = null
  }
}

function scheduleSearchEmit(value: string) {
  cancelPendingSearchEmit()
  searchEmitTimer = setTimeout(() => {
    searchEmitTimer = null
    if (value !== props.filterSearch) {
      emit('update:filterSearch', value)
    }
  }, SEARCH_EMIT_DEBOUNCE_MS)
}

watch(() => props.filterSearch, (value) => {
  if (value !== localSearch.value) {
    cancelPendingSearchEmit()
    localSearch.value = value
  }
})

watch(localSearch, (value) => {
  if (value === props.filterSearch) return
  scheduleSearchEmit(value)
})

onBeforeUnmount(() => {
  cancelPendingSearchEmit()
})

// 使用复用的行点击逻辑
const { handleMouseDown, shouldTriggerRowClick } = useRowClick()
const { isDark } = useDarkMode()

// 暗色模式下交换"流式"与"标准"徽章的填充/描边样式
function streamBadgeVariant(isStream: boolean): 'secondary' | 'outline' {
  if (isDark.value) {
    return isStream ? 'outline' : 'secondary'
  }
  return isStream ? 'secondary' : 'outline'
}

function handleRowMouseDown(event: MouseEvent, id: string) {
  handleMouseDown(event)
  if (!props.isAdmin) return
  if (event.button !== 0) return
  emit('prefetchDetail', id)
}

// 处理行点击，排除文本选择操作
function handleRowClick(event: MouseEvent, id: string) {
  if (!props.isAdmin) return
  if (!shouldTriggerRowClick(event)) return
  emit('showDetail', id)
}

// useDebounceFn 自动处理清理，无需 onUnmounted

</script>
