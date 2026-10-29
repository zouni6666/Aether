import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { getI18nLocale, setI18nLocale, type Locale } from '@/i18n'
import {
  USER_BATCH_ACTION_OPTIONS,
  USER_ROLE_FILTER_OPTIONS,
  USER_SORT_OPTIONS,
  USER_STATUS_FILTER_OPTIONS,
} from '../user-management-config'

let previousLocale: Locale

beforeEach(() => {
  previousLocale = getI18nLocale()
})

afterEach(() => setI18nLocale(previousLocale))

describe('user management option translations', () => {
  it('updates the existing option arrays after switching language', () => {
    setI18nLocale('zh-CN')
    expect(USER_ROLE_FILTER_OPTIONS[0].label).toBe('全部角色')
    expect(USER_STATUS_FILTER_OPTIONS[0].label).toBe('全部状态')
    expect(USER_SORT_OPTIONS[0].label).toBe('默认排序')
    expect(USER_BATCH_ACTION_OPTIONS[0].label).toBe('启用')
    expect(USER_BATCH_ACTION_OPTIONS[0].description).toBe('恢复用户登录与调用')

    setI18nLocale('en-US')
    expect(USER_ROLE_FILTER_OPTIONS[0].label).toBe('All roles')
    expect(USER_STATUS_FILTER_OPTIONS[0].label).toBe('All statuses')
    expect(USER_SORT_OPTIONS[0].label).toBe('Default sort')
    expect(USER_BATCH_ACTION_OPTIONS[0].label).toBe('Enable')
    expect(USER_BATCH_ACTION_OPTIONS[0].description).toBe('Restore user sign-in and API access')
  })
})
