<template>
  <div
    v-show="!isFullyHidden"
    class="gemini-star-cluster absolute inset-0 overflow-hidden pointer-events-none"
    :class="{ 'scattering': isScattering, 'fading-out': isFadingOut }"
  >
    <!-- SVG Defs for the Gemini multi-color gradient -->
    <svg
      class="absolute w-0 h-0 overflow-hidden"
      aria-hidden="true"
    >
      <defs>
        <!-- Main Gemini gradient (blue base with color overlays) -->
        <linearGradient
          id="gemini-base"
          x1="0%"
          y1="0%"
          x2="100%"
          y2="100%"
        >
          <stop
            offset="0%"
            stop-color="#1A73E8"
          />
          <stop
            offset="50%"
            stop-color="#4285F4"
          />
          <stop
            offset="100%"
            stop-color="#669DF6"
          />
        </linearGradient>
        <!-- Red accent overlay - from top -->
        <linearGradient
          id="gemini-red-overlay"
          x1="50%"
          y1="0%"
          x2="50%"
          y2="50%"
        >
          <stop
            offset="0%"
            stop-color="#EA4335"
          />
          <stop
            offset="100%"
            stop-color="#EA4335"
            stop-opacity="0"
          />
        </linearGradient>
        <!-- Yellow accent overlay - from left -->
        <linearGradient
          id="gemini-yellow-overlay"
          x1="0%"
          y1="50%"
          x2="50%"
          y2="50%"
        >
          <stop
            offset="0%"
            stop-color="#FBBC04"
          />
          <stop
            offset="100%"
            stop-color="#FBBC04"
            stop-opacity="0"
          />
        </linearGradient>
        <!-- Green accent overlay - from bottom -->
        <linearGradient
          id="gemini-green-overlay"
          x1="50%"
          y1="100%"
          x2="50%"
          y2="50%"
        >
          <stop
            offset="0%"
            stop-color="#34A853"
          />
          <stop
            offset="100%"
            stop-color="#34A853"
            stop-opacity="0"
          />
        </linearGradient>
        <!-- Glow filter -->
        <filter
          id="star-glow"
          x="-50%"
          y="-50%"
          width="200%"
          height="200%"
        >
          <feGaussianBlur
            stdDeviation="2"
            result="blur"
          />
          <feFlood
            flood-color="#4285F4"
            flood-opacity="0.3"
          />
          <feComposite
            in2="blur"
            operator="in"
          />
          <feMerge>
            <feMergeNode />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>
    </svg>

    <!-- Layer 1: Far background stars (smallest, lowest z-index) -->
    <div class="stars-layer far-layer">
      <div
        v-for="star in farStars"
        :key="`far-${star.id}`"
        class="star-wrapper"
        :class="{ 'star-visible': star.visible && hasScattered }"
        :style="getStarStyle(star)"
      >
        <svg
          viewBox="0 0 24 24"
          class="star-svg"
        >
          <path
            :d="starPath"
            fill="url(#gemini-base)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-red-overlay)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-yellow-overlay)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-green-overlay)"
          />
        </svg>
      </div>
    </div>

    <!-- Layer 2: Mid-distance stars -->
    <div class="stars-layer mid-layer">
      <div
        v-for="star in midStars"
        :key="`mid-${star.id}`"
        class="star-wrapper"
        :class="{ 'star-visible': star.visible && hasScattered }"
        :style="getStarStyle(star)"
      >
        <svg
          viewBox="0 0 24 24"
          class="star-svg"
          style="filter: url(#star-glow)"
        >
          <path
            :d="starPath"
            fill="url(#gemini-base)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-red-overlay)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-yellow-overlay)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-green-overlay)"
          />
        </svg>
      </div>
    </div>

    <!-- Layer 3: Near stars (largest, highest z-index - in front, not occluded by small stars) -->
    <div class="stars-layer near-layer">
      <div
        v-for="star in nearStars"
        :key="`near-${star.id}`"
        class="star-wrapper"
        :class="{ 'star-visible': star.visible && hasScattered }"
        :style="getStarStyle(star)"
      >
        <svg
          viewBox="0 0 24 24"
          class="star-svg"
          style="filter: url(#star-glow)"
        >
          <path
            :d="starPath"
            fill="url(#gemini-base)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-red-overlay)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-yellow-overlay)"
          />
          <path
            :d="starPath"
            fill="url(#gemini-green-overlay)"
          />
        </svg>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, type CSSProperties } from 'vue'

