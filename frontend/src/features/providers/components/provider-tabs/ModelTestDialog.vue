<template>
  <Dialog
    :open="open"
    size="3xl"
    :close-on-backdrop="false"
    @update:open="(val: boolean) => { if (!val) emit('close') }"
  >
    <template #header>
      <div class="border-b border-border px-6 py-4">
        <div class="space-y-3">
          <div class="text-lg font-semibold text-foreground leading-tight">
            {{ dialogTitle }}
          </div>
          <p
            v-if="dialogDescription && !showResult"
            class="text-xs text-muted-foreground"
          >
            {{ dialogDescription }}
          </p>
        </div>
      </div>
    </template>

    <div
      v-if="showSetup"
      class="space-y-4"
    >
      <div
        v-if="endpoints.length > 0"
        class="space-y-2"
      >
        <div class="grid min-h-8 w-full items-center gap-2 sm:h-8 sm:grid-cols-2">
          <div class="text-sm font-medium text-foreground">
            {{ t('ui.selectTestEndpoint') }}
          </div>
          <div
            v-if="modelMappingAvailable"
            class="grid w-full grid-cols-[auto_1fr] items-center gap-2"
          >
            <div class="text-sm font-medium text-foreground">
              {{ t('guide.nav.concepts.modelMapping') }}
            </div>
            <Select
              :model-value="selectedModelMappingValue"
              @update:model-value="handleModelMappingValueChange"
            >
              <SelectTrigger class="h-8 min-h-8 w-full shrink-0 overflow-hidden border-border/60 py-0 text-xs leading-none [transition-property:none] [&>span:first-child]:min-w-0 [&>span:first-child]:flex-1 [&>span:first-child]:text-left [&>span:first-child]:leading-none">
                <SelectValue :placeholder="requestedModelName || t('ui.currentModel')" />
              </SelectTrigger>
              <SelectContent
                class="w-[var(--radix-select-trigger-width)]"
                align="end"
                :disable-portal="false"
                :searchable="false"
              >
                <SelectItem
                  :value="requestedModelName"
                  :text-value="requestedModelName"
                >
                  {{ requestedModelName }}
                </SelectItem>
                <SelectItem
                  v-for="option in modelMappingOptions"
                  :key="option.name"
                  :value="option.name"
                  :text-value="option.name"
                >
                  {{ option.name }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <div class="grid gap-2 md:grid-cols-2">
          <button
            v-for="endpoint in endpoints"
            :key="endpoint.id"
            type="button"
            class="h-full w-full rounded-lg border px-3 py-3 text-left transition-colors"
            :class="selectedEndpoint?.id === endpoint.id
              ? 'border-primary bg-primary/5'
              : 'border-border/60 hover:bg-muted/40'"
            @click="emit('selectEndpoint', endpoint.id)"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="min-w-0">
                <div class="text-sm font-medium">
                  {{ formatApiFormat(endpoint.api_format) }}
                </div>
                <div class="mt-1 text-xs text-muted-foreground break-all">
                  {{ endpoint.base_url }}
                </div>
              </div>
              <Badge :variant="selectedEndpoint?.id === endpoint.id ? 'success' : 'outline'">
                {{ selectedEndpoint?.id === endpoint.id ? t('ui.selected3f4ebc4') : (endpoint.is_active ? t('ui.available4d99c97') : t('ui.disabledbc5a87a')) }}
              </Badge>
            </div>
          </button>
        </div>
      </div>

      <div
        v-if="showKeySelector"
        class="space-y-2"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="text-sm font-medium text-foreground">
            {{ t('ui.testKey') }}
          </div>
          <div class="text-xs text-muted-foreground">
            {{ keySelectionStatus }}
          </div>
        </div>
        <MultiSelect
          :model-value="selectedKeyIds"
          :options="keyOptions"
          :placeholder="keySelectorPlaceholder"
          :search-placeholder="t('ui.searchKeys')"
          :empty-text="t('ui.noKeysAvailableToSelect')"
          :no-results-text="t('ui.noMatchingKeys')"
          trigger-class="h-9 min-h-9 rounded-md border-border/60 text-xs"
          dropdown-min-width="24rem"
          :search-threshold="0"
          :disabled="keyOptionsLoading && keyOptions.length === 0"
          @update:model-value="emit('update:selectedKeyIds', $event)"
        />
      </div>

      <div class="grid gap-4 lg:grid-cols-2 lg:items-start">
        <div class="space-y-2">
          <div class="flex items-center justify-between gap-3">
            <div class="text-sm font-medium">
              {{ t('ui.testRequestHeaders') }}
            </div>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('ui.formatHeadersJson')"
                @click="formatRequestHeadersDraft"
              >
                <Code2 class="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('ui.resetHeaders')"
                @click="resetRequestHeadersDraft"
              >
                <RotateCcw class="h-4 w-4" />
              </Button>
            </div>
          </div>
          <Textarea
            :model-value="requestHeadersDraft"
            class="min-h-[220px] font-mono text-xs"
            :placeholder="t('ui.enterRequestHeadersAsJson')"
            @update:model-value="emit('update:requestHeadersDraft', $event)"
          />
          <div
            v-if="requestHeadersError"
            class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive"
          >
            {{ requestHeadersError }}
          </div>
          <div class="rounded-md border border-border/60 bg-muted/20 px-3 py-2 text-[11px] text-muted-foreground">
            {{ t('ui.theseHeadersAreMergedIntoTheTestRequestAuthenticationAnd') }}
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center justify-between gap-3">
            <div class="text-sm font-medium">
              {{ t('ui.testRequestBody') }}
            </div>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('ui.formatBodyJson')"
                @click="formatRequestBodyDraft"
              >
                <Code2 class="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('ui.resetRequestBody')"
                @click="resetRequestBodyDraft"
              >
                <RotateCcw class="h-4 w-4" />
              </Button>
            </div>
          </div>
          <Textarea
            :model-value="requestBodyDraft"
            class="min-h-[220px] font-mono text-xs"
            :placeholder="t('ui.enterRequestBodyAsJson')"
            @update:model-value="emit('update:requestBodyDraft', $event)"
          />
          <div
            v-if="requestBodyError"
            class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive"
          >
            {{ requestBodyError }}
          </div>
          <div class="rounded-md border border-border/60 bg-muted/20 px-3 py-2 text-[11px] text-muted-foreground">
            {{ t('ui.theModelFieldUsesYourEditedValueOrTheCurrent') }}
          </div>
        </div>
      </div>

      <Button
        class="w-full"
        :disabled="startDisabled"
        @click="emit('start')"
      >
        {{ t('ui.startTest') }}
      </Button>
    </div>

    <div
      v-else-if="testing"
      class="space-y-4 py-6"
    >
      <div class="flex flex-col items-center justify-center gap-3 text-center">
        <Loader2 class="h-8 w-8 animate-spin text-primary" />
        <div class="space-y-1">
          <p class="text-sm font-medium">
            {{ t('ui.testingModel') }}
          </p>
          <p class="text-xs text-muted-foreground">
            {{ selectingModelName || '-' }}
          </p>
          <p
            v-if="selectedEndpoint"
            class="text-xs text-muted-foreground"
          >
            {{ t('ui.endpoint') }}{{ formatApiFormat(selectedEndpoint.api_format) }} · {{ selectedEndpoint.base_url }}
          </p>
        </div>
      </div>

      <div class="rounded-lg border border-border/60 bg-muted/20 p-4 space-y-4">
        <div class="space-y-2">
          <div class="flex items-center justify-between gap-3 text-xs text-muted-foreground">
            <span>{{ t('ui.liveProgress') }}</span>
            <span>{{ liveProgressText }}</span>
          </div>
          <div class="h-2 overflow-hidden rounded-full bg-muted">
            <div
              class="h-full bg-primary transition-all duration-300"
              :style="{ width: `${liveProgressPercent}%` }"
            />
          </div>
          <div
            v-if="liveTraceSummary.total_candidates > 0"
            class="flex flex-wrap gap-1.5"
          >
            <Badge
              v-for="item in liveSummaryItems"
              :key="item.key"
              :variant="item.variant"
              class="px-1.5 py-0 text-[10px]"
            >
              {{ item.label }} {{ item.value }}
            </Badge>
          </div>
          <div
            v-else
            class="text-xs text-muted-foreground"
          >
            {{ t('ui.preparingTestRequest') }}
          </div>
        </div>

        <div class="grid gap-3 sm:grid-cols-2">
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ liveEntityLabel }}
            </div>
            <div class="break-all text-sm font-medium">
              {{ liveAccountTitle }}
            </div>
            <div
              v-if="liveAccountMeta"
              class="break-all text-xs text-muted-foreground"
            >
              {{ liveAccountMeta }}
            </div>
          </div>
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ t('ui.status6320b4a') }}
            </div>
            <div class="text-sm font-medium">
              {{ liveStatusTitle }}
            </div>
            <div
              v-if="liveStatusDetail"
              class="break-all text-xs text-muted-foreground"
            >
              {{ liveStatusDetail }}
            </div>
          </div>
        </div>

        <div
          v-if="requestId"
          class="break-all text-[11px] text-muted-foreground"
        >
          {{ t('ui.requestId') }}<code class="rounded bg-muted px-1 py-0.5">{{ requestId }}</code>
        </div>

        <div
          v-if="liveRecentCandidates.length > 0"
          class="space-y-2"
        >
          <div class="text-xs font-medium text-muted-foreground">
            {{ t('ui.recent') }}
          </div>
          <div class="space-y-2">
            <div
              v-for="candidate in liveRecentCandidates"
              :key="`${candidate.id}-${candidate.status}`"
              class="flex items-start justify-between gap-3 rounded-md border border-border/50 bg-background/70 px-3 py-2 text-xs"
            >
              <div class="min-w-0 space-y-1">
                <div class="flex min-w-0 items-center gap-2">
                  <span class="shrink-0 text-muted-foreground">{{ formatTraceCandidateIndex(candidate) }}</span>
                  <Badge
                    :variant="statusVariant(candidate.status)"
                    class="shrink-0 px-1.5 py-0 text-[10px]"
                  >
                    {{ statusDisplay(candidate) }}
                  </Badge>
                  <span class="truncate font-medium">{{ formatTraceCandidateAccount(candidate) }}</span>
                </div>
                <div
                  v-if="traceCandidateDetail(candidate)"
                  class="break-all text-muted-foreground"
                >
                  {{ traceCandidateDetail(candidate) }}
                </div>
              </div>
              <div class="shrink-0 tabular-nums text-muted-foreground">
                {{ candidate.latency_ms != null ? `${candidate.latency_ms}ms` : '' }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      v-else-if="result"
      class="space-y-4"
    >
      <div class="rounded-lg border border-border/60 bg-muted/20 p-4 space-y-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <Badge
              :variant="result.success ? 'success' : 'destructive'"
              class="px-2 py-0.5"
            >
              {{ result.success ? t('ui.testPassed') : t('ui.testFailed') }}
            </Badge>
          </div>
          <div
            v-if="hasEffectiveModel"
            class="text-xs text-muted-foreground"
          >
            {{ t('ui.requestedModel') }}{{ resultModelTitle }}
          </div>
        </div>

        <div class="grid gap-3 lg:grid-cols-3">
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ resultEntityLabel }}
            </div>
            <div class="break-all text-sm font-medium">
              {{ resultWinningTitle }}
            </div>
          </div>
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ t('ui.schedulingResult') }}
            </div>
            <div class="text-sm font-medium">
              {{ resultDispatchTitle }}
            </div>
          </div>
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ t('ui.actualRequest') }}
            </div>
            <div class="break-all text-sm font-medium">
              {{ resultModelTitle }}
            </div>
          </div>
        </div>

        <div
          v-if="showCandidateDiagnostics"
          class="flex flex-wrap gap-1.5"
        >
          <Badge
            v-for="item in resultSummaryItems"
            :key="item.key"
            :variant="item.variant"
            class="px-1.5 py-0 text-[10px]"
          >
            {{ item.label }} {{ item.value }}
          </Badge>
        </div>
      </div>

      <div
        v-if="shouldCollapseAttempts"
        class="flex items-center justify-between gap-3 text-xs text-muted-foreground"
      >
        <span>{{ t('ui.showingTheFirst') }} {{ visibleAttempts.length }} {{ t('ui.of') }} {{ resultAttempts.length }} {{ t('ui.itemsf004f1d') }}</span>
        <Button
          variant="ghost"
          size="sm"
          @click="showAllAttempts = !showAllAttempts"
        >
          {{ showAllAttempts ? t('ui.hideDetails') : t('ui.expandAllValueItems', { count: resultAttempts.length }) }}
        </Button>
      </div>

      <div
        v-if="resultAttempts.length > 0"
        class="max-h-[360px] space-y-2 overflow-y-auto pr-1 sm:hidden"
      >
        <div
          v-for="(attempt, idx) in visibleAttempts"
          :key="'m' + idx"
          class="rounded-md border px-3 py-2 text-xs"
          :class="attemptRowClass(attempt.status, selectedInspectionKey === inspectionKey(attempt))"
          role="button"
          tabindex="0"
          @click="selectInspectionAttempt(attempt)"
          @keydown.enter.prevent="selectInspectionAttempt(attempt)"
          @keydown.space.prevent="selectInspectionAttempt(attempt)"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="flex min-w-0 items-center gap-1.5">
              <span class="shrink-0 text-muted-foreground">{{ formatAttemptIndex(attempt) }}</span>
              <Badge
                :variant="statusVariant(attempt.status)"
                class="shrink-0 px-1.5 py-0 text-[10px]"
              >
                {{ statusDisplay(attempt) }}
              </Badge>
              <span
                v-if="attempt.latency_ms != null"
                class="shrink-0 tabular-nums text-muted-foreground"
              >
                {{ attempt.latency_ms }}ms
              </span>
            </div>
          </div>
          <div class="mt-1.5 space-y-0.5">
            <div class="truncate font-medium">
              {{ attempt.key_name || maskKey(attempt.key_id) }}
            </div>
            <div
              v-if="attemptImagePreviews(attempt).length > 0"
              class="mt-2 flex flex-wrap gap-2"
            >
              <button
                v-for="(preview, imageIndex) in attemptImagePreviews(attempt).slice(0, 3)"
                :key="`${preview.src}-${imageIndex}`"
                type="button"
                class="h-16 w-16 overflow-hidden rounded-md border border-border/60 bg-muted/30 transition-colors hover:border-primary/60"
                :title="preview.label"
                @click.stop="openImagePreview(preview)"
              >
                <img
                  :src="preview.src"
                  :alt="preview.label"
                  class="h-full w-full object-contain"
                  loading="lazy"
                >
              </button>
            </div>
            <div
              v-else-if="attemptDetail(attempt) !== '-'"
              class="mt-1 break-all text-muted-foreground"
            >
              {{ attemptDetail(attempt) }}
            </div>
          </div>
        </div>
      </div>

      <div
        v-if="resultAttempts.length > 0"
        class="hidden max-h-[360px] overflow-y-auto rounded-md border sm:block"
      >
        <table class="w-full table-fixed text-xs">
          <colgroup>
            <col class="w-8">
            <col class="w-[28%]">
            <col class="w-16">
            <col class="w-16">
            <col>
          </colgroup>
          <thead class="sticky top-0 z-10">
            <tr class="border-b bg-muted/30">
              <th class="py-2 pl-3 pr-1 text-left font-medium">
                #
              </th>
              <th class="px-3 py-2 text-left font-medium">
                Key
              </th>
              <th class="px-3 py-2 text-left font-medium">
                {{ t('ui.status6320b4a') }}
              </th>
              <th class="px-3 py-2 text-right font-medium">
                {{ t('ui.latency') }}
              </th>
              <th class="px-3 py-2 text-left font-medium">
                {{ t('ui.details') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(attempt, idx) in visibleAttempts"
              :key="idx"
              class="last:border-b-0 align-top border-b"
              :class="attemptRowClass(attempt.status, selectedInspectionKey === inspectionKey(attempt))"
              tabindex="0"
              @click="selectInspectionAttempt(attempt)"
              @keydown.enter.prevent="selectInspectionAttempt(attempt)"
              @keydown.space.prevent="selectInspectionAttempt(attempt)"
            >
              <td class="py-2 pl-3 pr-1 text-muted-foreground">
                {{ formatAttemptIndex(attempt) }}
              </td>
              <td class="px-3 py-2">
                <div
                  class="truncate font-medium"
                  :title="attempt.key_name || attempt.key_id"
                >
                  {{ attempt.key_name || maskKey(attempt.key_id) }}
                </div>
              </td>
              <td class="px-3 py-2">
                <Badge
                  :variant="statusVariant(attempt.status)"
                  class="px-1.5 py-0 text-[10px]"
                >
                  {{ statusDisplay(attempt) }}
                </Badge>
              </td>
              <td class="px-3 py-2 text-right tabular-nums text-muted-foreground">
                {{ attempt.latency_ms != null ? attempt.latency_ms + 'ms' : '-' }}
              </td>
              <td class="px-3 py-2 text-muted-foreground">
                <div
                  v-if="attemptImagePreviews(attempt).length > 0"
                  class="flex flex-wrap gap-2"
                >
                  <button
                    v-for="(preview, imageIndex) in attemptImagePreviews(attempt).slice(0, 4)"
                    :key="`${preview.src}-${imageIndex}`"
                    type="button"
                    class="h-14 w-14 overflow-hidden rounded-md border border-border/60 bg-muted/30 transition-colors hover:border-primary/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary/70"
                    :title="preview.label"
                    @click.stop="openImagePreview(preview)"
                  >
                    <img
                      :src="preview.src"
                      :alt="preview.label"
                      class="h-full w-full object-contain"
                      loading="lazy"
                    >
                  </button>
                  <span
                    v-if="attemptImagePreviews(attempt).length > 4"
                    class="flex h-14 items-center rounded-md border border-border/60 px-2 text-xs text-muted-foreground"
                  >
                    +{{ attemptImagePreviews(attempt).length - 4 }}
                  </span>
                </div>
                <div
                  v-else
                  class="line-clamp-2 break-all"
                  :title="attemptDetail(attempt)"
                >
                  {{ attemptDetail(attempt) }}
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div
        v-else-if="resultAttempts.length === 0"
        class="py-4 text-center text-sm text-muted-foreground"
      >
        {{ resultEmptyMessage }}
      </div>

      <div
        v-if="showDebugInspector"
        class="space-y-3"
      >
        <div
          v-if="selectedInspectionAttempt"
          class="space-y-3"
        >
          <div
            v-if="showAttemptDiagnostics"
            class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground"
          >
            <span class="font-medium text-foreground">
              {{ formatAttemptIndex(selectedInspectionAttempt) }}
            </span>
            <span>{{ selectedInspectionAttempt.key_name || maskKey(selectedInspectionAttempt.key_id) }}</span>
            <span>·</span>
            <span>{{ formatApiFormat(selectedInspectionAttempt.endpoint_api_format) }}</span>
          </div>

          <Card>
            <div class="p-3 sm:p-4">
              <Tabs
                v-model="inspectionTab"
                :default-value="inspectionTab"
              >
                <div class="flex items-center border-b pb-2 mb-3">
                  <button
                    v-for="tab in detailTabs"
                    :key="tab.name"
                    class="px-2 sm:px-3 py-1.5 text-sm transition-colors border-b-2 -mb-[9px] whitespace-nowrap"
                    :class="inspectionTab === tab.name
                      ? 'border-primary text-foreground font-medium'
                      : 'border-transparent text-muted-foreground hover:text-foreground'"
                    @click="inspectionTab = tab.name"
                  >
                    {{ tab.label }}
                  </button>
                </div>

                <div class="content-block rounded-md border overflow-hidden">
                  <div class="flex items-center justify-end gap-0.5 px-3 py-1 border-b bg-muted/40">
                    <button
                      :title="inspectionExpandDepth === 0 ? t('ui.expandAll') : t('ui.collapseAll')"
                      class="p-1 rounded transition-colors text-muted-foreground hover:bg-muted"
                      @click="inspectionExpandDepth === 0 ? expandInspectionContent() : collapseInspectionContent()"
                    >
                      <Maximize2
                        v-if="inspectionExpandDepth === 0"
                        class="w-3.5 h-3.5"
                      />
                      <Minimize2
                        v-else
                        class="w-3.5 h-3.5"
                      />
                    </button>

                    <button
                      :title="inspectionCopiedStates[inspectionTab] ? t('common.copied') : t('common.copy')"
                      class="p-1 rounded transition-colors text-muted-foreground hover:bg-muted"
                      @click="copyInspectionContent(inspectionTab)"
                    >
                      <Check
                        v-if="inspectionCopiedStates[inspectionTab]"
                        class="w-3.5 h-3.5 text-green-500"
                      />
                      <Copy
                        v-else
                        class="w-3.5 h-3.5"
                      />
                    </button>
                  </div>

                  <TabsContent value="request-headers">
                    <JsonContent
                      :data="selectedInspectionAttempt.request_headers"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('ui.noRequestHeaders')"
                    />
                  </TabsContent>

                  <TabsContent value="request-body">
                    <JsonContent
                      :data="selectedInspectionAttempt.request_body"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('ui.noRequestBody19667d3')"
                    />
                  </TabsContent>

                  <TabsContent value="response-headers">
                    <JsonContent
                      :data="selectedInspectionAttempt.response_headers"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('ui.noResponseHeaders')"
                    />
                  </TabsContent>

                  <TabsContent value="response-body">
                    <JsonContent
                      :data="selectedInspectionAttempt.response_body"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('ui.noResponseBody0f51f71')"
                    />
                  </TabsContent>
                </div>
              </Tabs>
            </div>
          </Card>
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="emit('close')"
      >
        {{ showSetup ? t('common.cancel') : t('ui.close') }}
      </Button>
      <Button
        v-if="showResult"
        variant="outline"
        @click="emit('back')"
      >
        {{ t('ui.back') }}
      </Button>
    </template>
  </Dialog>

  <Dialog
    :open="Boolean(activeImagePreview)"
    size="6xl"
    :z-index="120"
    @update:open="(val: boolean) => { if (!val) activeImagePreview = null }"
  >
    <template #header>
      <div class="border-b border-border px-6 py-4">
        <div class="text-lg font-semibold text-foreground leading-tight">
          {{ activeImagePreview?.label || t('ui.imagePreview') }}
        </div>
      </div>
    </template>

    <div class="flex max-h-[76vh] items-center justify-center overflow-auto rounded-md bg-muted/20 p-3">
      <img
        v-if="activeImagePreview"
        :src="activeImagePreview.src"
        :alt="activeImagePreview.label"
        class="max-h-[72vh] max-w-full rounded-md object-contain"
      >
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="activeImagePreview = null"
      >
        {{ t('ui.close') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref, watch } from 'vue'
import { Check, Code2, Copy, Loader2, Maximize2, Minimize2, RotateCcw } from 'lucide-vue-next'
import {
  Badge,
  Card,
  Dialog,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
} from '@/components/ui'
import Button from '@/components/ui/button.vue'
import Textarea from '@/components/ui/textarea.vue'
import MultiSelect from '@/components/common/MultiSelect.vue'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import type { TestAttemptDetail, TestModelFailoverResponse } from '@/api/endpoints/providers'
import type { RequestTrace } from '@/api/requestTrace'
import type { MultiSelectOption } from '@/components/common/MultiSelect.vue'
import JsonContent from '@/features/usage/components/RequestDetailDrawer/JsonContent.vue'
import { useClipboard } from '@/composables/useClipboard'
import { useDarkMode } from '@/composables/useDarkMode'
import {
  formatModelTestDiagnostic,
} from './model-test-request'
import type { ModelTestImagePreview } from './model-test-request'
import {
  deriveSummaryFromAttempts,
  detailTabs,
  statusVariant,
  statusLabel,
  statusDisplay,
  attemptRowClass,
  maskKey,
  formatAuthType,
  formatAttemptIndex,
  formatTraceCandidateIndex,
  formatTraceCandidateAccount,
  extractAttemptRequestModel,
  traceCandidateDetail,
  attemptDetail,
  attemptImagePreviews,
  inspectionKey,
  hasDebugData,
  getInspectionTabData,
  formatJsonDraft,
} from './model-test-display'
import type { SummaryView, SummaryBadgeItem } from './model-test-display'

type TestEndpointOption = {
  id: string
  api_format: string
  base_url: string
  is_active: boolean
}

type TestModelMappingOption = {
  name: string
  priority?: number
}

type TestKeyOption = MultiSelectOption

const props = defineProps<{
  open: boolean
  result: TestModelFailoverResponse | null
  mode?: 'global' | 'direct' | 'pool'
  providerType?: string | null
  selectingModelName?: string | null
  requestedModelName?: string | null
  endpoints?: TestEndpointOption[]
  selectedEndpoint?: TestEndpointOption | null
  testing?: boolean
  trace?: RequestTrace | null
  requestId?: string | null
  requestHeadersDraft?: string
  requestHeadersResetValue?: string
  requestHeadersError?: string | null
  requestBodyDraft?: string
  requestBodyResetValue?: string
  requestBodyError?: string | null
  modelMappingAvailable?: boolean
  modelMappingOptions?: TestModelMappingOption[]
  selectedModelMapping?: string | null
  keyOptions?: TestKeyOption[]
  selectedKeyIds?: string[]
  keyOptionsLoading?: boolean
  startDisabled?: boolean
}>()

const emit = defineEmits<{
  close: []
  back: []
  start: []
  selectEndpoint: [endpointId: string]
  selectModelMapping: [modelName: string]
  'update:selectedKeyIds': [value: string[]]
  'update:requestHeadersDraft': [value: string]
  'update:requestBodyDraft': [value: string]
}>()

const { t } = useI18n()

const endpoints = computed(() => props.endpoints ?? [])
const modelMappingOptions = computed(() => props.modelMappingOptions ?? [])
const keyOptions = computed(() => props.keyOptions ?? [])
const selectedKeyIds = computed(() => props.selectedKeyIds ?? [])
const keyOptionsLoading = computed(() => props.keyOptionsLoading === true)
const modelMappingAvailable = computed(
  () => props.modelMappingAvailable === true && modelMappingOptions.value.length > 0,
)
const showKeySelector = computed(() => (
  props.keyOptions !== undefined || keyOptionsLoading.value || selectedKeyIds.value.length > 0
))
const keySelectorPlaceholder = computed(() => (
  keyOptionsLoading.value && keyOptions.value.length === 0 ? t('ui.loadingKeys') : t('ui.defaultSchedulingNoSpecificKey')
))
const keySelectionStatus = computed(() => {
  if (selectedKeyIds.value.length > 0) return t('providers.migration.selected7c3915', { value: String(selectedKeyIds.value.length) })
  if (keyOptionsLoading.value) return t('ui.loading')
  return t('ui.default844b8cc')
})
const requestedModelName = computed(() => props.requestedModelName?.trim() || '')
const selectedModelMapping = computed(() => props.selectedModelMapping?.trim() || '')
const selectedModelMappingValue = computed(() => (
  selectedModelMapping.value || requestedModelName.value
))
const requestHeadersDraft = computed(() => props.requestHeadersDraft ?? '')
const requestBodyDraft = computed(() => props.requestBodyDraft ?? '')
const traceCandidates = computed(() => props.trace?.candidates ?? [])
const showSetup = computed(() => props.open && !props.testing && !props.result)
const showResult = computed(() => !!props.result)
const resultSummary = computed(() => deriveSummaryFromAttempts(props.result))
const showCandidateDiagnostics = computed(() => resultSummary.value.total_candidates > 1)
const showAttemptDiagnostics = computed(() => (
  showCandidateDiagnostics.value
))
const { isDark } = useDarkMode()
const { copyToClipboard } = useClipboard()

function handleModelMappingValueChange(value: string) {
  emit(
    'selectModelMapping',
    value === requestedModelName.value ? '' : value,
  )
}

const dialogTitle = computed(() => {
  if (props.result) return t('ui.modelTestResults')
  return t('ui.modelTest')
})

const dialogDescription = computed(() => {
  if (showSetup.value && props.selectingModelName) {
    return t('providers.migration.selectAnEndpointAndEdite74516', { value: String(props.selectingModelName) })
  }
  if (props.testing && props.selectedEndpoint) {
    return t('providers.migration.testingViadfc444', { value: String(formatApiFormat(props.selectedEndpoint.api_format)), value_2: String(props.selectingModelName || t('ui.model')) })
  }
  return ''
})

const resultAttempts = computed(() => props.result?.attempts ?? [])

const hasEffectiveModel = computed(() => {
  if (!props.result) return false
  return resultAttempts.value.some(
    attempt => attempt.effective_model && attempt.effective_model !== props.result?.model,
  )
})

const resultEmptyMessage = computed(() => {
  if (!props.result) return t('ui.noCandidatesAvailableForTesting')
  if (typeof props.result.error === 'string' && props.result.error.trim()) {
    return formatModelTestDiagnostic(props.result.error)
  }
  const rawResult = props.result as TestModelFailoverResponse & { message?: string }
  if (typeof rawResult.message === 'string' && rawResult.message.trim()) {
    return formatModelTestDiagnostic(rawResult.message)
  }
  return t('ui.noCandidatesAvailableForTesting')
})
const showAllAttempts = ref(false)
const inspectionTab = ref<'request-headers' | 'request-body' | 'response-headers' | 'response-body'>('request-body')
const selectedInspectionKey = ref<string | null>(null)
const inspectionExpandDepth = ref(0)
const inspectionCopiedStates = ref<Record<string, boolean>>({})
const activeImagePreview = ref<ModelTestImagePreview | null>(null)

watch(() => props.result, () => {
  showAllAttempts.value = false
  inspectionExpandDepth.value = 0
  inspectionCopiedStates.value = {}
  const defaultAttempt = resultImageAttempt.value
    ?? resultAttempts.value.find(attempt => attempt.status === 'success')
    ?? inspectableAttempts.value[0]
    ?? resultAttempts.value[0]
    ?? null
  selectedInspectionKey.value = defaultAttempt ? inspectionKey(defaultAttempt) : null
  inspectionTab.value = defaultAttempt && attemptImagePreviews(defaultAttempt).length > 0
    ? 'response-body'
    : 'request-body'
})

const shouldCollapseAttempts = computed(() => resultAttempts.value.length > 20)

const visibleAttempts = computed(() => {
  if (!shouldCollapseAttempts.value || showAllAttempts.value) {
    return resultAttempts.value
  }
  return resultAttempts.value.slice(0, 20)
})

const liveTraceSummary = computed(() => {
  const summary: SummaryView = {
    total_candidates: traceCandidates.value.length,
    attempted: 0,
    available: 0,
    pending: 0,
    success: 0,
    failed: 0,
    skipped: 0,
    unused: 0,
    completed: 0,
    stop_reason: traceCandidates.value.length > 0 ? 'pending' : 'no_candidate',
  }

  for (const candidate of traceCandidates.value) {
    if (candidate.status === 'available') summary.available += 1
    if (candidate.status === 'unused') summary.unused += 1
    if (candidate.status === 'pending' || candidate.status === 'streaming') summary.pending += 1
    if (candidate.status === 'success') summary.success += 1
    if (candidate.status === 'failed' || candidate.status === 'cancelled' || candidate.status === 'stream_interrupted') summary.failed += 1
    if (candidate.status === 'skipped') summary.skipped += 1
  }

  summary.attempted = summary.pending + summary.success + summary.failed
  summary.completed = summary.success + summary.failed + summary.skipped + summary.unused
  if (summary.success > 0) summary.stop_reason = 'first_success'
  if (summary.success === 0 && summary.pending === 0 && summary.failed + summary.skipped > 0) summary.stop_reason = 'exhausted'
  return summary
})

const normalizedProviderType = computed(() => (
  props.providerType
  || props.result?.provider?.provider_type
  || ''
).toLowerCase())

const testScenario = computed<'pool' | 'custom' | 'direct' | 'provider'>(() => {
  if (props.mode === 'pool') return 'pool'
  if (normalizedProviderType.value === 'custom') return 'custom'
  if (props.mode === 'direct') return 'direct'
  return 'provider'
})

const candidateNoun = computed(() => {
  if (testScenario.value === 'pool') return t('ui.account')
  if (testScenario.value === 'custom') return 'Key'
  return t('ui.candidate')
})

const liveEntityLabel = computed(() => {
  if (testScenario.value === 'pool') return t('ui.currentAccount')
  if (testScenario.value === 'custom') return t('ui.currentKey2126a46')
  return t('ui.currentCandidate')
})

const resultEntityLabel = computed(() => {
  if (testScenario.value === 'pool') return t('ui.matchedAccount')
  if (testScenario.value === 'custom') return t('ui.matchedKey')
  return t('ui.matchedCandidate')
})

function buildSummaryItems(summary: SummaryView): SummaryBadgeItem[] {
  const totalLabel = testScenario.value === 'pool'
    ? t('ui.candidateAccount')
    : testScenario.value === 'custom'
      ? t('ui.candidateKey')
      : t('ui.candidate')
  const successLabel = testScenario.value === 'pool'
    ? t('ui.matchedAccount')
    : testScenario.value === 'custom'
      ? t('ui.matchedKey')
      : t('ui.success')
  const failedLabel = testScenario.value === 'pool' ? t('ui.failover8cd0d36') : t('ui.failed28384d7')
  const skippedLabel = testScenario.value === 'pool' ? t('ui.skippedByScheduler') : t('ui.skip')
  const unusedLabel = testScenario.value === 'pool' ? t('ui.notRunAfterSuccess') : t('ui.notRun')

  return [
    { key: 'total', label: totalLabel, value: summary.total_candidates, variant: 'secondary' },
    { key: 'attempted', label: t('ui.attempted'), value: summary.attempted, variant: 'outline' },
    { key: 'success', label: successLabel, value: summary.success, variant: 'success' },
    { key: 'failed', label: failedLabel, value: summary.failed, variant: 'destructive' },
    { key: 'skipped', label: skippedLabel, value: summary.skipped, variant: 'secondary' },
    { key: 'unused', label: unusedLabel, value: summary.unused, variant: 'secondary' },
  ]
}

const liveSummaryItems = computed(() => buildSummaryItems(liveTraceSummary.value))
const resultSummaryItems = computed(() => buildSummaryItems(resultSummary.value))

const liveProgressText = computed(() => {
  if (liveTraceSummary.value.total_candidates <= 0) return t('ui.preparingb53be09')
  return `${liveTraceSummary.value.completed}/${liveTraceSummary.value.total_candidates}`
})

const liveProgressPercent = computed(() => {
  if (liveTraceSummary.value.total_candidates <= 0) return 8
  const raw = Math.round((liveTraceSummary.value.completed / liveTraceSummary.value.total_candidates) * 100)
  return Math.min(100, Math.max(raw, liveTraceSummary.value.pending > 0 ? 12 : 6))
})

const activeTraceCandidate = computed(() => {
  const preferredStatuses = ['pending', 'streaming', 'failed', 'success', 'skipped', 'cancelled']
  for (let index = traceCandidates.value.length - 1; index >= 0; index -= 1) {
    const candidate = traceCandidates.value[index]
    if (preferredStatuses.includes(candidate.status)) return candidate
  }
  return traceCandidates.value[0] ?? null
})

const liveAccountTitle = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return t('ui.waitingForCandidate')
  return candidate.key_account_label || candidate.key_name || candidate.key_preview || '-'
})

const liveAccountMeta = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return ''
  const parts: string[] = []
  if (candidate.key_auth_type) parts.push(formatAuthType(candidate.key_auth_type))
  if (candidate.key_oauth_plan_type) parts.push(candidate.key_oauth_plan_type)
  if (candidate.key_preview && candidate.key_preview !== candidate.key_account_label) parts.push(candidate.key_preview)
  return parts.join(' · ')
})

