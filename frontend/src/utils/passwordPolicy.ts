import { translate, type MessageKey } from '@/i18n'

export type PasswordPolicyLevel = 'weak' | 'medium' | 'strong'
export const PASSWORD_MAX_BYTES = 72

const textEncoder = new TextEncoder()

function getPasswordByteLength(password: string): number {
  return textEncoder.encode(password).length
}

export const PASSWORD_POLICY_OPTIONS: Array<{
  value: PasswordPolicyLevel
  label: string
  description: string
}> = [
  {
    value: 'weak',
    get label() { return translate('feedback.password.weak') },
    get description() { return translate('feedback.password.minCharacters', { count: 6 }) },
  },
  {
    value: 'medium',
    get label() { return translate('feedback.password.medium') },
    get description() { return translate('feedback.password.optionMedium') },
  },
  {
    value: 'strong',
    get label() { return translate('feedback.password.strong') },
    get description() { return translate('feedback.password.optionStrong') },
  },
]

export function normalizePasswordPolicyLevel(value: unknown): PasswordPolicyLevel {
  return value === 'medium' || value === 'strong' ? value : 'weak'
}

export function getPasswordPolicyHint(level: unknown): string {
  switch (normalizePasswordPolicyLevel(level)) {
    case 'medium': return translate('feedback.password.hintMedium')
    case 'strong': return translate('feedback.password.hintStrong')
    default: return translate('feedback.password.minCharacters', { count: 6 })
  }
}

export function getPasswordPolicyPlaceholder(level: unknown): string {
  switch (normalizePasswordPolicyLevel(level)) {
    case 'medium': return translate('feedback.password.placeholderMedium')
    case 'strong': return translate('feedback.password.placeholderStrong')
    default: return translate('feedback.password.minCharacters', { count: 6 })
  }
}

interface PasswordPolicyError {
  key: MessageKey
  params?: Record<string, string | number>
}

function getPasswordPolicyErrorMessages(password: string, level: unknown): PasswordPolicyError[] {
  if (!password) return []

  const normalized = normalizePasswordPolicyLevel(level)
  const errors: PasswordPolicyError[] = []
  if (getPasswordByteLength(password) > PASSWORD_MAX_BYTES) {
    errors.push({ key: 'feedback.password.maxBytes', params: { count: PASSWORD_MAX_BYTES } })
  }

  const minLen = normalized === 'weak' ? 6 : 8
  if (password.length < minLen) {
    errors.push({ key: 'feedback.password.minCharacters', params: { count: minLen } })
  }

  if (normalized === 'medium') {
    if (!/[A-Za-z]/.test(password)) errors.push({ key: 'feedback.password.letters' })
    if (!/[0-9]/.test(password)) errors.push({ key: 'feedback.password.numbers' })
  }

  if (normalized === 'strong') {
    if (!/[A-Z]/.test(password)) errors.push({ key: 'feedback.password.uppercase' })
    if (!/[a-z]/.test(password)) errors.push({ key: 'feedback.password.lowercase' })
    if (!/[0-9]/.test(password)) errors.push({ key: 'feedback.password.numbers' })
    if (!/[!@#$%^&*()_+\-=[\]{};:'",.<>?/\\|`~]/.test(password)) errors.push({ key: 'feedback.password.special' })
  }

  return errors
}

export function getPasswordPolicyErrors(password: string, level: unknown): string[] {
  return getPasswordPolicyErrorMessages(password, level).map(({ key, params }) => translate(key, params))
}

export function validatePasswordByPolicy(password: string, level: unknown): string {
  const errors = getPasswordPolicyErrorMessages(password, level)
  if (errors.length === 0) return ''
  if (errors.length === 1 && errors[0].key === 'feedback.password.maxBytes') {
    return translate('feedback.password.passwordMaxBytes', { count: PASSWORD_MAX_BYTES })
  }
  const requirements = errors
    .map(({ key, params }) => translate(key, params))
    .join(translate('feedback.password.separator'))
  return translate('feedback.password.requires', { requirements })
}
