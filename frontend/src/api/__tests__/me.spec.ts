import { beforeEach, describe, expect, it, vi } from 'vitest'

const { patchMock } = vi.hoisted(() => ({
  patchMock: vi.fn(),
}))

vi.mock('@/api/client', () => ({
  default: {
    patch: patchMock,
  },
}))

import { meApi } from '@/api/me'

describe('meApi API key status', () => {
  beforeEach(() => {
    patchMock.mockReset()
    patchMock.mockResolvedValue({
      data: {
        id: 'user-key-1',
        is_active: false,
      },
    })
  })

  it('sends the desired disabled state in the patch body', async () => {
    await meApi.toggleApiKey('user-key-1', false)

    expect(patchMock).toHaveBeenCalledWith(
      '/api/users/me/api-keys/user-key-1',
      { is_active: false },
    )
  })
})