const liveStatusTitle = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return t('ui.preparingTestRequest')
  if (candidate.status === 'pending' || candidate.status === 'streaming') {
    return t('providers.migration.requestinge6d7f8', { value: String(formatTraceCandidateIndex(candidate)) })
  }
  if (candidate.status === 'success') {
    return t('providers.migration.matched5ee71f', { value: String(formatTraceCandidateIndex(candidate)) })
  }
  if (candidate.status === 'unused') {
    return t('ui.notRunAfterSuccess')
  }
  return statusLabel(candidate.status)
})

const liveStatusDetail = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return ''
  return traceCandidateDetail(candidate)
})

const liveRecentCandidates = computed(() => {
  return traceCandidates.value
    .filter(candidate => candidate.status !== 'available')
    .slice(-4)
    .reverse()
})

const inspectableAttempts = computed(() => {
  return resultAttempts.value.filter(hasDebugData)
})

const showDebugInspector = computed(() => {
  return inspectableAttempts.value.length > 0
})

const selectedInspectionAttempt = computed(() => {
  const key = selectedInspectionKey.value
  if (key) {
    const fromKey = resultAttempts.value.find(attempt => inspectionKey(attempt) === key)
    if (fromKey) return fromKey
  }

  return inspectableAttempts.value[0] ?? resultAttempts.value[0] ?? null
})

