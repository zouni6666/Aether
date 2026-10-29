import { readFileSync, readdirSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { NodeTypes, parse as parseTemplate, type TemplateChildNode } from '@vue/compiler-dom'
import { parse } from 'vue/compiler-sfc'

const root = resolve(import.meta.dirname, '../../..')
const displayAttributes = new Set([
  'alt', 'aria-label', 'cancel-text', 'client-label', 'confirm-text', 'description',
  'drop-title', 'empty-message', 'empty-text', 'entity-label', 'filter-title', 'label',
  'manual-placeholder', 'message', 'path-hint', 'placeholder', 'provider-label',
  'search-placeholder', 'subtitle', 'title',
])

function sourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const file = resolve(directory, entry.name)
    if (entry.isDirectory()) return entry.name === '__tests__' ? [] : sourceFiles(file)
    return /\.(ts|vue)$/.test(entry.name) && !/\.(test|spec)\.ts$/.test(entry.name) ? [file] : []
  })
}

function untranslatedTemplateCopy(template: string): string[] {
  const remaining: string[] = []
  const visit = (node: TemplateChildNode): void => {
    if (node.type === NodeTypes.ELEMENT) {
      if (['script', 'style', 'pre', 'code', 'kbd', 'samp'].includes(node.tag.toLowerCase())) return
      if (node.props.some(prop => prop.type === NodeTypes.ATTRIBUTE && (
        ['data-i18n-skip', 'contenteditable'].includes(prop.name) || prop.name === 'translate' && prop.value?.content === 'no'
      ))) return
      if (/\sv-pre(?:\s|>|=)/u.test(node.loc.source.split('>')[0])) return
      for (const prop of node.props) {
        if (prop.type !== NodeTypes.ATTRIBUTE || !prop.value) continue
        const displayAttribute = displayAttributes.has(prop.name)
          || prop.name === 'text' && /help-?hint/i.test(node.tag)
        if (displayAttribute && /[\u4e00-\u9fff]/u.test(prop.value.content)) {
          remaining.push(`@${prop.name}: ${prop.value.content}`)
        }
      }
      // Textarea contents are editable data; its placeholder remains display copy.
      if (node.tag.toLowerCase() !== 'textarea') node.children.forEach(visit)
    } else if (node.type === NodeTypes.TEXT && /[\u4e00-\u9fff]/u.test(node.content)) {
      remaining.push(node.content.trim())
    }
  }
  parseTemplate(template).children.forEach(visit)
  return remaining
}

describe('explicit message migration', () => {
  it('keeps application code and build configuration free of automatic text translators', () => {
    const files = [...sourceFiles(resolve(root, 'src')), resolve(root, 'vite.config.ts'), resolve(root, 'vitest.config.ts')]
    for (const file of files) {
      expect(readFileSync(file, 'utf8'), file).not.toMatch(/\b(?:legacyT|translateLegacyText|legacyTemplateI18nPlugin|installLegacyDomTranslator)\b/u)
    }
  })

  it('uses explicit translations for static template copy while preserving code and editable content', () => {
    const remaining: string[] = []
    for (const file of sourceFiles(resolve(root, 'src')).filter(file => file.endsWith('.vue'))) {
      const { descriptor } = parse(readFileSync(file, 'utf8'))
      if (!descriptor.template) continue
      remaining.push(...untranslatedTemplateCopy(descriptor.template.content).map(copy => `${file}: ${copy}`))
    }
    expect(remaining).toEqual([])
  })

  it('checks display attributes without interpreting code or editable data as copy', () => {
    expect(untranslatedTemplateCopy(`
      <button title="保存" aria-label="删除">确认</button>
      <input placeholder="请输入" value="用户原文" />
      <Textarea placeholder="编辑提示">用户原文</Textarea>
      <HelpHint text="帮助提示" />
      <div :title="server.title" data-name="用户原文" />
      <pre title="示例标题">代码原文</pre>
      <div v-pre title="模板原文">{{ 原始内容 }}</div>
      <div contenteditable title="编辑原文">用户原文</div>
      <div data-i18n-skip title="跳过内容">原文</div>
      <div translate="no" title="固定内容">原文</div>
    `)).toEqual([
      '@title: 保存', '@aria-label: 删除', '确认', '@placeholder: 请输入',
      '@placeholder: 编辑提示', '@text: 帮助提示',
    ])
  })
})
