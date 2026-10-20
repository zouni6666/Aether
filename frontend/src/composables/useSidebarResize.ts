import { onScopeDispose, ref, watch } from 'vue'
import { useLocalStorage } from '@vueuse/core'

export function useSidebarResize() {
  const sidebarMinWidth = 200
  const sidebarMaxWidth = 480
  const sidebarWidth = useLocalStorage('aether-sidebar-width', 260)
  const sidebarResizing = ref(false)
  let startPosition = 0
  let startWidth = 260
  let activePointer: number | null = null
  let previousCursor = ''
  let previousUserSelect = ''

  function clampWidth(width: number) {
    return Number.isFinite(width)
      ? Math.min(sidebarMaxWidth, Math.max(sidebarMinWidth, width))
      : 260
  }

  watch(sidebarWidth, width => {
    const normalized = clampWidth(width)
    if (normalized !== width) sidebarWidth.value = normalized
  }, { immediate: true, flush: 'sync' })

  function moveSidebarResize(event: PointerEvent) {
    if (event.pointerId !== activePointer) return
    sidebarWidth.value = clampWidth(startWidth + event.clientX - startPosition)
  }

  function stopSidebarResize(event?: PointerEvent) {
    if (event && event.pointerId !== activePointer) return
    if (!sidebarResizing.value) return
    sidebarResizing.value = false
    activePointer = null
    document.body.style.cursor = previousCursor
    document.body.style.userSelect = previousUserSelect
    window.removeEventListener('pointermove', moveSidebarResize)
    window.removeEventListener('pointerup', stopSidebarResize)
    window.removeEventListener('pointercancel', stopSidebarResize)
    window.removeEventListener('blur', cancelSidebarResize)
  }

  function cancelSidebarResize() {
    stopSidebarResize()
  }

  function startSidebarResize(event: PointerEvent) {
    if (event.button !== 0 || sidebarResizing.value) return
    event.preventDefault()
    startPosition = event.clientX
    startWidth = sidebarWidth.value
    activePointer = event.pointerId
    sidebarResizing.value = true
    previousCursor = document.body.style.cursor
    previousUserSelect = document.body.style.userSelect
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('pointermove', moveSidebarResize)
    window.addEventListener('pointerup', stopSidebarResize)
    window.addEventListener('pointercancel', stopSidebarResize)
    window.addEventListener('blur', cancelSidebarResize)
  }

  function handleSidebarResizeKey(event: KeyboardEvent) {
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return
    event.preventDefault()
    sidebarWidth.value = clampWidth(sidebarWidth.value + (event.key === 'ArrowRight' ? 10 : -10))
  }

  onScopeDispose(cancelSidebarResize)

  return {
    sidebarWidth, sidebarMinWidth, sidebarMaxWidth, sidebarResizing,
    startSidebarResize, handleSidebarResizeKey,
  }
}