const resultImageAttempt = computed(() => {
  return resultAttempts.value.find(attempt => attempt.status === 'success' && attemptImagePreviews(attempt).length > 0)
    ?? resultAttempts.value.find(attempt => attemptImagePreviews(attempt).length > 0)
    ?? null
})

const resultWinningTitle = computed(() => {
  const summary = resultSummary.value
  const keyName = summary.winning_key_name || summary.winning_key_id
  if (keyName) return keyName
  if (props.result?.success) return t('providers.migration.matched24a4ab', { value: String(candidateNoun.value) })
  return t('providers.migration.no3230fd', { value: String(resultEntityLabel.value) })
})

const resultDispatchTitle = computed(() => {
  const summary = resultSummary.value
  return t('providers.migration.attempted17ff8b', { value: String(summary.attempted), value_2: String(summary.total_candidates) })
})

const attemptedEffectiveModelTitle = computed(() => {
  const successAttempt = resultAttempts.value.find(
    attempt => attempt.status === 'success' && attempt.effective_model,
  )
  if (successAttempt?.effective_model) return successAttempt.effective_model

  const attempted = resultAttempts.value.find(
    attempt => !['skipped', 'available', 'unused'].includes(attempt.status) && attempt.effective_model,
  )
  if (attempted?.effective_model) return attempted.effective_model

  return resultAttempts.value.find(attempt => attempt.effective_model)?.effective_model ?? null
})

