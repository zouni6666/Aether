import { translate } from '@/i18n'
/**
 * Claude API 格式解析器
 */

import type {
  ApiFormatParser,
  ParsedConversation,
  ParsedMessage,
  ContentBlock,
  MessageRole,
} from './types'
import {
  createEmptyConversation,
  createMessage,
  createTextBlock,
  createThinkingBlock,
  createToolUseBlock,
  createToolResultBlock,
  createImageBlock,
  isStreamResponse,
} from './types'
import type { RenderResult, RenderBlock, BadgeRenderBlock } from './render'
import {
  createTextBlock as createTextRenderBlock,
  createCollapsibleBlock,
  createCodeBlock,
  createBadgeBlock,
  createImageBlock as createImageRenderBlock,
  createErrorBlock as createErrorRenderBlock,
  createMessageBlock,
  createToolUseBlock as createToolUseRenderBlock,
  createToolResultBlock as createToolResultRenderBlock,
  createEmptyRenderResult,
} from './render'

/** Raw JSON object from API (loosely typed) */
type RawObject = Record<string, unknown>

/**
 * Claude API 格式解析器
 */
export class ClaudeParser implements ApiFormatParser {
  readonly format = 'claude' as const
  readonly displayName = 'Claude'

  /**
   * 检测是否为 Claude 格式
   */
  detect(requestBody: unknown, responseBody: unknown, hint?: string): number {
    // 1. 后端提示优先
    if (hint) {
      const lowerHint = hint.toLowerCase()
      if (lowerHint.includes('claude')) return 100
      // 如果明确是其他格式，返回 0
      if (lowerHint.includes('openai') || lowerHint.includes('gemini')) return 0
    }

    const req = requestBody as RawObject | null | undefined

    // 2. 检查模型名
    const model = (typeof req?.model === 'string' ? req.model : '').toLowerCase()
    if (model.includes('claude')) return 95

    // 3. 检查请求体结构
    if (!req?.messages || !Array.isArray(req.messages)) {
      return 0
    }

    // 4. 检查响应体特征
    const respBody = (isStreamResponse(responseBody)
      ? (responseBody.chunks?.[0] as RawObject | undefined)
      : responseBody) as RawObject | null | undefined

    if (respBody) {
      // Claude 响应特征
      const respType = typeof respBody.type === 'string' ? respBody.type : ''
      if (
        respType === 'message' ||
        respType.startsWith('content_block') ||
        respType.startsWith('message_')
      ) {
        return 90
      }
      // 明确是 OpenAI 格式
      const respObject = typeof respBody.object === 'string' ? respBody.object : ''
      if (respBody.choices || respObject.includes('chat.completion')) {
        return 0
      }
    }

    // 5. 检查 Claude 特有的请求字段
    if (req.system !== undefined) {
      // system 可以是字符串或数组，这是 Claude 的特征
      return 70
    }

    // 默认返回中等置信度（Aether 主要用于 Claude）
    return 50
  }

  /**
   * 解析请求体
   */
  parseRequest(requestBody: unknown): ParsedConversation {
    if (!requestBody) {
      return createEmptyConversation('claude', '无请求体')
    }

    try {
      const body = requestBody as RawObject
      const result: ParsedConversation = {
        messages: [],
        isStream: body.stream === true,
        apiFormat: 'claude',
        model: typeof body.model === 'string' ? body.model : undefined,
      }

      // 提取 system prompt
      result.system = this.extractSystemPrompt(body.system)

      // 提取 messages
      if (Array.isArray(body.messages)) {
        for (const msg of body.messages) {
          const parsedMsg = this.parseMessage(msg as RawObject)
          if (parsedMsg) {
            result.messages.push(parsedMsg)
          }
        }
      }

      return result
    } catch (e) {
      return createEmptyConversation('claude', `解析失败: ${e}`)
    }
  }

  /**
   * 解析响应体
   */
  parseResponse(responseBody: unknown): ParsedConversation {
    if (!responseBody) {
      return createEmptyConversation('claude', '无响应体')
    }

    try {
      const body = responseBody as RawObject
      const result: ParsedConversation = {
        messages: [],
        isStream: false,
        apiFormat: 'claude',
        model: typeof body.model === 'string' ? body.model : undefined,
      }

      // Claude 响应格式: { type: "message", content: [...] }
      if (Array.isArray(body.content)) {
        const contentBlocks = this.parseContentBlocks(body.content as RawObject[], 'assistant')
        if (contentBlocks.length > 0) {
          result.messages.push(createMessage('assistant', contentBlocks))
        }
      }

      return result
    } catch (e) {
      return createEmptyConversation('claude', `解析失败: ${e}`)
    }
  }

