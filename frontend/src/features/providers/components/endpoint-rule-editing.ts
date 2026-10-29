import type { ProviderEndpoint, HeaderRule, BodyRule, BodyRuleRegexReplace } from '@/api/endpoints'
import { conditionEquals, conditionToEditable, editableConditionToApi } from '@/features/providers/components/endpoint-rule-condition'
import type { EditableConditionNode } from '@/features/providers/components/endpoint-rule-condition'
import { endpointSecretMarkerPayload, retainsEndpointSecret } from '@/features/providers/components/endpoint-secret-markers'

// 编辑用的规则类型（统一的可编辑结构）
export interface EditableRule {
  action: 'set' | 'drop' | 'rename'
  enabled: boolean
  key: string      // set/drop 用
  value: string    // set 用
  retainValue: boolean
  from: string     // rename 用
  to: string       // rename 用
  condition: EditableConditionNode | null
}

// 编辑用的请求体规则类型
export type BodyRuleAction = 'set' | 'drop' | 'rename' | 'append' | 'insert' | 'regex_replace'

// 请求体规则行里可直接编辑的文本字段
export type BodyRuleEditableField = 'path' | 'value' | 'from' | 'to' | 'index' | 'pattern' | 'replacement' | 'flags' | 'count'

export interface EditableBodyRule {
  action: BodyRuleAction
  enabled: boolean
  path: string     // set/drop/append/insert/regex_replace 用
  value: string    // set/append/insert 用（JSON 格式）
  retainValue: boolean
  from: string     // rename 用
  to: string       // rename 用
  index: string    // insert 用（字符串输入，保存时解析为 int）
  pattern: string  // regex_replace 用
  retainPattern: boolean
  replacement: string // regex_replace 用
  retainReplacement: boolean
  flags: string    // regex_replace 用（i/m/s）
  count: string    // regex_replace 用（空=默认全部；0=全部）
  condition: EditableConditionNode | null
}

export const RESPONSE_HEADER_RULES_CONFIG_KEY = 'response_header_rules'

export const RESPONSE_HEADER_RULES_CAMEL_CONFIG_KEY = 'responseHeaderRules'

// {{$original}} 占位符处理
export const ORIGINAL_PLACEHOLDER = '{{$original}}'

export const ORIGINAL_SENTINEL = '__AETHER_ORIGINAL__'

// 将 {{$original}} 替换为合法 JSON 以便 JSON.parse 校验
// 处理三种写法：裸占位符 {{$original}}、带引号 "{{$original}}"、引号内拼接 "prefix_{{$original}}_suffix"
export function prepareValueForJsonParse(raw: string): string {
  // Step 1: 纯文本替换占位符为 sentinel
  const result = raw.replaceAll(ORIGINAL_PLACEHOLDER, ORIGINAL_SENTINEL)

  // Step 2: 尝试直接 parse（占位符在引号内时已经是合法 JSON）
  try { JSON.parse(result); return result } catch { /* sentinel not in valid JSON position */ }

  // Step 3: 有裸 sentinel 不在引号内，需要扫描并补引号
  let out = ''
  let inStr = false
  let i = 0
  while (i < result.length) {
    if (result[i] === '\\' && inStr) {
      out += result[i] + (result[i + 1] || '')
      i += 2
      continue
    }
    if (result[i] === '"') {
      inStr = !inStr
      out += result[i]
      i++
      continue
    }
    if (!inStr && result.startsWith(ORIGINAL_SENTINEL, i)) {
      out += `"${  ORIGINAL_SENTINEL  }"`
      i += ORIGINAL_SENTINEL.length
      continue
    }
    out += result[i]
    i++
  }
  return out
}

// 递归还原: 将 sentinel 字符串还原为 {{$original}}
export function restoreOriginalPlaceholder(value: unknown): unknown {
  if (typeof value === 'string') {
    if (value === ORIGINAL_SENTINEL) return ORIGINAL_PLACEHOLDER
    if (value.includes(ORIGINAL_SENTINEL)) {
      return value.replaceAll(ORIGINAL_SENTINEL, ORIGINAL_PLACEHOLDER)
    }
    return value
  }
  if (Array.isArray(value)) return value.map(item => restoreOriginalPlaceholder(item))
  if (value !== null && typeof value === 'object') {
    const result: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) result[k] = restoreOriginalPlaceholder(v)
    return result
  }
  return value
}

