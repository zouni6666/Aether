<template>
  <Teleport to="body">
    <Transition
      name="drawer"
      :appear="appear"
    >
      <div
        v-if="open"
        class="fixed inset-0 flex justify-end"
        :style="{ zIndex }"
      >
        <div
          class="absolute inset-0"
          :class="backdropClass"
          data-drawer-backdrop
          @click="handleBackdropClick"
        />
        <div :class="panelClasses">
          <slot />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'
import { useEscapeKey } from '@/composables/useEscapeKey'

/**
 * 右侧滑出抽屉
 *
 * 统一处理遮罩、滑入过渡、ESC 关闭与背景点击关闭。
 * 内容区默认纵向滚动；需要固定头部加内部滚动时使用 layout="column"。
 */
const props = withDefaults(defineProps<{
  open: boolean
  /** 面板宽度类，默认 700px 且不超过视口 90% */
  width?: string
  /** scroll: 整个面板滚动；column: 面板为纵向 flex 容器，由内部区域滚动 */
  layout?: 'scroll' | 'column'
  /** 追加到面板上的类名，可覆盖默认背景与文字颜色 */
  panelClass?: string
  /** 遮罩类名 */
  backdropClass?: string
  zIndex?: number
  /** 首次渲染即播放滑入动画（用于懒加载组件） */
  appear?: boolean
  closeOnBackdrop?: boolean
  closeOnEscape?: boolean
}>(), {
  width: 'sm:w-[700px] sm:max-w-[90vw]',
  layout: 'scroll',
  panelClass: undefined,
  backdropClass: 'bg-black/30 backdrop-blur-sm',
  zIndex: 50,
  appear: false,
  closeOnBackdrop: true,
  closeOnEscape: true,
})

const emit = defineEmits<{
  close: []
}>()

const panelClasses = computed(() => cn(
  'drawer-panel relative h-full w-full border-l border-border bg-card text-card-foreground shadow-2xl',
  props.layout === 'column' ? 'flex flex-col' : 'overflow-y-auto',
  props.width,
  props.panelClass,
))

function handleBackdropClick() {
  if (props.closeOnBackdrop) emit('close')
}

useEscapeKey(() => {
  if (!props.open || !props.closeOnEscape) return false
  emit('close')
  return true
})
</script>

<style scoped>
.drawer-enter-active,
.drawer-leave-active {
  transition: opacity 0.3s ease;
}

.drawer-enter-active .drawer-panel,
.drawer-leave-active .drawer-panel {
  transition: transform 0.3s ease;
}

.drawer-enter-from,
.drawer-leave-to {
  opacity: 0;
}

.drawer-enter-from .drawer-panel,
.drawer-leave-to .drawer-panel {
  transform: translateX(100%);
}
</style>
