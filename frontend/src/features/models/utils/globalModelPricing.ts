import type { GlobalModelResponse } from '@/api/global-models'
import { sortResolutionEntries } from '@/utils/form'

// 全局模型列表里的定价摘要展示

// 从 GlobalModel 的 default_tiered_pricing 获取第一阶梯价格
export function getFirstTierPrice(model: GlobalModelResponse, type: 'input' | 'output'): number | null {
  const tiered = model.default_tiered_pricing
  if (!tiered?.tiers?.length) return null
  const firstTier = tiered.tiers[0]
  if (type === 'input') {
    return firstTier.input_price_per_1m || null
  }
  return firstTier.output_price_per_1m || null
}

// 检测是否有阶梯计费（多于一个阶梯）
export function hasTieredPricing(model: GlobalModelResponse): boolean {
  const tiered = model.default_tiered_pricing
  return (tiered?.tiers?.length || 0) > 1
}

// 检测是否有视频分辨率计费配置
export function hasVideoPricing(model: GlobalModelResponse): boolean {
  const priceByResolution = model.config?.billing?.video?.price_per_second_by_resolution
  return !!priceByResolution && typeof priceByResolution === 'object' && Object.keys(priceByResolution).length > 0
}

// 获取视频计费的显示文本（如：720p $0.1/s [多分辨率]）
export function getVideoPricingDisplay(model: GlobalModelResponse): string {
  const priceByResolution = model.config?.billing?.video?.price_per_second_by_resolution
  if (!priceByResolution || typeof priceByResolution !== 'object') return ''
  const entries = sortResolutionEntries(Object.entries(priceByResolution))
  if (entries.length === 0) return ''
  // 获取最低分辨率和价格
  const [firstRes, firstPrice] = entries[0]
  const priceStr = `${firstRes} $${(firstPrice as number).toFixed(2)}/s`
  // 如果有多个分辨率，添加标记
  if (entries.length > 1) {
    return `${priceStr} [${entries.length}种]`
  }
  return priceStr
}

// 获取视频计费详情的 tooltip
export function getVideoPricingTooltip(model: GlobalModelResponse): string {
  const priceByResolution = model.config?.billing?.video?.price_per_second_by_resolution
  if (!priceByResolution || typeof priceByResolution !== 'object') return ''
  const entries = sortResolutionEntries(Object.entries(priceByResolution))
  return entries.map(([res, price]) => `${res}: $${(price as number).toFixed(4)}/s`).join('\n')
}
