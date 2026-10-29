<template>
  <div class="fixed top-4 left-1/2 -translate-x-1/2 z-[100] flex flex-col items-center gap-2">
    <TransitionGroup
      name="toast"
      tag="div"
      class="flex flex-col items-center gap-2"
    >
      <ToastWithProgress
        v-for="toast in toasts"
        :key="toast.id"
        :toast="toast"
        @remove="removeToast(toast.id)"
      />
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import ToastWithProgress from './ToastWithProgress.vue'
import { useToast } from '@/composables/useToast'

const { toasts, removeToast } = useToast()
</script>

<style scoped>
/* 进入动画 - 从上方弹入 */
.toast-enter-active {
  transition: all 0.4s cubic-bezier(0.2, 0.9, 0.3, 1);
}

.toast-enter-from {
  transform: translateY(-20px) scale(0.95);
  opacity: 0;
}

.toast-enter-to {
  transform: translateY(0) scale(1);
  opacity: 1;
}

/* 弹出动画 - 向上消失 */
.toast-leave-active {
  transition: all 0.2s ease-out;
}

.toast-leave-from {
  transform: translateY(0) scale(1);
  opacity: 1;
}

.toast-leave-to {
  transform: translateY(-20px) scale(0.95);
  opacity: 0;
}

/* 移动动画 */
.toast-move {
  transition: all 0.4s cubic-bezier(0.2, 0.9, 0.3, 1);
}

/* 响应式调整 */
@media (max-width: 640px) {
  div.fixed {
    top: 1rem;
    left: 1rem;
    right: 1rem;
    transform: none;
  }
}
</style>