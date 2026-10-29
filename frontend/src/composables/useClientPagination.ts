import { computed, ref, watch, toValue, type MaybeRefOrGetter } from 'vue'

export interface UseClientPaginationOptions {
  /** 每页条数，默认 20 */
  pageSize?: number
}

/**
 * 客户端分页
 *
 * 对已完整加载的列表按页切片，配合 Pagination 组件使用：
 * `<Pagination :current="currentPage" :total="total" :page-size="pageSize"
 *   @update:current="currentPage = $event" @update:page-size="pageSize = $event" />`
 *
 * 列表缩短导致当前页越界时自动回到最后一页。
 */
export function useClientPagination<T>(
  items: MaybeRefOrGetter<T[]>,
  options: UseClientPaginationOptions = {},
) {
  const currentPage = ref(1)
  const pageSize = ref(options.pageSize ?? 20)

  const total = computed(() => toValue(items).length)
  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize.value)))

  const pageItems = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value
    return toValue(items).slice(start, start + pageSize.value)
  })

  watch(totalPages, (pages) => {
    if (currentPage.value > pages) currentPage.value = pages
  })

  function resetPage(): void {
    currentPage.value = 1
  }

  return { currentPage, pageSize, total, totalPages, pageItems, resetPage }
}
