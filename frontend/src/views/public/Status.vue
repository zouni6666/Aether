<template>
  <main class="min-h-screen bg-background text-foreground">
    <header class="border-b border-border">
      <div class="mx-auto flex max-w-6xl items-center justify-between px-5 py-4">
        <RouterLink
          to="/"
          class="flex items-center gap-3 font-semibold"
        >
          <HeaderLogo size="h-8 w-8" />{{ siteName }}
        </RouterLink>
        <RouterLink
          to="/"
          class="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft class="h-4 w-4" />首页
        </RouterLink>
      </div>
    </header>
    <div class="mx-auto max-w-6xl px-5 py-7">
      <HealthMonitorView public-page />
    </div>
  </main>
</template>

<script setup lang="ts">
import { ArrowLeft } from 'lucide-vue-next'
import HeaderLogo from '@/components/HeaderLogo.vue'
import { useSiteInfo } from '@/composables/useSiteInfo'
import HealthMonitorView from '@/features/health-monitor/HealthMonitorView.vue'

const { siteName } = useSiteInfo()
</script>
