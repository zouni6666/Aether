import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const source = readFileSync(
  resolve(process.cwd(), 'src/views/admin/Users.vue'),
  'utf8',
)

describe('Users request loading', () => {
  it('debounces search without delaying discrete filters', () => {
    const searchWatcher = source
      .split('watch(searchQuery, () => {')[1]
      ?.split('watch([filterRole, filterStatus, filterGroup, sortOption]')[0]
    expect(searchWatcher).toBeTruthy()
    expect(searchWatcher).toContain('setTimeout(')
    expect(searchWatcher).toContain('USERS_SEARCH_DEBOUNCE_MS')

    const discreteFilterWatcher = source
      .split('watch([filterRole, filterStatus, filterGroup, sortOption], () => {')[1]
      ?.split('watch(paginatedUsers')[0]
    expect(discreteFilterWatcher).toBeTruthy()
    expect(discreteFilterWatcher).not.toContain('setTimeout(')
    expect(discreteFilterWatcher).toContain('refreshUsers()')
  })

  it('does not reload invariant metadata for every list refresh', () => {
    const listRefresh = source
      .split('async function refreshUsers(')[1]
      ?.split('async function handleManualRefresh()')[0]
    expect(listRefresh).toBeTruthy()
    expect(listRefresh).not.toContain('loadUserGroups()')
    expect(listRefresh).not.toContain('loadUserWallets(')

    const manualRefresh = source
      .split('async function handleManualRefresh()')[1]
      ?.split('function handleTableSort')[0]
    expect(manualRefresh).toBeTruthy()
    expect(manualRefresh).toContain('refreshUsers()')
    expect(manualRefresh).toContain('loadUserGroups()')
    expect(manualRefresh).toContain('loadUserWallets()')
    expect(source).toContain('@refresh="handleManualRefresh"')
  })

  it('refreshes wallet state after user access-control mutations', () => {
    const batchCompleted = source
      .split('async function handleUserBatchCompleted')[1]
      ?.split('function invalidateUserOptions')[0]
    expect(batchCompleted).toContain('Promise.all([refreshUsers(), loadUserWallets()])')

    const formSubmit = source
      .split('async function handleUserFormSubmit')[1]
      ?.split('async function manageApiKeys')[0]
    expect(formSubmit).toContain('Promise.all([refreshUsers(), loadUserWallets()])')
  })
})