export function initBodyRuleSetValueForEditor(value: unknown): { value: string } {
  if (value === undefined) return { value: '' }

  // 所有值都用 JSON 格式回显
  try {
    return { value: JSON.stringify(value) }
  } catch {
    return { value: String(value) }
  }
}

export function emptyHeaderRule(): EditableRule {
  return {
    action: 'set',
    enabled: true,
    key: '',
    value: '',
    retainValue: false,
    from: '',
    to: '',
    condition: null,
  }
}

export function editableHeaderRulesFromRules(rules: HeaderRule[] | null | undefined): EditableRule[] {
  if (!Array.isArray(rules)) return []
  const editableRules: EditableRule[] = []
  for (const rule of rules) {
    if (rule.action === 'set') {
      editableRules.push({
        ...emptyHeaderRule(),
        action: 'set',
        enabled: rule.enabled !== false,
        key: rule.key,
        value: rule.value || '',
        retainValue: retainsEndpointSecret(rule.value, rule.has_value),
        condition: conditionToEditable(rule.condition),
      })
    } else if (rule.action === 'drop') {
      editableRules.push({ ...emptyHeaderRule(), action: 'drop', enabled: rule.enabled !== false, key: rule.key, condition: conditionToEditable(rule.condition) })
    } else if (rule.action === 'rename') {
      editableRules.push({ ...emptyHeaderRule(), action: 'rename', enabled: rule.enabled !== false, from: rule.from, to: rule.to, condition: conditionToEditable(rule.condition) })
    }
  }
  return editableRules
}

export function emptyBodyRule(action: BodyRuleAction = 'set'): EditableBodyRule {
  return {
    action,
    enabled: true,
    path: '',
    value: '',
    retainValue: false,
    from: '',
    to: '',
    index: '',
    pattern: '',
    retainPattern: false,
    replacement: '',
    retainReplacement: false,
    flags: '',
    count: '',
    condition: null,
  }
}

export function editableBodyRulesFromRules(rules: BodyRule[] | null | undefined): EditableBodyRule[] {
  const bodyRules: EditableBodyRule[] = []
  if (!Array.isArray(rules)) return bodyRules

  for (const rule of rules) {
    if (rule.action === 'set') {
      const { value } = initBodyRuleSetValueForEditor(rule.value)
      bodyRules.push({ ...emptyBodyRule('set'), enabled: rule.enabled !== false, path: rule.path, value, retainValue: retainsEndpointSecret(rule.value, rule.has_value), condition: conditionToEditable(rule.condition) })
    } else if (rule.action === 'drop') {
      bodyRules.push({ ...emptyBodyRule('drop'), enabled: rule.enabled !== false, path: rule.path, condition: conditionToEditable(rule.condition) })
    } else if (rule.action === 'rename') {
      bodyRules.push({ ...emptyBodyRule('rename'), enabled: rule.enabled !== false, from: rule.from, to: rule.to, condition: conditionToEditable(rule.condition) })
    } else if (rule.action === 'append') {
      const { value } = initBodyRuleSetValueForEditor(rule.value)
      bodyRules.push({ ...emptyBodyRule('append'), enabled: rule.enabled !== false, path: rule.path || '', value, retainValue: retainsEndpointSecret(rule.value, rule.has_value), condition: conditionToEditable(rule.condition) })
    } else if (rule.action === 'insert') {
      const { value } = initBodyRuleSetValueForEditor(rule.value)
      bodyRules.push({ ...emptyBodyRule('insert'), enabled: rule.enabled !== false, path: rule.path || '', value, retainValue: retainsEndpointSecret(rule.value, rule.has_value), index: String(rule.index ?? ''), condition: conditionToEditable(rule.condition) })
    } else if (rule.action === 'regex_replace') {
      bodyRules.push({
        ...emptyBodyRule('regex_replace'),
        enabled: rule.enabled !== false,
        path: rule.path || '',
        pattern: rule.pattern || '',
        retainPattern: retainsEndpointSecret(rule.pattern, rule.has_pattern),
        replacement: rule.replacement || '',
        retainReplacement: retainsEndpointSecret(rule.replacement, rule.has_replacement),
        flags: rule.flags || '',
        count: rule.count === undefined || rule.count === null ? '' : String(rule.count),
        condition: conditionToEditable(rule.condition),
      })
    }
  }
  return bodyRules
}

