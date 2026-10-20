<template>
  <div
    class="app-shell"
    :class="{ 'pt-24': showNotice }"
  >
    <div
      v-if="showNotice"
      class="fixed top-6 left-0 right-0 z-50 flex justify-center px-4"
    >
      <slot name="notice" />
    </div>

    <div class="app-shell__backdrop">
      <div class="app-shell__gradient app-shell__gradient--primary" />
      <div class="app-shell__gradient app-shell__gradient--accent" />
    </div>

    <div class="app-shell__body">
      <aside
        v-if="$slots.sidebar"
        class="app-shell__sidebar"
        :class="sidebarClass"
        :style="sidebarStyle"
      >
        <slot name="sidebar" />
      </aside>

      <div
        class="app-shell__content"
        :class="contentClass"
      >
        <slot name="header" />
        <main :class="mainClass">
          <slot />
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type CSSProperties } from 'vue'

const props = withDefaults(defineProps<{
  showNotice?: boolean
  contentClass?: string
  mainClass?: string
  sidebarClass?: string
  sidebarStyle?: CSSProperties
}>(), {
  showNotice: false,
  contentClass: '',
  mainClass: '',
  sidebarClass: '',
  sidebarStyle: () => ({}),
})

const showNotice = computed(() => props.showNotice)

// contentClass and mainClass are now just the props, base classes are in template
const contentClass = computed(() => props.contentClass)
const mainClass = computed(() => ['app-shell__main', props.mainClass].filter(Boolean).join(' '))
const sidebarClass = computed(() => props.sidebarClass)
</script>