  /**
   * 解析流式响应
   */
  parseStreamResponse(chunks: unknown[]): ParsedConversation {
    if (!chunks || chunks.length === 0) {
      return createEmptyConversation('claude', '无响应数据')
    }

    try {
      const result: ParsedConversation = {
        messages: [],
        isStream: true,
        apiFormat: 'claude',
      }

      // 按 content block index 分组累积
      const blocks = new Map<number, {
        type: ContentBlock['type']
        parts: string[]
        metadata?: Record<string, string>
      }>()

      for (const rawChunk of chunks) {
        const chunk = rawChunk as RawObject
        // 提取模型名
        const chunkMessage = chunk.message as RawObject | undefined
        if (typeof chunkMessage?.model === 'string' && !result.model) {
          result.model = chunkMessage.model
        }

        if (chunk.type === 'content_block_start') {
          const index = (typeof chunk.index === 'number' ? chunk.index : 0)
          const block = chunk.content_block as RawObject | undefined
          if (block?.type === 'text') {
            blocks.set(index, { type: 'text', parts: [String(block.text || '')] })
          } else if (block?.type === 'thinking') {
            blocks.set(index, {
              type: 'thinking',
              parts: [String(block.thinking || '')],
              metadata: { signature: String(block.signature || '') },
            })
          } else if (block?.type === 'tool_use') {
            blocks.set(index, {
              type: 'tool_use',
              parts: [],
              metadata: { toolName: String(block.name || ''), toolId: String(block.id || '') },
            })
          }
        } else if (chunk.type === 'content_block_delta') {
          const index = (typeof chunk.index === 'number' ? chunk.index : 0)
          const delta = chunk.delta as RawObject | undefined
          const block = blocks.get(index)
          if (block && delta) {
            if (delta.type === 'text_delta') {
              block.parts.push(String(delta.text || ''))
            } else if (delta.type === 'thinking_delta') {
              block.parts.push(String(delta.thinking || ''))
            } else if (delta.type === 'input_json_delta') {
              block.parts.push(String(delta.partial_json || ''))
            } else if (delta.type === 'signature_delta') {
              block.metadata = block.metadata || {}
              block.metadata.signature = (block.metadata.signature || '') + String(delta.signature || '')
            }
          }
        }
      }

      // 转换为消息内容块
      const contentBlocks: ContentBlock[] = []
      const sortedEntries = Array.from(blocks.entries()).sort((a, b) => a[0] - b[0])

      for (const [, block] of sortedEntries) {
        const content = block.parts.join('')

        switch (block.type) {
          case 'text':
            contentBlocks.push(createTextBlock(content))
            break
          case 'thinking':
            contentBlocks.push(createThinkingBlock(content, block.metadata?.signature))
            break
          case 'tool_use':
            contentBlocks.push(createToolUseBlock(
              block.metadata?.toolId || '',
              block.metadata?.toolName || '',
              content || '{}'
            ))
            break
        }
      }

      if (contentBlocks.length > 0) {
        result.messages.push(createMessage('assistant', contentBlocks))
      }

      return result
    } catch (e) {
      return createEmptyConversation('claude', `解析失败: ${e}`)
    }
  }

  /**
   * 提取 system prompt
   */
  private extractSystemPrompt(system: unknown): string | undefined {
    if (!system) return undefined

    if (typeof system === 'string') {
      return system
    }

    if (Array.isArray(system)) {
      return system
        .filter((b: RawObject) => b.type === 'text')
        .map((b: RawObject) => String(b.text || ''))
        .join('\n')
    }

    return undefined
  }

  /**
   * 解析单条消息
   */
  private parseMessage(msg: RawObject): ParsedMessage | null {
    if (!msg || !msg.role) return null

    const role = msg.role as MessageRole
    const contentBlocks = this.parseMessageContent(msg.content, role)

    if (contentBlocks.length === 0) return null

    return createMessage(role, contentBlocks)
  }

  /**
   * 解析消息内容
   */
  private parseMessageContent(content: unknown, role: MessageRole): ContentBlock[] {
    if (typeof content === 'string') {
      return [createTextBlock(content)]
    }

    if (Array.isArray(content)) {
      return this.parseContentBlocks(content as RawObject[], role)
    }

    return []
  }

