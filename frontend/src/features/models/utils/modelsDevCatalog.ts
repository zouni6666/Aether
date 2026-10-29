import type { ModelsDevModelItem } from '@/api/models-dev'

// 在线模型目录（models.dev）在预设面板中的分组与展示

export function formatTokenLimit(value: number): string {
  if (value >= 1_000_000) {
    return `${Number((value / 1_000_000).toFixed(1))}M`
  }
  if (value >= 1_000) {
    return `${Number((value / 1_000).toFixed(1))}K`
  }
  return String(value)
}

export function formatModelPrice(value?: number): string {
  if (value === undefined) return '-'
  if (value === 0) return '0'
  const precision = value < 0.01 ? 4 : value < 1 ? 3 : 2
  return value.toFixed(precision).replace(/\.?0+$/, '')
}

// 按提供商分组的模型
export interface ModelsDevProviderGroup {
  providerId: string
  providerName: string
  models: ModelsDevModelItem[]
}

const PROVIDER_PRIORITY_KEYWORDS = [
  ['anthropic', 'claude'],
  ['openai'],
  ['google', 'gemini'],
]

function getProviderPriority(group: ModelsDevProviderGroup): number {
  const searchableText = `${group.providerId} ${group.providerName}`.toLowerCase()
  const priority = PROVIDER_PRIORITY_KEYWORDS.findIndex(keywords => (
    keywords.some(keyword => searchableText.includes(keyword))
  ))
  return priority === -1 ? PROVIDER_PRIORITY_KEYWORDS.length : priority
}

export function getDefaultProviderId(groups: ModelsDevProviderGroup[]): string | null {
  const claudeProvider = groups.find(group => getProviderPriority(group) === 0)
  return claudeProvider?.providerId ?? groups[0]?.providerId ?? null
}

/** 图片加载失败时直接隐藏，避免出现破图图标 */
export function hideBrokenImage(event: Event) {
  const img = event.target as HTMLImageElement
  img.style.display = 'none'
}

/**
 * 把在线目录按提供商分组：无搜索词时只保留官方提供商，有搜索词时支持空格分隔的多关键词 AND 匹配。
 * Claude / OpenAI / Google 固定排在最前，搜索时提供商名直接命中的组优先。
 */
export function groupModelsDevCatalog(
  catalog: ModelsDevModelItem[],
  searchQuery: string,
): ModelsDevProviderGroup[] {
  const visible = searchQuery ? catalog : catalog.filter(m => m.official)
  let models = visible.filter(m => !m.deprecated)
  if (searchQuery) {
    const keywords = searchQuery.toLowerCase().split(/\s+/).filter(k => k.length > 0)
    models = models.filter(model => {
      const searchableText = `${model.providerId} ${model.providerName} ${model.modelId} ${model.modelName} ${model.family || ''}`.toLowerCase()
      return keywords.every(keyword => searchableText.includes(keyword))
    })
  }

  // 按提供商分组
  const groups = new Map<string, ModelsDevProviderGroup>()
  for (const model of models) {
    if (!groups.has(model.providerId)) {
      groups.set(model.providerId, {
        providerId: model.providerId,
        providerName: model.providerName,
        models: []
      })
    }
    groups.get(model.providerId)?.models.push(model)
  }

  // 转换为数组并排序
  const result = Array.from(groups.values())

  const searchKeywords = searchQuery.toLowerCase().split(/\s+/).filter(k => k.length > 0)
  result.sort((a, b) => {
    // 搜索时，优先展示提供商名称或 ID 直接匹配的结果
    if (searchKeywords.length > 0) {
      const aText = `${a.providerId} ${a.providerName}`.toLowerCase()
      const bText = `${b.providerId} ${b.providerName}`.toLowerCase()
      const aProviderMatch = searchKeywords.some(keyword => aText.includes(keyword))
      const bProviderMatch = searchKeywords.some(keyword => bText.includes(keyword))
      if (aProviderMatch && !bProviderMatch) return -1
      if (!aProviderMatch && bProviderMatch) return 1
    }

    // Claude（Anthropic）、OpenAI、Google 固定排在最前
    const priorityDifference = getProviderPriority(a) - getProviderPriority(b)
    if (priorityDifference !== 0) return priorityDifference
    return a.providerName.localeCompare(b.providerName)
  })

  return result
}