// 获取值验证状态：true=有效, false=无效, null=空
export function getBodySetValueValidation(rule: EditableBodyRule): boolean | null {
  if (rule.action !== 'set' && rule.action !== 'append' && rule.action !== 'insert') return null
  const raw = rule.value.trim()
  if (!raw) return null
  try {
    JSON.parse(prepareValueForJsonParse(raw))
    return true
  } catch {
    return false
  }
}

// 正则表达式验证状态：true=有效, false=无效, null=空
export function getRegexPatternValidation(rule: EditableBodyRule): boolean | null {
  if (rule.action !== 'regex_replace') return null
  if (rule.retainPattern) return true
  const pattern = rule.pattern.trim()
  if (!pattern) return null
  try {
    new RegExp(pattern)
    // 校验 flags
    const flags = rule.flags.trim()
    if (flags) {
      const validFlags = new Set(['i', 'm', 's'])
      for (const f of flags) {
        if (!validFlags.has(f)) return false
      }
    }
    return true
  } catch {
    return false
  }
}

export function isStrictIntegerString(value: string): boolean {
  return /^-?\d+$/.test(value.trim())
}

export function isStrictNonNegativeIntegerString(value: string): boolean {
  return /^\d+$/.test(value.trim())
}

// 判断请求体规则是否有效（必填字段已填写）
export function isBodyRuleEffective(r: EditableBodyRule): boolean {
  switch (r.action) {
    case 'set':
    case 'drop':
      return !!r.path.trim()
    case 'rename':
      return !!(r.from.trim() && r.to.trim())
    case 'append':
      return !!r.path.trim()
    case 'insert':
      return !!(r.path.trim() && r.index.trim())
    case 'regex_replace':
      return !!(r.path.trim() && r.pattern.trim())
    default:
      return false
  }
}

// 将可编辑请求体规则数组转换为 API 需要的 BodyRule[]
export function rulesToBodyRules(rules: EditableBodyRule[]): BodyRule[] | null {
  const result: BodyRule[] = []

  for (const rule of rules) {
    const condition = editableConditionToApi(rule.condition)
    const common = { ...(rule.enabled ? {} : { enabled: false }), ...(condition ? { condition } : {}) }
    if (rule.action === 'set' && rule.path.trim()) {
      let value: unknown = rule.value
      try { value = restoreOriginalPlaceholder(JSON.parse(prepareValueForJsonParse(rule.value.trim()))) } catch { value = rule.value }
      result.push({ action: 'set', path: rule.path.trim(), value, ...endpointSecretMarkerPayload('has_value', rule.retainValue, value), ...common })
    } else if (rule.action === 'drop' && rule.path.trim()) {
      result.push({ action: 'drop', path: rule.path.trim(), ...common })
    } else if (rule.action === 'rename' && rule.from.trim() && rule.to.trim()) {
      result.push({ action: 'rename', from: rule.from.trim(), to: rule.to.trim(), ...common })
    } else if (rule.action === 'append' && rule.path.trim()) {
      let value: unknown = rule.value
      try { value = restoreOriginalPlaceholder(JSON.parse(prepareValueForJsonParse(rule.value.trim()))) } catch { value = rule.value }
      result.push({ action: 'append', path: rule.path.trim(), value, ...endpointSecretMarkerPayload('has_value', rule.retainValue, value), ...common })
    } else if (rule.action === 'insert' && rule.path.trim()) {
      let value: unknown = rule.value
      try { value = restoreOriginalPlaceholder(JSON.parse(prepareValueForJsonParse(rule.value.trim()))) } catch { value = rule.value }
      if (!isStrictIntegerString(rule.index)) continue
      const idx = parseInt(rule.index.trim(), 10)
      result.push({ action: 'insert', path: rule.path.trim(), index: idx, value, ...endpointSecretMarkerPayload('has_value', rule.retainValue, value), ...common })
    } else if (rule.action === 'regex_replace' && rule.path.trim() && rule.pattern.trim()) {
      const entry: BodyRuleRegexReplace = {
        action: 'regex_replace',
        path: rule.path.trim(),
        pattern: rule.pattern,
        replacement: rule.replacement || '',
        ...endpointSecretMarkerPayload('has_pattern', rule.retainPattern, rule.pattern),
        ...endpointSecretMarkerPayload('has_replacement', rule.retainReplacement, rule.replacement),
        ...(rule.flags.trim() ? { flags: rule.flags.trim() } : {}),
        ...(isStrictNonNegativeIntegerString(rule.count) ? { count: parseInt(rule.count.trim(), 10) } : {}),
      }
      result.push({ ...entry, ...common })
    }
  }

  return result.length > 0 ? result : null
}

