
import { translate as t } from '@/i18n'
import type { BodyRuleCondition, BodyRuleConditionOp } from '@/api/endpoints'
import {
  endpointSecretMarkerPayload,
  retainsEndpointSecret,
} from './endpoint-secret-markers'

export type ConditionSource = 'body' | 'original' | 'request_headers'
export type ConditionGroupMode = 'all' | 'any'

export interface EditableConditionLeaf {
  kind: 'leaf'
  path: string
  op: BodyRuleConditionOp
  value: string
  retainValue: boolean
  source: ConditionSource
}

export interface EditableConditionGroup {
  kind: 'group'
  mode: ConditionGroupMode
  children: EditableConditionNode[]
}

export type EditableConditionNode = EditableConditionLeaf | EditableConditionGroup

export const CONDITION_OP_OPTIONS: Array<{ value: BodyRuleConditionOp; label: string }> = [
  { value: 'eq', get label() { return t('ui.equals') } },
  { value: 'neq', get label() { return t('ui.doesNotEqual') } },
  { value: 'gt', get label() { return t('ui.greaterThan') } },
  { value: 'lt', get label() { return t('ui.lessThan') } },
  { value: 'gte', get label() { return t('ui.greaterThanOrEqualTo') } },
  { value: 'lte', get label() { return t('ui.lessThanOrEqualTo') } },
  { value: 'starts_with', get label() { return t('ui.startsWith') } },
  { value: 'ends_with', get label() { return t('ui.endsWith') } },
  { value: 'contains', get label() { return t('ui.contains') } },
  { value: 'matches', get label() { return t('ui.regexMatch') } },
  { value: 'exists', get label() { return t('ui.exists') } },
  { value: 'not_exists', get label() { return t('ui.doesNotExist') } },
  { value: 'in', get label() { return t('ui.inList') } },
  { value: 'type_is', get label() { return t('ui.typeIs') } },
]

const NUMERIC_OPS = new Set(['gt', 'lt', 'gte', 'lte'])
const STRING_OPS = new Set(['starts_with', 'ends_with'])
const TYPE_IS_VALUES = new Set(['string', 'number', 'boolean', 'array', 'object', 'null'])

export function createEmptyConditionLeaf(): EditableConditionLeaf {
  return {
    kind: 'leaf',
    path: '',
    op: 'eq',
    value: '',
    retainValue: false,
    source: 'body',
  }
}

export function createConditionGroup(
  mode: ConditionGroupMode = 'all',
  children: EditableConditionNode[] = [createEmptyConditionLeaf()],
): EditableConditionGroup {
  return {
    kind: 'group',
    mode,
    children,
  }
}

export function cloneEditableCondition(node: EditableConditionNode): EditableConditionNode {
  if (node.kind === 'group') {
    return {
      kind: 'group',
      mode: node.mode,
      children: node.children.map(cloneEditableCondition),
    }
  }
  return { ...node }
}

export function conditionToEditable(condition?: BodyRuleCondition | null): EditableConditionNode | null {
  if (!condition) return null
  if ('all' in condition) {
    return createConditionGroup(
      'all',
      condition.all.map(child => conditionToEditable(child) || createEmptyConditionLeaf()),
    )
  }
  if ('any' in condition) {
    return createConditionGroup(
      'any',
      condition.any.map(child => conditionToEditable(child) || createEmptyConditionLeaf()),
    )
  }
  const source = (condition as { source?: unknown }).source
  return {
    kind: 'leaf',
    path: condition.path || '',
    op: condition.op || 'eq',
    value: condition.value !== undefined
      ? (typeof condition.value === 'string' ? condition.value : JSON.stringify(condition.value))
      : '',
    retainValue: retainsEndpointSecret(condition.value, condition.has_value),
    source: source === 'request_headers' || source === 'headers'
        ? 'request_headers'
        : source === 'original'
          ? 'original'
        : 'body',
  }
}

