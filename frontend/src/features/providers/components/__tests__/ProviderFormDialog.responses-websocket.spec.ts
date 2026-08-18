import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

function readSource(path: string): string {
  return readFileSync(resolve(process.cwd(), path), 'utf8')
}

describe('ProviderFormDialog Responses WebSocket switch', () => {
  it('displays and submits the switch for every provider type', () => {
    const source = readSource('src/features/providers/components/ProviderFormDialog.vue')

    expect(source).toContain('Responses WebSocket 模式')
    expect(source).toContain('responses_websocket_enabled')
    expect(source).toContain('responses_websocket_enabled: form.value.responses_websocket_enabled')
    expect(source).toMatch(
      /<div(?=[^>]*data-testid="responses-websocket-setting")(?![^>]*\bv-if=)[^>]*>[\s\S]{0,500}Responses WebSocket 模式/,
    )
    expect(source).toContain('id="responses-websocket-enabled"')
    expect(source).toContain(':aria-label="legacyT(\'Responses WebSocket 模式\')"')
  })
})