const attemptedRequestModelTitle = computed(() => {
  const successAttempt = resultAttempts.value.find(
    attempt => attempt.status === 'success' && extractAttemptRequestModel(attempt),
  )
  const successRequestModel = successAttempt ? extractAttemptRequestModel(successAttempt) : null
  if (successRequestModel) return successRequestModel

  const attempted = resultAttempts.value.find(
    attempt => !['skipped', 'available', 'unused'].includes(attempt.status)
      && extractAttemptRequestModel(attempt),
  )
  const attemptedRequestModel = attempted ? extractAttemptRequestModel(attempted) : null
  if (attemptedRequestModel) return attemptedRequestModel

  const anyAttempt = resultAttempts.value.find(attempt => extractAttemptRequestModel(attempt))
  return anyAttempt ? extractAttemptRequestModel(anyAttempt) : null
})

const resultModelTitle = computed(() => {
  return attemptedRequestModelTitle.value
    || resultSummary.value.winning_effective_model
    || attemptedEffectiveModelTitle.value
    || props.result?.model
    || props.selectingModelName
    || '-'
})

function openImagePreview(preview: ModelTestImagePreview) {
  activeImagePreview.value = preview
}

function selectInspectionAttempt(attempt: TestAttemptDetail) {
  selectedInspectionKey.value = inspectionKey(attempt)
  inspectionTab.value = attemptImagePreviews(attempt).length > 0 ? 'response-body' : 'request-body'
}