// Props for transition control
const props = withDefaults(defineProps<{
  isVisible?: boolean
}>(), {
  isVisible: true
})

// Gemini star SVG path (4-point star) - viewBox 0 0 24 24
const starPath = 'M12 1.5c.2 3.4 1.4 6.4 3.8 8.8 2.4 2.4 5.4 3.6 8.8 3.8-3.4.2-6.4 1.4-8.8 3.8-2.4 2.4-3.6 5.4-3.8 8.8-.2-3.4-1.4-6.4-3.8-8.8-2.4-2.4-5.4-3.6-8.8-3.8 3.4-.2 6.4-1.4 8.8-3.8 2.4-2.4 3.6-5.4 3.8-8.8z'

interface Star {
  id: number
  x: number
  y: number
  targetX: number
  targetY: number
  size: number
  baseOpacity: number
  visible: boolean
  zIndex: number
  // Animation props
  twinkleDuration: number
  twinkleDelay: number
}

const farStars = ref<Star[]>([])
const midStars = ref<Star[]>([])
const nearStars = ref<Star[]>([])
const hasScattered = ref(false)
const isScattering = ref(false)
const isFadingOut = ref(false)
const isFullyHidden = ref(false)

const CENTER_X = 50
const CENTER_Y = 50

// Generate constrained position to keep stars within bounds
const getConstrainedPosition = (size: number): { x: number; y: number } => {
  // Calculate padding based on star size (in percentage)
  // Assuming container is roughly 400-600px wide, use a safe estimate
  const paddingPercent = Math.max(3, (size / 5))
  const minPos = paddingPercent
  const maxPos = 100 - paddingPercent

  return {
    x: minPos + Math.random() * (maxPos - minPos),
    y: minPos + Math.random() * (maxPos - minPos)
  }
}

const createStar = (id: number, sizeRange: [number, number, number, number, number, number], opacityBase: number, opacityRange: number, visibleThreshold: number, zIndexBase: number): Star => {
  const sizeVariant = Math.random()
  let size: number
  if (sizeVariant < 0.4) {
    size = sizeRange[0] + Math.random() * sizeRange[1]
  } else if (sizeVariant < 0.7) {
    size = sizeRange[2] + Math.random() * sizeRange[3]
  } else {
    size = sizeRange[4] + Math.random() * sizeRange[5]
  }

  const { x: targetX, y: targetY } = getConstrainedPosition(size)

  return {
    id,
    x: CENTER_X,
    y: CENTER_Y,
    targetX,
    targetY,
    size,
    baseOpacity: opacityBase + Math.random() * opacityRange,
    visible: true,
    zIndex: zIndexBase + Math.round(size),
    twinkleDuration: 3 + Math.random() * 4, // 3-7s duration
    twinkleDelay: Math.random() * 5 // 0-5s delay
  }
}

const createStarLayers = () => {
  const far: Star[] = []
  for (let i = 0; i < 30; i++) {
    far.push(createStar(i, [6, 4, 10, 6, 14, 6], 0.2, 0.25, 0.35, 0))
  }
  farStars.value = far

  const mid: Star[] = []
  for (let i = 0; i < 18; i++) {
    mid.push(createStar(i, [18, 8, 24, 10, 32, 10], 0.35, 0.35, 0.45, 30))
  }
  midStars.value = mid

  const near: Star[] = []
  for (let i = 0; i < 10; i++) {
    near.push(createStar(i, [40, 15, 55, 20, 70, 25], 0.5, 0.5, 0.55, 100))
  }
  nearStars.value = near
}

// Removed: handleAnimationIteration was causing stars to "teleport" visibly
// Stars now stay in place and only twinkle without changing position

// Scatter stars from center to their target positions
const scatterStars = () => {
  isScattering.value = true
  hasScattered.value = true

  const allStars = [...farStars.value, ...midStars.value, ...nearStars.value]
  allStars.forEach(star => {
    star.x = star.targetX
    star.y = star.targetY
  })

  setTimeout(() => {
    isScattering.value = false
  }, 2000)
}

// Reset stars to center
const resetStarsToCenter = () => {
  hasScattered.value = false
  isScattering.value = false

  const allStars = [...farStars.value, ...midStars.value, ...nearStars.value]
  allStars.forEach(star => {
    star.x = CENTER_X
    star.y = CENTER_Y
    const { x, y } = getConstrainedPosition(star.size)
    star.targetX = x
    star.targetY = y
  })
}

