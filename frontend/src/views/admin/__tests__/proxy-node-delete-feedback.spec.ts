import { describe, expect, it } from 'vitest'

import { proxyNodeDeleteSuccessMessage } from '../proxy-node-delete-feedback'

describe('proxyNodeDeleteSuccessMessage', () => {
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
