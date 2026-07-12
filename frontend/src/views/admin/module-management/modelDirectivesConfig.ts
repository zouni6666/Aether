export interface ReasoningApiFormatConfig {
  enabled: boolean
  mappings: Record<string, unknown>
}

export interface ModelDirectivesConfig {
  reasoning_effort: {
    enabled: boolean
    api_formats: Record<string, ReasoningApiFormatConfig>
  }
}

export const MODEL_DIRECTIVES_MODULE_NAME = 'model_directives'

export const MODEL_DIRECTIVE_API_FORMATS = [
  {
    key: 'openai:chat',
    label: 'OpenAI Chat',
    parameter: 'reasoning_effort',
  },
  {
    key: 'openai:responses',
    label: 'OpenAI Responses',
    parameter: 'reasoning.effort',
  },
  {
    key: 'openai:responses:compact',
    label: 'OpenAI Responses Compact',
    parameter: 'reasoning.effort',
  },
  {
    key: 'claude:messages',
    label: 'Claude Messages',
    parameter: 'output_config.effort + thinking',
  },
  {
    key: 'gemini:generate_content',
    label: 'Gemini GenerateContent',
    parameter: 'generationConfig.thinkingConfig',
  },
] as const

export const DEFAULT_REASONING_SUFFIXES = ['low', 'medium', 'high', 'xhigh', 'max'] as const

function defaultMappingsForApiFormat(apiFormat: string): Record<string, unknown> {
  switch (apiFormat) {
    case 'openai:chat':
      return {
        low: { reasoning_effort: 'low' },
        medium: { reasoning_effort: 'medium' },
        high: { reasoning_effort: 'high' },
        xhigh: { reasoning_effort: 'xhigh' },
        max: { reasoning_effort: 'max' },
      }
    case 'openai:responses':
    case 'openai:responses:compact':
      return {
        low: { reasoning: { effort: 'low' } },
        medium: { reasoning: { effort: 'medium' } },
        high: { reasoning: { effort: 'high' } },
        xhigh: { reasoning: { effort: 'xhigh' } },
        max: { reasoning: { effort: 'max' } },
      }
    case 'claude:messages':
      return {
        low: { thinking: { type: 'enabled', budget_tokens: 1024 } },
        medium: { thinking: { type: 'enabled', budget_tokens: 4096 } },
        high: { thinking: { type: 'enabled', budget_tokens: 8192 } },
        xhigh: { thinking: { type: 'enabled', budget_tokens: 16384 } },
        max: { thinking: { type: 'enabled', budget_tokens: 32768 } },
      }
    case 'gemini:generate_content':
      return {
        low: { generationConfig: { thinkingConfig: { thinkingBudget: 1024 } } },
        medium: { generationConfig: { thinkingConfig: { thinkingBudget: 4096 } } },
        high: { generationConfig: { thinkingConfig: { thinkingBudget: 8192 } } },
        xhigh: { generationConfig: { thinkingConfig: { thinkingBudget: 16384 } } },
        max: { generationConfig: { thinkingConfig: { thinkingBudget: -1 } } },
      }
    default:
      return {}
  }
}

export function createDefaultModelDirectivesConfig(): ModelDirectivesConfig {
  return {
    reasoning_effort: {
      enabled: true,
      api_formats: Object.fromEntries(
        MODEL_DIRECTIVE_API_FORMATS.map((format) => [
          format.key,
          {
            enabled: true,
            mappings: defaultMappingsForApiFormat(format.key),
          },
        ])
      ),
    },
  }
}

function mappingsFromLegacySuffixes(apiFormat: string, value: unknown): Record<string, unknown> {
  if (!Array.isArray(value)) return defaultMappingsForApiFormat(apiFormat)
  const supported = new Set<string>(DEFAULT_REASONING_SUFFIXES)
  const defaults = defaultMappingsForApiFormat(apiFormat)
  return Object.fromEntries(value
    .map((item) => String(item).trim().toLowerCase())
    .filter((item, index, array) => supported.has(item) && array.indexOf(item) === index)
    .map((suffix) => [suffix, defaults[suffix]])
    .filter(([, mapping]) => mapping !== undefined))
}

function normalizeMappings(apiFormat: string, value: unknown, legacySuffixes: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return mappingsFromLegacySuffixes(apiFormat, legacySuffixes)
  }
  return { ...(value as Record<string, unknown>) }
}

export function normalizeModelDirectivesConfig(value: unknown): ModelDirectivesConfig {
  const defaults = createDefaultModelDirectivesConfig()
  if (!value || typeof value !== 'object') return defaults

  const source = value as Partial<ModelDirectivesConfig>
  const reasoning = source.reasoning_effort
  const apiFormats: Record<string, ReasoningApiFormatConfig> = {
    ...defaults.reasoning_effort.api_formats,
  }
  const sourceApiFormats = reasoning?.api_formats
  if (sourceApiFormats && typeof sourceApiFormats === 'object') {
    for (const [apiFormat, rawConfig] of Object.entries(sourceApiFormats)) {
      if (typeof rawConfig === 'boolean') {
        apiFormats[apiFormat] = {
          enabled: rawConfig,
          mappings: defaultMappingsForApiFormat(apiFormat),
        }
      } else if (rawConfig && typeof rawConfig === 'object') {
        const value = rawConfig as Partial<ReasoningApiFormatConfig> & { suffixes?: unknown }
        apiFormats[apiFormat] = {
          enabled: typeof value.enabled === 'boolean' ? value.enabled : true,
          mappings: normalizeMappings(apiFormat, value.mappings, value.suffixes),
        }
      }
    }
  }

  return {
    reasoning_effort: {
      enabled:
        typeof reasoning?.enabled === 'boolean'
          ? reasoning.enabled
          : defaults.reasoning_effort.enabled,
      api_formats: apiFormats,
    },
  }
}
