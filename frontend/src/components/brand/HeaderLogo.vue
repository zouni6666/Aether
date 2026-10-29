<template>
  <div
    class="relative group cursor-pointer"
    :class="containerClass"
  >
    <!-- Logo SVG -->
    <svg
      xmlns="http://www.w3.org/2000/svg"
      :viewBox="viewBox"
      class="transition-colors duration-500 ease-out"
      :class="[logoClass]"
    >
      <path
        :d="aetherFullPath"
        fill-rule="evenodd"
        fill="currentColor"
      />
    </svg>
  </div>
</template>

<script setup lang="ts">
import { AETHER_FULL_PATH, AETHER_SVG_VIEWBOX } from '@/constants/logoPaths'

interface Props {
  size?: string
  className?: string
}

const props = withDefaults(defineProps<Props>(), {
  size: 'h-8 w-8',
  className: ''
})

const aetherFullPath = AETHER_FULL_PATH
const viewBox = AETHER_SVG_VIEWBOX
const containerClass = `${props.size} ${props.className}`.trim()
const logoClass = 'w-full h-full'
</script>

<style scoped>
</style>
