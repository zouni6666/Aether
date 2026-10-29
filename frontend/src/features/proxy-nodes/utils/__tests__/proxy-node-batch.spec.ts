import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setI18nLocale } from '@/i18n'

import { parseBatchProxyNodeInput } from '../proxy-node-batch'

beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => setI18nLocale('zh-CN'))

describe('parseBatchProxyNodeInput', () => {
  it('parses newline and comma separated proxy URLs into manual node payloads', () => {
    const result = parseBatchProxyNodeInput([
      'socks5://alice:secret@1.2.3.4:1080',
      'http://bob:pwd@5.6.7.8:8080, https://carol:p%40ss@example.com:8443',
    ].join('\n'))

    expect(result.errors).toEqual([])
    expect(result.nodes).toEqual([
      {
        name: '1.2.3.4:1080',
        proxy_url: 'socks5://1.2.3.4:1080',
        username: 'alice',
        password: 'secret',
      },
      {
        name: '5.6.7.8:8080',
        proxy_url: 'http://5.6.7.8:8080',
        username: 'bob',
        password: 'pwd',
      },
      {
        name: 'example.com:8443',
        proxy_url: 'https://example.com:8443',
        username: 'carol',
        password: 'p@ss',
      },
    ])
  })

  it('reports unsupported and invalid entries without dropping valid entries', () => {
    const result = parseBatchProxyNodeInput('socks5://user:pass@127.0.0.1:1080, ftp://user:pass@127.0.0.1:21, nope')

    expect(result.nodes).toEqual([
      {
        name: '127.0.0.1:1080',
        proxy_url: 'socks5://127.0.0.1:1080',
        username: 'user',
        password: 'pass',
      },
    ])
    expect(result.errors).toHaveLength(2)
    expect(result.errors[0]).toContain('仅支持 http/https/socks5/socks5h 协议')
    expect(result.errors[1]).toContain('必须是合法的代理 URL')
  })

  it('translates validation messages while preserving entered values and credentials', () => {
    setI18nLocale('en-US')
    const result = parseBatchProxyNodeInput('保存, http://%E4%BF%9D%E5%AD%98:%E5%88%A0%E9%99%A4@example.com:8080')
    expect(result.errors).toEqual(['Entry 1 保存: Must be a valid proxy URL'])
    expect(result.nodes[0]).toMatchObject({ username: '保存', password: '删除' })
    expect(parseBatchProxyNodeInput('http://bad%ZZ:pass@example.com').errors[0])
      .toContain('Username contains invalid URL encoding')
  })
})
