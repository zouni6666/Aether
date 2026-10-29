import { describe, expect, it } from 'vitest'
import { nextTick, ref } from 'vue'

import { useClientPagination } from '../useClientPagination'

describe('useClientPagination', () => {
  it('slices the current page and reports totals', () => {
    const items = ref([1, 2, 3, 4, 5])
    const { currentPage, pageSize, pageItems, total, totalPages } = useClientPagination(items, { pageSize: 2 })

    expect(total.value).toBe(5)
    expect(totalPages.value).toBe(3)
    expect(pageItems.value).toEqual([1, 2])

    currentPage.value = 3
    expect(pageItems.value).toEqual([5])

    pageSize.value = 10
    expect(totalPages.value).toBe(1)
  })

  it('clamps the page when the list shrinks and resets on demand', async () => {
    const items = ref([1, 2, 3, 4])
    const { currentPage, pageItems, resetPage } = useClientPagination(items, { pageSize: 2 })

    currentPage.value = 2
    expect(pageItems.value).toEqual([3, 4])

    items.value = [1]
    await nextTick()
    expect(currentPage.value).toBe(1)
    expect(pageItems.value).toEqual([1])

    items.value = [1, 2, 3]
    currentPage.value = 2
    resetPage()
    expect(currentPage.value).toBe(1)
  })

  it('accepts a getter source', () => {
    const source = ref(['a', 'b', 'c'])
    const { pageItems } = useClientPagination(() => source.value.filter(v => v !== 'b'), { pageSize: 5 })
    expect(pageItems.value).toEqual(['a', 'c'])
  })
})
