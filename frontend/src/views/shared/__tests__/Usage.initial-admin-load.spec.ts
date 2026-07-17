import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const source = readFileSync(
  resolve(process.cwd(), 'src/views/shared/Usage.vue'),
  'utf8',
)

describe('admin usage initial loading', () => {
  it('starts the user filter request before analytics completes', () => {
    const mountedBlock = source
      .split('onMounted(async () => {')[1]
      ?.split('// 处理时间范围变化')[0]

    expect(mountedBlock).toBeTruthy()
    expect(mountedBlock).toContain('const adminUsersPromise = loadAdminUsers()')
    expect(mountedBlock).toContain('Promise.all([heatmapPromise, adminUsersPromise])')
    expect(mountedBlock?.indexOf('const adminUsersPromise = loadAdminUsers()'))
      .toBeLessThan(mountedBlock?.indexOf('await loadRecords(') ?? -1)
    expect(mountedBlock).not.toContain('await loadAdminUsers()')
  })
})
