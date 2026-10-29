import { translate } from '@/i18n'
export interface BatchManualProxyNode {
  name: string
  proxy_url: string
  username?: string
  password?: string
}

export interface BatchProxyNodeParseResult {
  nodes: BatchManualProxyNode[]
  errors: string[]
}

const SUPPORTED_PROXY_PROTOCOLS = new Set(['http:', 'https:', 'socks5:', 'socks5h:'])

export function parseBatchProxyNodeInput(input: string): BatchProxyNodeParseResult {
  const entries = input
    .split(/[\n,]+/)
    .map(entry => entry.trim())
    .filter(Boolean)

  const nodes: BatchManualProxyNode[] = []
  const errors: string[] = []

  entries.forEach((entry, index) => {
    try {
      nodes.push(parseBatchProxyNodeEntry(entry))
    } catch (err) {
      const message = err instanceof Error ? err.message : translate('ui.invalidFormat')
      errors.push(translate('feedback.proxy.entryError', { index: index + 1, entry, message }))
    }
  })

  return { nodes, errors }
}

function parseBatchProxyNodeEntry(entry: string): BatchManualProxyNode {
  let url: URL
  try {
    url = new URL(entry)
  } catch {
    throw new Error(translate('ui.mustBeAValidProxyUrl'))
  }

  if (!SUPPORTED_PROXY_PROTOCOLS.has(url.protocol)) {
    throw new Error(translate('ui.onlyHttpHttpsSocks5Socks5hProtocolsAreSupported'))
  }

  const host = url.hostname.trim()
  if (!host) {
    throw new Error(translate('ui.missingHostAddress'))
  }

  const username = decodeUrlCredential(url.username, translate('auth.register.username'))
  const password = decodeUrlCredential(url.password, translate('auth.register.password'))
  if (!username && password) {
    throw new Error(translate('ui.aUsernameIsRequiredWhenAPasswordIsProvided'))
  }

  const port = url.port || defaultProxyPort(url.protocol)
  const proxyHost = url.host || `${host}:${port}`
  const proxyUrl = `${url.protocol}//${proxyHost}`

  return {
    name: `${formatNodeNameHost(host)}:${port}`,
    proxy_url: proxyUrl,
    username: username || undefined,
    password: password || undefined,
  }
}

function defaultProxyPort(protocol: string): string {
  if (protocol === 'https:') return '443'
  if (protocol === 'socks5:' || protocol === 'socks5h:') return '1080'
  return '80'
}

function decodeUrlCredential(value: string, field: string): string {
  if (!value) return ''
  try {
    return decodeURIComponent(value)
  } catch {
    throw new Error(translate('feedback.proxy.invalidCredential', { field }))
  }
}

function formatNodeNameHost(host: string): string {
  if (host.includes(':') && !host.startsWith('[')) {
    return `[${host}]`
  }
  return host
}