export function editableConditionToApi(node: EditableConditionNode | null): BodyRuleCondition | undefined {
  if (!node) return undefined

  if (node.kind === 'group') {
    const children = node.children
      .map(child => editableConditionToApi(child))
      .filter((child): child is BodyRuleCondition => !!child)
    if (!children.length) return undefined
    return node.mode === 'all' ? { all: children } : { any: children }
  }

  const path = node.path.trim()
  if (!path) return undefined

  const base = {
    path,
    op: node.op,
    ...(node.source === 'request_headers'
      ? { source: 'request_headers' as const }
      : node.source === 'original'
        ? { source: 'original' as const }
        : {}),
  }

  if (node.op === 'exists' || node.op === 'not_exists') {
    return base
  }

  const raw = node.value.trim()
  const marker = endpointSecretMarkerPayload('has_value', node.retainValue, raw)
  if (!raw) {
    return { ...base, value: '', ...marker }
  }

  try {
    return { ...base, value: JSON.parse(raw), ...marker }
  } catch {
    return { ...base, value: raw, ...marker }
  }
}

export function isConditionValueRequired(op: BodyRuleConditionOp): boolean {
  return op !== 'exists' && op !== 'not_exists'
}

export function getConditionValuePlaceholder(op: BodyRuleConditionOp): string {
  if (op === 'in') return '["a", "b"]'
  if (op === 'type_is') return 'string/number/boolean/...'
  return t('ui.value')
}

export function getBodyRuleConditionPathPlaceholder(path: string): string {
  return path.includes('[*]') || /\[\d+-\d+\]/.test(path) ? t('providers.migration.itemFieldb22c65') : t('ui.fieldPath')
}

export function conditionEquals(
  left: EditableConditionNode | null,
  right: EditableConditionNode | null,
): boolean {
  if (left === right) return true
  if (!left || !right) return false
  if (left.kind !== right.kind) return false

  if (left.kind === 'group' && right.kind === 'group') {
    if (left.mode !== right.mode) return false
    if (left.children.length !== right.children.length) return false
    return left.children.every((child, i) => conditionEquals(child, right.children[i]))
  }

  if (left.kind === 'leaf' && right.kind === 'leaf') {
    return left.path === right.path
      && left.op === right.op
      && left.value === right.value
      && left.retainValue === right.retainValue
      && left.source === right.source
  }

  return false
}

export function validateEditableCondition(node: EditableConditionNode | null): string | null {
  if (!node) return null

  if (node.kind === 'group') {
    if (!node.children.length) return t('ui.aConditionGroupRequiresAtLeastOneCondition')
    for (let i = 0; i < node.children.length; i += 1) {
      const err = validateEditableCondition(node.children[i])
      if (err) return t('providers.migration.childCondition774c00', { value: String(i + 1), value_2: String(err) })
    }
    return null
  }

  const path = node.path.trim()
  if (!path) return t('ui.conditionPathIsRequired')

  if (!isConditionValueRequired(node.op)) return null
  if (node.retainValue) return null

  const raw = node.value.trim()
  let parsed: unknown = raw
  if (raw) {
    try {
      parsed = JSON.parse(raw)
    } catch {
      parsed = raw
    }
  }

  if (NUMERIC_OPS.has(node.op)) {
    if (typeof parsed !== 'number' || Number.isNaN(parsed)) return t('ui.enterANumberForNumericConditions')
    return null
  }

  if (STRING_OPS.has(node.op)) {
    if (typeof parsed !== 'string') return t('ui.theConditionValueMustBeAString')
    return null
  }

  if (node.op === 'matches') {
    if (typeof parsed !== 'string' || !parsed) return t('ui.aRegularExpressionIsRequired')
    try {
      new RegExp(parsed)
      return null
    } catch (error: unknown) {
      return t('providers.migration.invalidRegularExpressionb25ec5', { value: String(error instanceof Error ? error.message : String(error)) })
    }
  }

  if (node.op === 'in') {
    if (!Array.isArray(parsed)) return t('ui.theValueForAnInConditionMustBeAJson')
    return null
  }

  if (node.op === 'type_is') {
    if (typeof parsed !== 'string' || !TYPE_IS_VALUES.has(parsed)) {
      return t('ui.typeIsSupportsOnlyStringNumberBooleanArrayObjectOr')
    }
  }

  return null
}
