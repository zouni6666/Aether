import { updateGlobalModel, type GlobalModelResponse } from '@/api/global-models'
import { runTasksWithConcurrency } from '@/utils/concurrency'
import { log } from '@/utils/logger'
import {
  getModelsDevPricingSourceFromConfig,
  useModelsDevPricingSources,
  withModelsDevPricingSource,
} from './useModelsDevPricingSources'

/** 本会话内已尝试迁移过的模型，避免列表与批量对话框重复提交同一条更新 */
const attemptedModelIds = new Set<string>()

/**
 * 把早期只存在浏览器本地的在线价格来源写回模型配置。
 * onModelUpdated 在每条写回成功后收到服务端返回的模型，用于同步本地列表。
 */
export function useModelsDevPricingSourceMigration(
  onModelUpdated: (model: GlobalModelResponse) => void,
) {
  const { getLocalSource } = useModelsDevPricingSources()

  function migrateLegacyModelsDevPricingSources(models: GlobalModelResponse[]) {
    const tasks = models.flatMap(model => {
      const localSource = getLocalSource(model.id)
      if (
        !localSource
        || getModelsDevPricingSourceFromConfig(model.config)
        || attemptedModelIds.has(model.id)
      ) {
        return []
      }
      attemptedModelIds.add(model.id)
      return [async () => {
        const nextConfig = withModelsDevPricingSource(model.config, localSource)
        try {
          const updatedModel = await updateGlobalModel(model.id, { config: nextConfig })
          onModelUpdated({ ...updatedModel, config: nextConfig })
        } catch (err: unknown) {
          attemptedModelIds.delete(model.id)
          log.warn('迁移本地模型价格来源失败:', err)
        }
      }]
    })
    if (tasks.length > 0) void runTasksWithConcurrency(tasks, 4)
  }

  return { migrateLegacyModelsDevPricingSources }
}
