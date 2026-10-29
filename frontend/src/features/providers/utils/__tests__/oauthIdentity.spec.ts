import { describe, expect, it } from 'vitest'

import { getOAuthOrgBadge } from '../oauthIdentity'

describe('getOAuthOrgBadge', () => {
  it('prefers compact account id over organization badge when available', () => {
    const badge = getOAuthOrgBadge({
      oauth_account_id: 'acct-demo-001',
      oauth_account_name: 'Workspace Alpha',
      oauth_account_user_id: 'user-1__acct-demo-001',
      oauth_organizations: [
        { id: 'org-personal-1234', title: 'Personal', is_default: true },
      ],
    })

    expect(badge).toEqual({
      id: 'acct-demo-001',
      label: 'demo0',
      title: 'name: Workspace Alpha | account_id: acct-demo-001 | account_user_id: user-1__acct-demo-001 | org_id: org-personal-1234 | org_title: Personal',
    })
  })

  it('falls back to short account id when no organization is available', () => {
    const badge = getOAuthOrgBadge({
      oauth_account_id: 'acct-demo-001',
      oauth_account_name: 'Workspace Alpha',
    })

    expect(badge).toEqual({
      id: 'acct-demo-001',
      label: 'demo0',
      title: 'name: Workspace Alpha | account_id: acct-demo-001',
    })
  })

  it('drops the leading prefix before taking the 5-character account badge', () => {
    const badge = getOAuthOrgBadge({
      oauth_account_id: 'org-HUone5DwhkDWrgXpDR3JJbGi',
    })

    expect(badge).toEqual({
      id: 'org-HUone5DwhkDWrgXpDR3JJbGi',
      label: 'HUone',
      title: 'account_id: org-HUone5DwhkDWrgXpDR3JJbGi',
    })
  })

  it('does not create an identity badge from account name alone', () => {
    const badge = getOAuthOrgBadge({
      oauth_account_name: 'Free',
    })

    expect(badge).toBeNull()
  })

  it('reuses cached badge objects when identity fields are unchanged', () => {
    const identity = {
      oauth_account_id: 'acct-demo-001',
      oauth_account_name: 'Workspace Alpha',
      oauth_account_user_id: 'user-1__acct-demo-001',
      oauth_organizations: [
        { id: 'org-personal-1234', title: 'Personal', is_default: true },
      ],
    }

    const first = getOAuthOrgBadge(identity)
    const second = getOAuthOrgBadge(identity)

    expect(first).toBe(second)
  })

  it('invalidates the cached badge when the selected organization changes in place', () => {
    const organizations = [
      { id: 'org-personal-1234', title: 'Personal', is_default: true },
      { id: 'org-team-5678', title: 'Team', is_default: false },
    ]
    const identity = {
      oauth_account_id: 'acct-demo-001',
      oauth_organizations: organizations,
    }

    const first = getOAuthOrgBadge(identity)
    organizations[0].is_default = false
    organizations[1].is_default = true
    const second = getOAuthOrgBadge(identity)

    expect(first).not.toBe(second)
    expect(second).toEqual({
      id: 'acct-demo-001',
      label: 'demo0',
      title: 'account_id: acct-demo-001 | org_id: org-team-5678 | org_title: Team',
    })
  })
})
