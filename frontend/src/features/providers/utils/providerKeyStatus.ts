
import { translate as t } from '@/i18n'
import type { ProviderKeyStatusSnapshot } from '@/api/endpoints/types/statusSnapshot'
import { getOAuthExpiresCountdown, type OAuthStatusInfo } from '@/composables/useCountdownTimer'
import {
  classifyAccountBlockLabel,
  cleanAccountBlockReason,
  isAccountLevelBlockReason,
  isRefreshFailedReason,
} from './accountBlock'
import {
  canRefreshOAuthCredential,
  isOAuthManagedCredential,
  type ProviderKeyAuthCarrier,
} from './providerKeyAuth'

export interface ProviderKeyStatusCarrier extends ProviderKeyAuthCarrier {
  oauth_expires_at?: number | null
  oauth_invalid_at?: number | null  // compatibility only
  oauth_invalid_reason?: string | null  // compatibility only
  account_status_label?: string | null  // compatibility only
  account_status_reason?: string | null  // compatibility only
  account_status_blocked?: boolean | null  // compatibility only
  status_snapshot?: ProviderKeyStatusSnapshot | null
}

export interface AccountStatusDisplay {
  code: string
  label: string | null
  reason: string | null
  blocked: boolean
}

function normalizeText(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const text = value.trim()
  return text || null
}

function buildLegacyAccountStatus(input: ProviderKeyStatusCarrier): AccountStatusDisplay {
  const explicitLabel = normalizeText(input.account_status_label)
  if (input.account_status_blocked && explicitLabel) {
    return {
      code: 'legacy',
      label: explicitLabel,
      reason: normalizeText(input.account_status_reason),
      blocked: true,
    }
  }

  const invalidReason = normalizeText(input.oauth_invalid_reason)
  if (!invalidReason || !isAccountLevelBlockReason(invalidReason)) {
    return { code: 'ok', label: null, reason: null, blocked: false }
  }

  const cleaned = cleanAccountBlockReason(invalidReason) || invalidReason
  return {
    code: 'legacy',
    label: classifyAccountBlockLabel(cleaned || invalidReason),
    reason: normalizeText(cleaned),
    blocked: true,
  }
}

export function getAccountStatusDisplay(input: ProviderKeyStatusCarrier): AccountStatusDisplay {
  const snapshot = input.status_snapshot?.account
  if (snapshot) {
    return {
      code: normalizeText(snapshot.code) || 'ok',
      label: normalizeText(snapshot.label),
      reason: normalizeText(snapshot.reason),
      blocked: Boolean(snapshot.blocked),
    }
  }
  return buildLegacyAccountStatus(input)
}

function getSnapshotOAuthState(
  input: ProviderKeyStatusCarrier,
  tick: number,
): OAuthStatusInfo | null {
  const oauth = input.status_snapshot?.oauth
  if (!oauth) return null

  const code = normalizeText(oauth.code) || 'none'
  const expiresAt = oauth.expires_at ?? input.oauth_expires_at ?? null
  const reason = normalizeText(oauth.reason)

  if (code === 'reauth_required') {
    const countdown = expiresAt == null ? null : getOAuthExpiresCountdown(expiresAt, tick, null, null)
    return {
      text: countdown?.text ? t('providers.migration.renewalFaileda307c0', { value: String(countdown.text) }) : t('ui.renewalFailed'),
      isExpired: false,
      isExpiringSoon: countdown?.isExpiringSoon ?? false,
      isInvalid: false,
      invalidReason: reason || undefined,
      requiresReauth: true,
      usableUntilExpiry: true,
    }
  }

  if (code === 'invalid') {
    return {
      text: t('ui.invalid'),
      isExpired: false,
      isExpiringSoon: false,
      isInvalid: true,
      invalidReason: reason || undefined,
    }
  }

  if (code === 'expired') {
    return { text: t('ui.expired'), isExpired: true, isExpiringSoon: false, isInvalid: false }
  }

  if (code === 'check_failed') {
    if (expiresAt == null) return null
    return getOAuthExpiresCountdown(expiresAt, tick, null, null)
  }

  if (expiresAt == null) return null
  return getOAuthExpiresCountdown(expiresAt, tick, null, null)
}

function refreshFailureAccessTokenStillUsable(expiresAt: number | null | undefined): boolean {
  return typeof expiresAt === 'number' && expiresAt > Math.floor(Date.now() / 1000)
}

function getReauthRequiredOAuthState(
  expiresAt: number | null | undefined,
  tick: number,
  reason: string,
): OAuthStatusInfo {
  const countdown = expiresAt == null ? null : getOAuthExpiresCountdown(expiresAt, tick, null, null)
  return {
    text: countdown?.text ? t('providers.migration.renewalFaileda307c0', { value: String(countdown.text) }) : t('ui.renewalFailed'),
    isExpired: false,
    isExpiringSoon: countdown?.isExpiringSoon ?? false,
    isInvalid: false,
    invalidReason: reason,
    requiresReauth: true,
    usableUntilExpiry: true,
  }
}

