import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setI18nLocale } from '@/i18n'

import { proxyNodeDeleteSuccessMessage } from '../proxy-node-delete-feedback'

beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => setI18nLocale('zh-CN'))

describe('proxyNodeDeleteSuccessMessage', () => {
  it('uses the current locale for coordinated delete feedback', () => {
    setI18nLocale('en-US')
    expect(proxyNodeDeleteSuccessMessage(true, true))
      .toBe('Proxy node deleted. The system default proxy and external model catalog proxy were cleared automatically.')
    expect(proxyNodeDeleteSuccessMessage(false, false)).toBe('Proxy node deleted')
  })

  it.each([
    {
      clearedSystemProxy: true,
      clearedExternalModelsProxy: true,
      expected: '代理节点已删除，系统默认代理和外部模型目录代理已自动清除',
    },
    {
      clearedSystemProxy: true,
      clearedExternalModelsProxy: false,
      expected: '代理节点已删除，系统默认代理已自动清除',
    },
    {
      clearedSystemProxy: false,
      clearedExternalModelsProxy: true,
      expected: '代理节点已删除，外部模型目录代理已自动清除',
    },
    {
      clearedSystemProxy: false,
      clearedExternalModelsProxy: false,
      expected: '代理节点已删除',
    },
  ])(
    'returns the coordinated message for system=$clearedSystemProxy and external=$clearedExternalModelsProxy',
    ({ clearedSystemProxy, clearedExternalModelsProxy, expected }) => {
      expect(proxyNodeDeleteSuccessMessage(
        clearedSystemProxy,
        clearedExternalModelsProxy,
      )).toBe(expected)
    },
  )
})
