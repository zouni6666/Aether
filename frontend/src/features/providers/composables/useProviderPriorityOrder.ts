import { computed, nextTick, onScopeDispose, ref, watch, type Ref } from 'vue'
import { useEventListener, useRafFn } from '@vueuse/core'
import { useI18n } from '@/i18n'

interface SortableProvider {
  id: string
  name: string
}

interface ProviderPointerDrag {
  providerId: string
  pointerId: number
  startX: number
  startY: number
  handle: HTMLElement
  scrollContainer: HTMLElement | null
}

export function useProviderPriorityOrder<Provider extends SortableProvider>(
  providers: () => Provider[],
  container: Ref<HTMLElement | null>,
  scheduling: {
    disabled: () => boolean
    move: (providerId: string, targetId: string) => void
  },
) {
  const { legacyT } = useI18n()
  const draggingProviderId = ref<string | null>(null)
  const dropTargetId = ref<string | null>(null)
  const pointerPosition = ref({ clientX: 0, clientY: 0 })
  const announcement = ref('')
  let pointerDrag: ProviderPointerDrag | null = null
  let suppressClickUntil = 0

  const orderedProviders = computed(providers)

  const draggingProvider = computed(() => orderedProviders.value.find(provider => provider.id === draggingProviderId.value))
  const dragPreviewStyle = computed(() => ({
    left: `${Math.max(8, Math.min(pointerPosition.value.clientX + 12, window.innerWidth - 208))}px`,
    top: `${Math.max(8, Math.min(pointerPosition.value.clientY + 12, window.innerHeight - 48))}px`,
  }))

  function moveProvider(providerId: string, targetId: string) {
    if (scheduling.disabled()) return
    scheduling.move(providerId, targetId)
    announcement.value = legacyT('调度顺序已调整，保存后生效')
  }

  function updateDropTarget() {
    const target = document.elementFromPoint(pointerPosition.value.clientX, pointerPosition.value.clientY)
      ?.closest<HTMLElement>('[data-provider-sort-id]')
    const targetId = target?.dataset.providerSortId
    dropTargetId.value = target && container.value?.contains(target)
      && targetId !== draggingProviderId.value
      && orderedProviders.value.some(provider => provider.id === targetId)
      ? targetId ?? null
      : null
  }

  function findScrollContainer(handle: HTMLElement): HTMLElement | null {
    let ancestor = handle.parentElement
    while (ancestor && ancestor !== document.body) {
      if (/(auto|scroll)/.test(getComputedStyle(ancestor).overflowY) && ancestor.scrollHeight > ancestor.clientHeight) {
        return ancestor
      }
      ancestor = ancestor.parentElement
    }
    return document.scrollingElement as HTMLElement | null
  }

  const { pause, resume } = useRafFn(() => {
    const scrollContainer = pointerDrag?.scrollContainer
    if (scrollContainer) {
      const bounds = scrollContainer.getBoundingClientRect()
      const isDocument = scrollContainer === document.scrollingElement
      const top = isDocument ? 0 : Math.max(0, bounds.top)
      const bottom = isDocument ? window.innerHeight : Math.min(window.innerHeight, bounds.bottom)
      const pointerY = pointerPosition.value.clientY
      const pointerX = pointerPosition.value.clientX
      if (isDocument || (pointerX >= bounds.left && pointerX <= bounds.right)) {
        if (pointerY < top + 48) {
          scrollContainer.scrollTop -= Math.min(12, (top + 48 - pointerY) / 4)
        } else if (pointerY > bottom - 48) {
          scrollContainer.scrollTop += Math.min(12, (pointerY - bottom + 48) / 4)
        }
      }
    }
    updateDropTarget()
  }, { immediate: false })

  function cancelDrag() {
    const previous = pointerDrag
    pointerDrag = null
    pause()
    if (draggingProviderId.value) suppressClickUntil = Date.now() + 250
    draggingProviderId.value = null
    dropTargetId.value = null
    if (previous?.handle.hasPointerCapture?.(previous.pointerId)) {
      previous.handle.releasePointerCapture(previous.pointerId)
    }
  }

  function startDrag(providerId: string, event: PointerEvent) {
    if (scheduling.disabled()) return
    if (event.button !== 0 || event.isPrimary === false || orderedProviders.value.length < 2) return
    if (!orderedProviders.value.some(provider => provider.id === providerId)) return
    const handle = event.currentTarget
    if (!(handle instanceof HTMLElement)) return

    cancelDrag()
    event.preventDefault()
    handle.focus({ preventScroll: true })
    pointerDrag = {
      providerId,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      handle,
      scrollContainer: findScrollContainer(handle),
    }
    pointerPosition.value = { clientX: event.clientX, clientY: event.clientY }
    handle.setPointerCapture?.(event.pointerId)
  }

  function handlePointerMove(event: PointerEvent) {
    if (!pointerDrag || pointerDrag.pointerId !== event.pointerId) return
    pointerPosition.value = { clientX: event.clientX, clientY: event.clientY }
    if (!draggingProviderId.value) {
      if (Math.hypot(event.clientX - pointerDrag.startX, event.clientY - pointerDrag.startY) < 5) return
      draggingProviderId.value = pointerDrag.providerId
      resume()
    }
    event.preventDefault()
    updateDropTarget()
  }

  function handlePointerUp(event: PointerEvent) {
    if (!pointerDrag || pointerDrag.pointerId !== event.pointerId) return
    const providerId = draggingProviderId.value
    if (providerId) {
      pointerPosition.value = { clientX: event.clientX, clientY: event.clientY }
      updateDropTarget()
    }
    const targetId = dropTargetId.value
    cancelDrag()
    if (providerId && targetId) moveProvider(providerId, targetId)
  }

  function handleSortKeydown(providerId: string, event: KeyboardEvent) {
    if (event.key === 'Escape') {
      cancelDrag()
      return
    }
    const directions: Record<string, number> = { ArrowUp: -1, ArrowLeft: -1, ArrowDown: 1, ArrowRight: 1 }
    const direction = directions[event.key]
    if (direction === undefined || pointerDrag) return
    event.preventDefault()
    const index = orderedProviders.value.findIndex(provider => provider.id === providerId)
    const target = orderedProviders.value[index + direction]
    if (index < 0 || !target) return
    const handle = event.currentTarget as HTMLElement
    moveProvider(providerId, target.id)
    void nextTick(() => handle.focus({ preventScroll: true }))
  }

  function handleSortClick(event: MouseEvent) {
    if (!draggingProviderId.value && Date.now() >= suppressClickUntil) return
    suppressClickUntil = 0
    event.preventDefault()
    event.stopPropagation()
  }

  function sortItemClass(providerId: string) {
    return {
      'opacity-40': draggingProviderId.value === providerId,
      'ring-2 ring-inset ring-primary/60 bg-primary/5': dropTargetId.value === providerId,
    }
  }

  watch(() => providers().map(provider => provider.id), cancelDrag, { immediate: true })
  useEventListener(window, 'pointermove', handlePointerMove, { passive: false })
  useEventListener(window, 'pointerup', handlePointerUp)
  useEventListener(window, 'pointercancel', (event) => {
    if (pointerDrag?.pointerId === event.pointerId) cancelDrag()
  })
  useEventListener(window, 'lostpointercapture', (event) => {
    if (pointerDrag?.pointerId === event.pointerId) cancelDrag()
  })
  useEventListener(window, 'blur', cancelDrag)
  useEventListener(window, 'keydown', (event) => {
    if (event.key === 'Escape') cancelDrag()
  })
  onScopeDispose(cancelDrag)

  return {
    orderedProviders,
    draggingProvider,
    dragPreviewStyle,
    announcement,
    startDrag,
    cancelDrag,
    handleSortKeydown,
    handleSortClick,
    sortItemClass,
  }
}
