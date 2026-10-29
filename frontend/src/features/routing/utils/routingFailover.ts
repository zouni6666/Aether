import { translate as t } from '@/i18n'

export const MAX_ROUTING_FAILOVER_RULES = 64
export const MAX_ROUTING_FAILOVER_PATTERN_BYTES = 4096

export interface RoutingFailoverRule {
  pattern: string
  status_codes: number[]
}

export interface RoutingFailoverRules {
  success_failover_patterns: RoutingFailoverRule[]
  error_stop_patterns: RoutingFailoverRule[]
}

export interface RoutingFailoverPolicy {
  max_transfer_count: number
  max_transfer_timeout_seconds: number
  failover_rules: RoutingFailoverRules
}

export function normalizeRoutingFailoverPolicy(value?: Partial<RoutingFailoverPolicy>): RoutingFailoverPolicy {
  const rules = value?.failover_rules
  const cloneRules = (entries?: RoutingFailoverRule[]): RoutingFailoverRule[] => (
    Array.isArray(entries) ? entries.map(rule => ({
      pattern: String(rule.pattern ?? ''),
      status_codes: Array.isArray(rule.status_codes) ? [...rule.status_codes] : [],
    })) : []
  )
  return {
    max_transfer_count: Number(value?.max_transfer_count ?? 0),
    max_transfer_timeout_seconds: Number(value?.max_transfer_timeout_seconds ?? 0),
    failover_rules: {
      success_failover_patterns: cloneRules(rules?.success_failover_patterns),
      error_stop_patterns: cloneRules(rules?.error_stop_patterns),
    },
  }
}

export function validateRoutingFailoverPolicy(policy: RoutingFailoverPolicy): string | null {
  for (const [name, value] of [
    [t('ui.globalMaximumFailovers'), policy.max_transfer_count],
    [t('ui.globalMaximumFailoverTime'), policy.max_transfer_timeout_seconds],
  ] as const) {
    if (!Number.isSafeInteger(value) || value < 0) return t('routing.validation.nonnegative', { name })
  }
  for (const [name, entries, success] of [
    [t('ui.successfulResponseFailoverRules'), policy.failover_rules.success_failover_patterns, true],
    [t('ui.errorTerminationRules'), policy.failover_rules.error_stop_patterns, false],
  ] as const) {
    if (entries.length > MAX_ROUTING_FAILOVER_RULES) return t('routing.validation.tooManyRules', { name, count: MAX_ROUTING_FAILOVER_RULES })
    for (const [index, rule] of entries.entries()) {
      if (!rule.pattern.trim() && (success || rule.status_codes.length === 0)) {
        return success
          ? t('routing.validation.patternRequired', { name, index: index + 1 })
          : t('routing.validation.statusOrPatternRequired', { name, index: index + 1 })
      }
      if (new TextEncoder().encode(rule.pattern.trim()).length > MAX_ROUTING_FAILOVER_PATTERN_BYTES) {
        return t('routing.validation.patternTooLong', { name, index: index + 1 })
      }
      if (rule.status_codes.some(status => !Number.isInteger(status) || (success ? status !== 200 : status < 400 || status > 599))) {
        return t('routing.validation.invalidStatusCodes', { name, index: index + 1, statuses: success ? '200' : '400–599' })
      }
    }
  }
  return null
}
