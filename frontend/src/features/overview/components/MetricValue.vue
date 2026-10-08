<template>
  <span
    ref="container"
    class="block min-w-0"
    :title="value"
  ><span
    ref="text"
    class="inline-block whitespace-nowrap"
    :style="{ fontSize: `${fontSize}px` }"
  >{{ value }}</span></span>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'

const props = withDefaults(defineProps<{ value: string; maxFontSize?: number }>(), {
  maxFontSize: 20,
})
const container = ref<HTMLElement>()
const text = ref<HTMLElement>()
const fontSize = ref(props.maxFontSize)
let observer: ResizeObserver | null = null
let frame: number | null = null
let mounted = false

function fit() {
  frame = null
  if (!text.value) return
  const available = container.value?.clientWidth || 0
  const width = text.value.getBoundingClientRect().width
  const renderedFontSize = Number.parseFloat(getComputedStyle(text.value).fontSize)
  if (!available || !width || !Number.isFinite(renderedFontSize)) return

  // Measure at the rendered size so fitting never briefly restores an oversized value.
  fontSize.value = Math.max(10, Math.min(
    props.maxFontSize,
    Math.floor(available * renderedFontSize / width * 100) / 100,
  ))
}

function scheduleFit() {
  if (mounted && frame === null) frame = requestAnimationFrame(fit)
}

onMounted(() => {
  mounted = true
  if (typeof ResizeObserver !== 'undefined') {
    let previousWidth = -1
    observer = new ResizeObserver(([entry]) => {
      const width = entry?.contentRect.width
      if (width === undefined || width === previousWidth) return
      previousWidth = width
      scheduleFit()
    })
    if (container.value) observer.observe(container.value)
  }
  void document.fonts?.ready.then(scheduleFit)
  document.fonts?.addEventListener('loadingdone', scheduleFit)
  scheduleFit()
})

onUnmounted(() => {
  mounted = false
  observer?.disconnect()
  if (frame !== null) cancelAnimationFrame(frame)
  document.fonts?.removeEventListener('loadingdone', scheduleFit)
})

watch(() => [props.value, props.maxFontSize], scheduleFit, { flush: 'post' })
</script>