function getLegacyOAuthState(
  input: ProviderKeyStatusCarrier,
  tick: number,
): OAuthStatusInfo | null {
  if (!isOAuthManagedCredential(input)) return null
  if (!input.oauth_expires_at && !input.oauth_invalid_at && !input.oauth_invalid_reason) return null

  const rawReason = normalizeText(input.oauth_invalid_reason)
  if (
    rawReason
    && isRefreshFailedReason(rawReason)
    && refreshFailureAccessTokenStillUsable(input.oauth_expires_at)
  ) {
    return getReauthRequiredOAuthState(input.oauth_expires_at, tick, rawReason)
  }

  if (rawReason && isAccountLevelBlockReason(rawReason) && !isRefreshFailedReason(rawReason)) {
    if (!input.oauth_expires_at) return null
    return getOAuthExpiresCountdown(input.oauth_expires_at, tick, null, null)
  }

  return getOAuthExpiresCountdown(
    input.oauth_expires_at,
    tick,
    input.oauth_invalid_at,
    input.oauth_invalid_reason,
  )
}

function getOAuthStatusSeverity(status: OAuthStatusInfo | null): number {
  if (!status) return 0
  if (status.isInvalid) return 3
  if (status.isExpired) return 2
  if (status.requiresReauth) return 2
  return 1
}

function mergeOAuthStatusDisplay(
  snapshotStatus: OAuthStatusInfo | null,
  legacyStatus: OAuthStatusInfo | null,
): OAuthStatusInfo | null {
  if (getOAuthStatusSeverity(legacyStatus) > getOAuthStatusSeverity(snapshotStatus)) {
    return legacyStatus
  }

  if (
    snapshotStatus?.isInvalid
    && !snapshotStatus.invalidReason
    && legacyStatus?.isInvalid
    && legacyStatus.invalidReason
  ) {
    return {
      ...snapshotStatus,
      invalidReason: legacyStatus.invalidReason,
    }
  }

  return snapshotStatus ?? legacyStatus
}

function isOAuthCredentialWithoutRefreshToken(input: ProviderKeyStatusCarrier): boolean {
  return isOAuthManagedCredential(input) && input.oauth_temporary === true
}

function getMissingRefreshTokenStatus(): OAuthStatusInfo {
  return {
    text: t('ui.notAdded'),
    kind: 'missing_refresh_token',
    isExpired: false,
    isExpiringSoon: false,
    isInvalid: false,
  }
}

export function getOAuthStatusDisplay(
  input: ProviderKeyStatusCarrier,
  tick: number,
): OAuthStatusInfo | null {
  return mergeOAuthStatusDisplay(
    getSnapshotOAuthState(input, tick),
    getLegacyOAuthState(input, tick),
  )
}

export function getOAuthStatusDisplayWithFallback(
  input: ProviderKeyStatusCarrier,
  tick: number,
): OAuthStatusInfo | null {
  if (isOAuthCredentialWithoutRefreshToken(input)) {
    return getMissingRefreshTokenStatus()
  }

  const status = getOAuthStatusDisplay(input, tick)
  if (status) return status
  if (!isOAuthManagedCredential(input)) return null

  return {
    text: t('ui.unknownExpiration'),
    kind: 'unknown_expiration',
    isExpired: false,
    isExpiringSoon: false,
    isInvalid: false,
  }
}

export function getOAuthStatusTitle(
  input: ProviderKeyStatusCarrier,
  tick: number,
): string {
  if (isOAuthCredentialWithoutRefreshToken(input)) {
    return t('providers.migration.refreshTokenMissingAutomaticRefresh5c3beb')
  }

  const status = getOAuthStatusDisplay(input, tick)
  if (!status) {
    return isOAuthManagedCredential(input) ? t('providers.migration.tokenExpiryUnknown7c72f1') : ''
  }
  if (status.isInvalid) {
    const reason = normalizeText(status.invalidReason)
    return reason ? t('providers.migration.tokenInvalidaa777d', { value: String(reason) }) : t('providers.migration.tokenInvalid1b1494')
  }
  const snapshotCode = normalizeText(input.status_snapshot?.oauth?.code)
  if (snapshotCode === 'reauth_required' || status.requiresReauth) {
    const reason = normalizeText(status.invalidReason)
    return reason
      ? t('providers.migration.refreshTokenRenewalFailedThe63e8b3', { value: String(reason) })
      : t('providers.migration.refreshTokenRenewalFailedThee083e9')
  }
  if (status.isExpired) {
    return t('providers.migration.accessTokenExpiredWaitingFora1af2b')
  }
  return t('providers.migration.tokenExpiresIn2e6f8c', { value: String(status.text) })
}

export function getOAuthRefreshButtonTitle(
  input: ProviderKeyStatusCarrier,
  tick: number,
): string {
  if (isOAuthManagedCredential(input) && !canRefreshOAuthCredential(input)) {
    if (input.oauth_temporary === true) {
      return t('providers.migration.importedWithAnAccessTokenf94185')
    }
    return t('providers.migration.theseOauthCredentialsCannotRefresh9c2742')
  }

  const status = getOAuthStatusDisplay(input, tick)
  if (status?.isInvalid || status?.isExpired || status?.requiresReauth) {
    return t('providers.migration.reauthorize203ccc')
  }
  return t('providers.migration.refreshToken3e9059')
}

export function getAccountStatusTitle(input: ProviderKeyStatusCarrier): string {
  const account = getAccountStatusDisplay(input)
  if (!account.blocked || !account.label) return ''
  return account.reason ? `${account.label}: ${account.reason}` : account.label
}
