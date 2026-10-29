import type { OAuthOrganizationInfo } from '@/api/endpoints/types/provider'

type OAuthIdentityDisplayValue = {
  oauth_account_id?: string | null
  oauth_account_name?: string | null
  oauth_account_user_id?: string | null
  oauth_organizations?: OAuthOrganizationInfo[] | null
} | null | undefined

type OAuthOrgBadge = {
  id: string
  label: string
  title: string
}

type SelectedOAuthOrganization = {
  id: string
  title: string
  rawId: unknown
  rawTitle: unknown
  rawIsDefault: boolean | null | undefined
  index: number
}

type OAuthOrgBadgeCacheEntry = {
  rawAccountId: unknown
  rawAccountName: unknown
  rawAccountUserId: unknown
  rawOrganizations: OAuthOrganizationInfo[] | null
  organizationCount: number
  selectedOrganizationIndex: number
  selectedOrganizationRawId: unknown
  selectedOrganizationRawTitle: unknown
  selectedOrganizationRawIsDefault: boolean | null | undefined
  result: OAuthOrgBadge | null
}

const oauthOrgBadgeCache = new WeakMap<object, OAuthOrgBadgeCacheEntry>()

function readStr(raw: unknown): string {
  return typeof raw === 'string' ? raw.trim() : ''
}

function formatOAuthIdentityShort(value: string, head = 8, tail = 6): string {
  if (!value) return ''
  if (value.length <= head + tail + 3) return value
  return `${value.slice(0, head)}...${value.slice(-tail)}`
}

function stripOAuthOrganizationPrefix(orgId: string): string {
  if (orgId.length < 4) return orgId

  const first = orgId.charCodeAt(0) | 32
  const second = orgId.charCodeAt(1) | 32
  const third = orgId.charCodeAt(2) | 32

  if (first !== 111 || second !== 114 || third !== 103) {
    return orgId
  }

  let index = 3
  while (index < orgId.length) {
    const code = orgId.charCodeAt(index)
    if (code !== 45 && code !== 95 && code !== 58) break
    index += 1
  }

  return index === 3 ? orgId : orgId.slice(index)
}

function formatOAuthOrganizationBadge(orgId: string): string {
  const compactOrgId = stripOAuthOrganizationPrefix(orgId)
  const normalized = compactOrgId || orgId
  if (normalized.length <= 10) {
    return `org:${normalized}`
  }
  return `org:${normalized.slice(0, 6)}...${normalized.slice(-4)}`
}

function compactOAuthAccountId(accountId: string): string {
  const normalized = accountId.trim()
  if (!normalized) return ''

  const withoutPrefix = normalized.replace(/^[^:_-]+[-_:]+/, '')
  const compact = withoutPrefix.replace(/[-_:]+/g, '')

  return compact || withoutPrefix || normalized
}

function formatOAuthAccountBadge(accountId: string): string {
  return compactOAuthAccountId(accountId).slice(0, 5)
}

function formatOAuthAccountUserBadge(accountUserId: string): string {
  return formatOAuthIdentityShort(accountUserId, 8, 6)
}

function getPrimaryOAuthOrganization(
  value: OAuthIdentityDisplayValue,
): SelectedOAuthOrganization | null {
  const organizations = Array.isArray(value?.oauth_organizations)
    ? value.oauth_organizations
    : null
  if (!organizations?.length) return null

  let firstWithId: SelectedOAuthOrganization | null = null

  for (let index = 0; index < organizations.length; index += 1) {
    const org = organizations[index]
    const rawId = org?.id
    if (typeof rawId !== 'string') continue

    const id = rawId.trim()
    if (!id) continue

    const selectedOrganization: SelectedOAuthOrganization = {
      id,
      title: readStr(org?.title),
      rawId,
      rawTitle: org?.title,
      rawIsDefault: org?.is_default,
      index,
    }

    if (!firstWithId) firstWithId = selectedOrganization
    if (org.is_default) {
      firstWithId = selectedOrganization
      break
    }
  }

  return firstWithId
}

