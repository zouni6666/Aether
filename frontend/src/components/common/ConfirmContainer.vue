<template>
  <AlertDialog
    :model-value="state.isOpen"
    :title="state.title"
    :description="state.message"
    :confirm-text="state.confirmText"
    :cancel-text="state.cancelText"
    :type="state.variant || 'question'"
    @update:model-value="handleClose"
    @confirm="handleConfirm"
    @cancel="handleCancel"
  />
</template>

<script setup lang="ts">
import AlertDialog from './AlertDialog.vue'
import { useConfirm } from '@/composables/useConfirm'

const { state, handleConfirm, handleCancel } = useConfirm()

function handleClose(value: boolean) {
  if (!value) {
    handleCancel()
  }
}

</script>
