<template>
  <Button
    variant="ghost"
    size="icon"
    class="h-8 w-8"
    :class="active ? 'text-primary' : ''"
    :disabled="loading && !active"
    :aria-pressed="active"
    :title="title"
    @click="handleClick"
  >
    <RefreshCcw
      class="w-3.5 h-3.5"
      :class="loading || active ? 'animate-spin' : ''"
    />
  </Button>
</template>

<script setup lang="ts">
import { Button } from '@/components/ui'
import { RefreshCcw } from 'lucide-vue-next'

interface Props {
  loading?: boolean
  active?: boolean
  title?: string
}

interface Emits {
  (e: 'click'): void
}

withDefaults(defineProps<Props>(), {
  loading: false,
  active: undefined,
  title: '刷新'
})

const emit = defineEmits<Emits>()

function handleClick() {
  emit('click')
}
</script>