function appendTitlePart(title: string, label: string, value: string): string {
  if (!value) return title
  if (!title) return `${label}: ${value}`
  return `${title} | ${label}: ${value}`
}

function buildOAuthIdentityTitle(
  accountName: string,
  accountId: string,
  accountUserId: string,
  organization: SelectedOAuthOrganization | null,
): string {
  let title = ''

  title = appendTitlePart(title, 'name', accountName)
  title = appendTitlePart(title, 'account_id', accountId)
  title = appendTitlePart(title, 'account_user_id', accountUserId)
  title = appendTitlePart(title, 'org_id', organization?.id || '')
  title = appendTitlePart(title, 'org_title', organization?.title || '')

  return title
}

function getCachedOAuthOrgBadge(
  value: OAuthIdentityDisplayValue,
  organization: SelectedOAuthOrganization | null,
): OAuthOrgBadge | null | undefined {
  if (!value || typeof value !== 'object') return undefined

  const cached = oauthOrgBadgeCache.get(value)
  if (!cached) return undefined

  const organizations = Array.isArray(value.oauth_organizations)
    ? value.oauth_organizations
    : null

  if (
    cached.rawAccountId === value.oauth_account_id
    && cached.rawAccountName === value.oauth_account_name
    && cached.rawAccountUserId === value.oauth_account_user_id
    && cached.rawOrganizations === organizations
    && cached.organizationCount === (organizations?.length || 0)
    && cached.selectedOrganizationIndex === (organization?.index ?? -1)
    && cached.selectedOrganizationRawId === organization?.rawId
    && cached.selectedOrganizationRawTitle === organization?.rawTitle
    && cached.selectedOrganizationRawIsDefault === organization?.rawIsDefault
  ) {
    return cached.result
  }

  return undefined
}

function setCachedOAuthOrgBadge(
  value: OAuthIdentityDisplayValue,
  organization: SelectedOAuthOrganization | null,
  result: OAuthOrgBadge | null,
): void {
  if (!value || typeof value !== 'object') return

  const organizations = Array.isArray(value.oauth_organizations)
    ? value.oauth_organizations
    : null

  oauthOrgBadgeCache.set(value, {
    rawAccountId: value.oauth_account_id,
    rawAccountName: value.oauth_account_name,
    rawAccountUserId: value.oauth_account_user_id,
    rawOrganizations: organizations,
    organizationCount: organizations?.length || 0,
    selectedOrganizationIndex: organization?.index ?? -1,
    selectedOrganizationRawId: organization?.rawId,
    selectedOrganizationRawTitle: organization?.rawTitle,
    selectedOrganizationRawIsDefault: organization?.rawIsDefault,
    result,
  })
}

export function getOAuthOrgBadge(
  value: OAuthIdentityDisplayValue,
): OAuthOrgBadge | null {
  const org = getPrimaryOAuthOrganization(value)
  const cached = getCachedOAuthOrgBadge(value, org)
  if (cached !== undefined) return cached

  const accountId = readStr(value?.oauth_account_id)
  const accountName = readStr(value?.oauth_account_name)
  const accountUserId = readStr(value?.oauth_account_user_id)

  const badgeId = accountId || accountUserId || org?.id || ''
  if (!badgeId) {
    setCachedOAuthOrgBadge(value, org, null)
    return null
  }

  const label = accountId
    ? formatOAuthAccountBadge(accountId)
    : accountUserId
      ? formatOAuthAccountUserBadge(accountUserId)
      : org
        ? formatOAuthOrganizationBadge(org.id)
        : ''
  if (!label) {
    setCachedOAuthOrgBadge(value, org, null)
    return null
  }

  const result: OAuthOrgBadge = {
    id: badgeId,
    label,
    title: buildOAuthIdentityTitle(accountName, accountId, accountUserId, org),
  }

  setCachedOAuthOrgBadge(value, org, result)
  return result
}