// 检查端点规则是否有修改
export function editableHeaderRulesChanged(edited: EditableRule[], originalRules: HeaderRule[]): boolean {
  const editedRules = edited.filter(r => {
    if (r.action === 'set' || r.action === 'drop') return r.key.trim()
    if (r.action === 'rename') return r.from.trim() && r.to.trim()
    return false
  })
  if (editedRules.length !== originalRules.length) return true
  for (let i = 0; i < editedRules.length; i++) {
    const edited = editedRules[i]
    const original = originalRules[i]
    if (!original) return true
    if (edited.action !== original.action) return true
    if (edited.enabled !== (original.enabled !== false)) return true
    if (edited.action === 'set' && original.action === 'set') {
      if (edited.key !== original.key || edited.value !== (original.value || '')) return true
      if (edited.retainValue !== retainsEndpointSecret(original.value, original.has_value)) return true
    } else if (edited.action === 'drop' && original.action === 'drop') {
      if (edited.key !== original.key) return true
    } else if (edited.action === 'rename' && original.action === 'rename') {
      if (edited.from !== original.from || edited.to !== original.to) return true
    }
    if (!conditionEquals(edited.condition, conditionToEditable(original.condition))) return true
  }
  return false
}

// 将可编辑规则数组转换为 API 需要的 HeaderRule[]
export function rulesToHeaderRules(rules: EditableRule[]): HeaderRule[] | null {
  const result: HeaderRule[] = []

  for (const rule of rules) {
    const condition = editableConditionToApi(rule.condition)
    const common = { ...(rule.enabled ? {} : { enabled: false }), ...(condition ? { condition } : {}) }
    if (rule.action === 'set' && rule.key.trim()) {
      result.push({
        action: 'set',
        key: rule.key.trim(),
        value: rule.value,
        ...endpointSecretMarkerPayload('has_value', rule.retainValue, rule.value),
        ...common,
      })
    } else if (rule.action === 'drop' && rule.key.trim()) {
      result.push({ action: 'drop', key: rule.key.trim(), ...common })
    } else if (rule.action === 'rename' && rule.from.trim() && rule.to.trim()) {
      result.push({ action: 'rename', from: rule.from.trim(), to: rule.to.trim(), ...common })
    }
  }

  return result.length > 0 ? result : null
}

export function endpointConfigWithResponseHeaderRules(endpoint: ProviderEndpoint, rules: HeaderRule[] | null): Record<string, unknown> | null {
  const merged: Record<string, unknown> = { ...(endpoint.config || {}) }
  delete merged[RESPONSE_HEADER_RULES_CONFIG_KEY]
  delete merged[RESPONSE_HEADER_RULES_CAMEL_CONFIG_KEY]
  if (rules && rules.length > 0) {
    merged[RESPONSE_HEADER_RULES_CONFIG_KEY] = rules
  }
  return Object.keys(merged).length > 0 ? merged : null
}
