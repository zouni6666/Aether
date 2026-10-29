import { translate } from '@/i18n'
export type RoutingConditionOp = 'eq' | 'ne' | 'in' | 'contains' | 'exists' | 'matches'

export interface RoutingConditionLeaf {
  field: string
  op: RoutingConditionOp
  value?: unknown
}

export interface RoutingConditionGroup {
  all?: RoutingCondition[]
  any?: RoutingCondition[]
  not?: RoutingCondition
}

export type RoutingCondition = RoutingConditionLeaf | RoutingConditionGroup

export const routingConditionFieldLabels: Record<string, string> = {
  get model() { return translate('ui.model') },
  get api_format() { return translate('ui.apiFormat') },
  get user_id() { return translate('auth.role.user') },
  api_key_id: 'API Key',
}

export const routingConditionOpLabels: Record<RoutingConditionOp, string> = {
  get eq() { return translate('ui.equals') },
  get ne() { return translate('ui.doesNotEqual') },
  get in() { return translate('routing.conditions.in') },
  get contains() { return translate('ui.contains') },
  get exists() { return translate('ui.exists') },
  get matches() { return translate('ui.match') },
}

export function isConditionLeaf(condition: RoutingCondition): condition is RoutingConditionLeaf {
  return typeof (condition as RoutingConditionLeaf).field === 'string'
}

export function summarizeRoutingCondition(condition: RoutingCondition): string {
  if (isConditionLeaf(condition)) {
    const field = routingConditionFieldLabels[condition.field] ?? condition.field
    const op = routingConditionOpLabels[condition.op] ?? condition.op
    return `${field} ${op} ${formatConditionValue(condition.value)}`
  }

  if (condition.all?.length) {
    return condition.all.map(summarizeRoutingCondition).join(translate('routing.conditions.andSeparator'))
  }

  if (condition.any?.length) {
    return condition.any.map(summarizeRoutingCondition).join(translate('routing.conditions.orSeparator'))
  }

  if (condition.not) {
    return translate('routing.conditions.not', { condition: summarizeRoutingCondition(condition.not) })
  }

  return translate('routing.conditions.unconditional')
}

function formatConditionValue(value: unknown): string {
  if (Array.isArray(value)) {
    return value.map(formatConditionValue).join(', ')
  }

  if (value === undefined || value === null) {
    return ''
  }

  if (typeof value === 'object') {
    return JSON.stringify(value)
  }

  return String(value)
}
