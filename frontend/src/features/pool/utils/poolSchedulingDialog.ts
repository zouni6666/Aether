export interface SchedulingDialogPresetLike {
  mutexGroup: string | null
}

export interface SchedulingDialogSelectablePresetLike extends SchedulingDialogPresetLike {
  enabled: boolean
  applicable: boolean
}

export function mergePoolAdvancedPatch(
  current: unknown,
  patch: Record<string, unknown>,
): Record<string, unknown> {
  const currentRecord = typeof current === 'object' && current !== null && !Array.isArray(current)
    ? current as Record<string, unknown>
    : {}
  return {
    ...currentRecord,
    ...patch,
  }
}

export function normalizeMutexSelection<T extends SchedulingDialogSelectablePresetLike>(
  items: readonly T[],
): T[] {
  const next = items.map(item => ({ ...item }))
  const groups = new Map<string, number[]>()

  next.forEach((item, index) => {
    if (!item.mutexGroup) return
    const indexes = groups.get(item.mutexGroup) ?? []
    indexes.push(index)
    groups.set(item.mutexGroup, indexes)
  })

  for (const indexes of groups.values()) {
    const winner = indexes.find(index => next[index].enabled && next[index].applicable)
    indexes.forEach((index) => {
      next[index].enabled = winner !== undefined && index === winner && next[index].applicable
    })
  }

  return next
}

export function moveStrategyItem<T extends SchedulingDialogPresetLike>(
  items: readonly T[],
  itemIndex: number,
  direction: -1 | 1,
): T[] {
  const strategyIndexes: number[] = []

  items.forEach((item, index) => {
    if (!item.mutexGroup) {
      strategyIndexes.push(index)
    }
  })

  const currentPosition = strategyIndexes.indexOf(itemIndex)
  if (currentPosition === -1) {
    return [...items]
  }

  const targetPosition = currentPosition + direction
  if (targetPosition < 0 || targetPosition >= strategyIndexes.length) {
    return [...items]
  }

  const sourceIndex = strategyIndexes[currentPosition]
  const targetIndex = strategyIndexes[targetPosition]
  const nextItems = [...items]

  ;[nextItems[sourceIndex], nextItems[targetIndex]] = [nextItems[targetIndex], nextItems[sourceIndex]]

  return nextItems
}