const getStarStyle = (star: Star): CSSProperties => {
  return {
    left: `${star.x}%`,
    top: `${star.y}%`,
    width: `${star.size}px`,
    height: `${star.size}px`,
    zIndex: star.zIndex,
    '--base-opacity': star.baseOpacity,
    '--twinkle-duration': `${star.twinkleDuration}s`,
    '--twinkle-delay': `${star.twinkleDelay}s`
  } as CSSProperties
}

// Watch for visibility changes
watch(() => props.isVisible, (newVal, oldVal) => {
  if (newVal && !oldVal) {
    // Entering: recreate stars if needed, reset to center then scatter
    isFadingOut.value = false
    isFullyHidden.value = false
    // Recreate stars if they were cleared
    if (farStars.value.length === 0) {
      createStarLayers()
    } else {
      resetStarsToCenter()
    }
    setTimeout(() => {
      scatterStars()
    }, 50)
  } else if (!newVal && oldVal) {
    // Leaving: immediately stop animation and hide to release GPU resources
    hasScattered.value = false
    isScattering.value = false // Force stop scattering transition immediately
    isFadingOut.value = true

    // Wait for fade-out transition (150ms) to complete before cleanup
    // Use a single timeout matching the CSS transition duration
    setTimeout(() => {
      if (!props.isVisible) {
        isFullyHidden.value = true
        isFadingOut.value = false
        // Clear star arrays to fully release memory
        farStars.value = []
        midStars.value = []
        nearStars.value = []
      }
    }, 180) // Slightly longer than CSS transition (150ms) to ensure smooth fade
  }
}, { flush: 'post' }) // Change to post flush to ensure DOM is updated before our cleanup logic runs

onMounted(() => {
  createStarLayers()
  if (props.isVisible) {
    isFullyHidden.value = false
    setTimeout(() => {
      scatterStars()
    }, 100)
  } else {
    // Start hidden if not visible
    isFullyHidden.value = true
  }
})

onUnmounted(() => {
  // No explicit cleanup needed for CSS animations
})
</script>

<style scoped>
.gemini-star-cluster {
  perspective: 800px;
}

.stars-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.far-layer {
  z-index: 1;
}

.mid-layer {
  z-index: 2;
}

.near-layer {
  z-index: 3;
}

.star-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.star-wrapper {
  position: absolute;
  opacity: 0;
  transform: scale(0.3) translate(-50%, -50%);
  transition:
    opacity 0.8s ease-out,
    transform 0.8s ease-out;
  /* Avoid persistent will-change to reduce GPU memory usage */
}

.star-wrapper.star-visible {
  opacity: 0; /* Default to invisible, animation handles opacity */
  transform: scale(1) translate(-50%, -50%);
  animation: twinkle var(--twinkle-duration) ease-in-out infinite;
  animation-delay: var(--twinkle-delay);
}

@keyframes twinkle {
  0% {
    opacity: 0;
    transform: scale(0.5) translate(-50%, -50%);
  }
  50% {
    opacity: var(--base-opacity);
    transform: scale(1) translate(-50%, -50%);
    filter: brightness(1.2);
  }
  100% {
    opacity: 0;
    transform: scale(0.5) translate(-50%, -50%);
  }
}

/* Scatter animation - stars fly outward from center (2s duration) */
.scattering .star-wrapper {
  transition:
    opacity 0.8s ease-out,
    transform 0.8s ease-out,
    left 2s cubic-bezier(0.16, 1, 0.3, 1),
    top 2s cubic-bezier(0.16, 1, 0.3, 1);
  /* Only use will-change during active scatter animation */
  will-change: opacity, transform, left, top;
}

/* Fade out animation - quick fade and disable child transitions */
.fading-out {
  opacity: 0;
  transition: opacity 0.15s ease-out;
  pointer-events: none;
  /* Force GPU layer removal */
  will-change: auto;
}

.fading-out .star-wrapper {
  transition: none !important;
  will-change: auto !important;
  animation: none !important;
  opacity: 0 !important;
}

/* Depth blur effect */
.far-layer .star-wrapper {
  filter: blur(0.5px);
}

.mid-layer .star-wrapper {
  filter: blur(0.2px);
}

.near-layer .star-wrapper {
  filter: none;
}
</style>