  /**
   * 解析内容块数组
   */
  private parseContentBlocks(blocks: RawObject[], role: MessageRole): ContentBlock[] {
    const result: ContentBlock[] = []

    for (const block of blocks) {
      const parsed = this.parseContentBlock(block, role)
      if (parsed) {
        result.push(parsed)
      }
    }

    return result
  }

  /**
   * 解析单个内容块
   */
  private parseContentBlock(block: RawObject, _role: MessageRole): ContentBlock | null {
    if (!block || !block.type) return null

    switch (block.type) {
      case 'text':
        return createTextBlock(String(block.text || ''))

      case 'thinking':
        return createThinkingBlock(
          String(block.thinking || ''),
          typeof block.signature === 'string' ? block.signature : undefined
        )

      case 'tool_use':
        return createToolUseBlock(
          String(block.id || ''),
          String(block.name || ''),
          (block.input as Record<string, unknown>) || {}
        )

      case 'tool_result':
        return createToolResultBlock(
          String(block.tool_use_id || ''),
          this.parseToolResultContent(block.content),
          block.is_error as boolean | undefined
        )

      case 'image':
        return this.parseImageBlock(block)

      default:
        return null
    }
  }

  /**
   * 解析图片块
   */
  private parseImageBlock(block: RawObject): ContentBlock | null {
    const source = block.source as RawObject | undefined
    if (!source) {
      return createImageBlock('base64', { alt: '[图片]' })
    }

    if (source.type === 'base64') {
      return createImageBlock('base64', {
        data: typeof source.data === 'string' ? source.data : undefined,
        mimeType: typeof source.media_type === 'string' ? source.media_type : undefined,
      })
    }

    if (source.type === 'url') {
      return createImageBlock('url', {
        url: typeof source.url === 'string' ? source.url : undefined,
        mimeType: typeof source.media_type === 'string' ? source.media_type : undefined,
      })
    }

    return createImageBlock('base64', { alt: '[图片]' })
  }

  /**
   * 解析工具结果内容
   */
  private parseToolResultContent(content: unknown): string | ContentBlock[] {
    if (typeof content === 'string') {
      return content
    }

    if (Array.isArray(content)) {
      const blocks: ContentBlock[] = []
      for (const rawItem of content) {
        const item = rawItem as RawObject
        if (item.type === 'text') {
          blocks.push(createTextBlock(String(item.text || '')))
        } else if (item.type === 'image') {
          const imgBlock = this.parseImageBlock(item)
          if (imgBlock) blocks.push(imgBlock)
        }
      }
      return blocks.length > 0 ? blocks : JSON.stringify(content, null, 2)
    }

    return JSON.stringify(content, null, 2)
  }

  // ============================================================
  // 渲染方法
  // ============================================================

  /**
   * 渲染请求体
   */
  renderRequest(requestBody: unknown): RenderResult {
    if (!requestBody) {
      return createEmptyRenderResult('无请求体')
    }

    try {
      const body = requestBody as RawObject
      const blocks: RenderBlock[] = []
      const isStream = body.stream === true

      // 渲染 system prompt
      const system = this.extractSystemPrompt(body.system)
      if (system) {
        blocks.push(createMessageBlock('system', [
          createTextRenderBlock(system),
        ], { roleLabel: 'System' }))
      }

      // 渲染 messages
      if (Array.isArray(body.messages)) {
        for (const msg of body.messages) {
          const msgBlock = this.renderMessage(msg as RawObject)
          if (msgBlock) {
            blocks.push(msgBlock)
          }
        }
      }

      return { blocks, isStream }
    } catch (e) {
      return createEmptyRenderResult(`渲染失败: ${e}`)
    }
  }

  /**
   * 渲染响应体
   */
  renderResponse(responseBody: unknown): RenderResult {
    if (!responseBody) {
      return createEmptyRenderResult('无响应体')
    }

    // 检查是否为流式响应
    if (isStreamResponse(responseBody)) {
      return this.renderStreamResponse(responseBody.chunks || [])
    }

    try {
      const body = responseBody as RawObject
      const blocks: RenderBlock[] = []

      // Claude 响应格式: { type: "message", content: [...] }
      if (Array.isArray(body.content)) {
        const rawContent = body.content as RawObject[]
        const contentBlocks = this.renderContentBlocks(rawContent)
        if (contentBlocks.length > 0) {
          const badges = this.getBadgesForContent(rawContent)
          blocks.push(createMessageBlock('assistant', contentBlocks, {
            roleLabel: 'Assistant',
            badges,
          }))
        }
      }

      return { blocks, isStream: false }
    } catch (e) {
      return createEmptyRenderResult(`渲染失败: ${e}`)
    }
  }