function copyInspectionContent(tabName: typeof detailTabs[number]['name']) {
  const data = getInspectionTabData(tabName, selectedInspectionAttempt.value)
  if (data === null || data === undefined || data === '') return

  const text = typeof data === 'string' ? data : JSON.stringify(data, null, 2)
  copyToClipboard(text, false)
  inspectionCopiedStates.value[tabName] = true
  setTimeout(() => {
    inspectionCopiedStates.value[tabName] = false
  }, 2000)
}

function expandInspectionContent() {
  inspectionExpandDepth.value = 999
}

function collapseInspectionContent() {
  inspectionExpandDepth.value = 0
}

function formatRequestHeadersDraft() {
  formatJsonDraft(requestHeadersDraft.value, value => emit('update:requestHeadersDraft', value), '{}')
}

function formatRequestBodyDraft() {
  formatJsonDraft(requestBodyDraft.value, value => emit('update:requestBodyDraft', value))
}

function resetRequestHeadersDraft() {
  emit('update:requestHeadersDraft', props.requestHeadersResetValue ?? '{}')
}

function resetRequestBodyDraft() {
  emit('update:requestBodyDraft', props.requestBodyResetValue ?? '')
}

</script>

<style scoped>
.content-block :deep(.rounded-2xl) {
  border: none !important;
  border-radius: 0 !important;
  box-shadow: none !important;
}
</style>
