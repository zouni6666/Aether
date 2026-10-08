import { describe, expect, it, vi } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { adminRoutes } from '../admin'

vi.mock('../helpers', () => ({ view: () => ({ render: () => null }) }))

function createAdminRouter() {
  return createRouter({ history: createMemoryHistory(), routes: adminRoutes })
}

describe('provider scheduling routes', () => {
  it.each([
    ['/admin/routing', undefined],
    ['/admin/routing/new', 'new'],
    ['/admin/routing/strategy-a', 'strategy-a'],
  ])('redirects %s into the provider group directory', async (path, group) => {
    const router = createAdminRouter()

    await router.push(`${path}?view=overview&group=wrong&model=gpt-5#rules`)

    expect(router.currentRoute.value.name).toBe('ProviderManagement')
    expect(router.currentRoute.value.path).toBe('/admin/providers')
    expect(router.currentRoute.value.query).toEqual({ model: 'gpt-5', ...(group ? { group } : {}) })
    expect(router.currentRoute.value.hash).toBe('#rules')
    expect(router.currentRoute.value.meta).toMatchObject({ requiresAuth: true, requiresAdmin: true })
  })

  it.each([
    ['RoutingProfiles', undefined],
    ['RoutingProfileCreate', 'new'],
    ['RoutingProfileDetail', 'strategy-a'],
  ])('preserves navigation by the legacy %s route name', async (name, group) => {
    const router = createAdminRouter()

    await router.push({ name, ...(name === 'RoutingProfileDetail' ? { params: { groupId: group } } : {}) })

    expect(router.currentRoute.value.name).toBe('ProviderManagement')
    expect(router.currentRoute.value.query).toEqual({ ...(group ? { group } : {}) })
  })

  it('keeps the provider overview directly accessible', async () => {
    const router = createAdminRouter()

    await router.push('/admin/providers')

    expect(router.currentRoute.value.name).toBe('ProviderManagement')
    expect(router.currentRoute.value.query).toEqual({})
    expect(router.currentRoute.value.redirectedFrom).toBeUndefined()
  })
})