  /**
   * 渲染流式响应
   */
  private renderStreamResponse(chunks: unknown[]): RenderResult {
    if (!chunks || chunks.length === 0) {
      return createEmptyRenderResult('无响应数据')
    }

    try {
      // 先解析流式响应
      const parsed = this.parseStreamResponse(chunks)
      if (parsed.parseError) {
        return createEmptyRenderResult(parsed.parseError)
      }

      const blocks: RenderBlock[] = []

      // 渲染解析后的消息
      for (const msg of parsed.messages) {
        const contentBlocks = this.renderParsedContentBlocks(msg.content)
        if (contentBlocks.length > 0) {
          const badges = this.getBadgesForParsedContent(msg.content)
          blocks.push(createMessageBlock(msg.role, contentBlocks, {
            roleLabel: this.getRoleLabel(msg.role),
            badges,
          }))
        }
      }

      return { blocks, isStream: true }
    } catch (e) {
      return createEmptyRenderResult(`渲染失败: ${e}`)
    }
  }

  /**
   * 渲染单条消息
   */
  private renderMessage(msg: RawObject): RenderBlock | null {
    if (!msg || !msg.role) return null

    const role = msg.role as MessageRole
    const contentBlocks = this.renderMessageContent(msg.content)

    if (contentBlocks.length === 0) return null

    const badges = this.getBadgesForRawContent(msg.content)

    return createMessageBlock(role, contentBlocks, {
      roleLabel: this.getRoleLabel(role),
      badges,
    })
  }

  /**
   * 渲染消息内容
   */
  private renderMessageContent(content: unknown): RenderBlock[] {
    if (typeof content === 'string') {
      return [createTextRenderBlock(content)]
    }

    if (Array.isArray(content)) {
      return this.renderContentBlocks(content as RawObject[])
    }

    return []
  }

  /**
   * 渲染原始内容块数组
   */
  private renderContentBlocks(blocks: RawObject[]): RenderBlock[] {
    const result: RenderBlock[] = []

    for (const block of blocks) {
      const rendered = this.renderContentBlock(block)
      if (rendered) {
        result.push(rendered)
      }
    }

    return result
  }

  /**
   * 渲染单个原始内容块
   */
  private renderContentBlock(block: RawObject): RenderBlock | null {
    if (!block || !block.type) return null

    switch (block.type) {
      case 'text':
        return createTextRenderBlock(String(block.text || ''))

      case 'thinking':
        return createCollapsibleBlock(
          `思考过程 (${String(block.thinking || '').length} 字符)`,
          [createCodeBlock(String(block.thinking || ''))],
          { defaultOpen: false, className: 'thinking-block' }
        )

      case 'tool_use':
        return createToolUseRenderBlock(
          String(block.name || '工具调用'),
          this.formatJson(block.input),
          typeof block.id === 'string' ? block.id : undefined
        )

      case 'tool_result': {
        const content = this.formatToolResultContent(block.content)
        return createToolResultRenderBlock(content, block.is_error as boolean | undefined)
      }

      case 'image':
        return this.renderImageBlock(block)

      default:
        return null
    }
  }

  /**
   * 渲染已解析的内容块数组
   */
  private renderParsedContentBlocks(blocks: ContentBlock[]): RenderBlock[] {
    const result: RenderBlock[] = []

    for (const block of blocks) {
      const rendered = this.renderParsedContentBlock(block)
      if (rendered) {
        result.push(rendered)
      }
    }

    return result
  }

  /**
   * 渲染单个已解析的内容块
   */
  private renderParsedContentBlock(block: ContentBlock): RenderBlock | null {
    switch (block.type) {
      case 'text':
        return createTextRenderBlock(block.text)

      case 'thinking':
        return createCollapsibleBlock(
          `思考过程 (${block.thinking.length} 字符)`,
          [createCodeBlock(block.thinking)],
          { defaultOpen: false, className: 'thinking-block' }
        )

      case 'tool_use':
        return createToolUseRenderBlock(
          block.toolName || '工具调用',
          this.formatJson(block.input),
          block.toolId
        )

      case 'tool_result': {
        const content = typeof block.content === 'string'
          ? block.content
          : this.formatParsedToolResultContent(block.content)
        return createToolResultRenderBlock(content, block.isError)
      }

      case 'image':
        return createImageRenderBlock({
          src: block.sourceType === 'base64'
            ? `data:${block.mimeType || 'image/png'};base64,${block.data}`
            : block.url,
          mimeType: block.mimeType,
          alt: block.alt || '图片',
        })

      case 'error':
        return createErrorRenderBlock(block.message, block.code)

      default:
        return null
    }
  }

  /**
   * 渲染图片块
   */
  private renderImageBlock(block: RawObject): RenderBlock | null {
    const source = block.source as RawObject | undefined
    if (!source) {
      return createImageRenderBlock({ alt: '[图片]' })
    }

    if (source.type === 'base64') {
      return createImageRenderBlock({
        src: `data:${source.media_type || 'image/png'};base64,${source.data}`,
        mimeType: typeof source.media_type === 'string' ? source.media_type : undefined,
      })
    }

    if (source.type === 'url') {
      return createImageRenderBlock({
        src: typeof source.url === 'string' ? source.url : undefined,
        mimeType: typeof source.media_type === 'string' ? source.media_type : undefined,
      })
    }

    return createImageRenderBlock({ alt: '[图片]' })
  }

  /**
   * 获取角色显示标签
   */
  private getRoleLabel(role: MessageRole): string {
    switch (role) {
      case 'user': return 'User'
      case 'assistant': return 'Assistant'
      case 'system': return 'System'
      case 'tool': return 'Tool'
      default: return role
    }
  }

  /**
   * 获取原始内容的徽章
   */
  private getBadgesForRawContent(content: unknown): BadgeRenderBlock[] {
    if (!Array.isArray(content)) return []

    const badges: BadgeRenderBlock[] = []
    const types = new Set(content.map((b: RawObject) => b.type))

    if (types.has('thinking')) {
      badges.push(createBadgeBlock('思考', 'secondary'))
    }
    if (types.has('tool_use')) {
      badges.push(createBadgeBlock('工具调用', 'outline'))
    }
    if (types.has('tool_result')) {
      badges.push(createBadgeBlock('工具结果', 'outline'))
    }
    if (types.has('image')) {
      badges.push(createBadgeBlock('图片', 'secondary'))
    }

    return badges
  }

  /**
   * 获取内容的徽章
   */
  private getBadgesForContent(content: RawObject[]): BadgeRenderBlock[] {
    return this.getBadgesForRawContent(content)
  }

  /**
   * 获取已解析内容的徽章
   */
  private getBadgesForParsedContent(content: ContentBlock[]): BadgeRenderBlock[] {
    const badges: BadgeRenderBlock[] = []
    const types = new Set(content.map(b => b.type))

    if (types.has('thinking')) {
      badges.push(createBadgeBlock('思考', 'secondary'))
    }
    if (types.has('tool_use')) {
      badges.push(createBadgeBlock('工具调用', 'outline'))
    }
    if (types.has('tool_result')) {
      badges.push(createBadgeBlock('工具结果', 'outline'))
    }
    if (types.has('image')) {
      badges.push(createBadgeBlock('图片', 'secondary'))
    }

    return badges
  }

  /**
   * 格式化 JSON
   */
  private formatJson(input: unknown): string {
    if (typeof input === 'string') {
      try {
        const parsed = JSON.parse(input) as unknown
        return JSON.stringify(parsed, null, 2)
      } catch {
        return input
      }
    }
    return JSON.stringify(input, null, 2)
  }

  /**
   * 格式化工具结果内容
   */
  private formatToolResultContent(content: unknown): string {
    if (typeof content === 'string') {
      return content
    }

    if (Array.isArray(content)) {
      return content
        .map((item: RawObject) => {
          if (item.type === 'text') return String(item.text || '')
          if (item.type === 'image') return translate('ui.imagea71f445')
          return ''
        })
        .filter(Boolean)
        .join('\n')
    }

    return JSON.stringify(content, null, 2)
  }

  /**
   * 格式化已解析的工具结果内容
   */
  private formatParsedToolResultContent(content: ContentBlock[]): string {
    return content
      .map(block => {
        if (block.type === 'text') return block.text
        if (block.type === 'image') return translate('ui.imagea71f445')
        if (block.type === 'error') return `[错误: ${block.message}]`
        return ''
      })
      .filter(Boolean)
      .join('\n')
  }
}

/** 单例实例 */
export const claudeParser = new ClaudeParser()
