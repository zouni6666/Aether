# Agent Note: 使用记录标记“实际触发过敏感信息保护”

Status: implemented

## Problem

敏感信息保护（chat PII redaction）的实际效果此前只存在于执行期：请求体里有没有被替换成占位符，只有 planner 的 `request_redacted`（HTTP）和每轮 WS 的脱敏结果知道。使用记录（`usage_requests`）在“管理员请求记录 / 用户使用记录”里展示状态、耗时、费用，但**看不出这条请求有没有真的脱敏过**。

这直接造成两类排查困难：

1. 用户报“功能失效”时，无法从记录区分「没命中任何规则」「用户/Key 没开分闸」「导出到上游的确实是明文」；此前只能靠聊天内容或上游日志反推。
2. 开启脱敏后，审计侧无法证明某条记录的请求体是占位符形态（审计 body 存的是脱敏体，但没有一个独立、可筛选的事实字段）。

## Decision

给使用记录增加一个**实际命中脱敏才为真**的标记，并贯穿到管理端与用户端的使用记录展示。

### 事实与语义

- 口径：只有请求体内**至少替换出一个占位符**（`redacted == true`）才标记。开启保护但未命中、请求在规划前失败、格式不支持（如 Gemini 原生）都不标记。
- 持久化：写进使用记录的 `request_metadata`，键 `chat_pii_redaction`，值只接受 JSON 布尔 `true`（缺失/非法视为未命中）。
- 常量收口在 `aether-data-contracts` 的 `CHAT_PII_REDACTION_METADATA_KEY`，同时进两处白名单：
  - `sanitize_usage_request_metadata`（审计行落库投影，非法形态直接丢弃）；
  - aether-usage-runtime 的窄投影（生命周期/首字事件压缩时保留该键）。

### 三个写入点

1. **HTTP 与 planner 可覆盖的 WS 轮次**：`ai_serving/planner/report_context.rs::mark_chat_pii_redaction` 在 `request_redacted == true` 时把键写进 report context。四个候选 payload builder（OpenAI Chat / OpenAI Responses / 标准同族 / 同格式透传）在 `extra_fields` 组装完成后统一调用。
2. **Responses WebSocket 规划前脱敏**：WS 在 planner 之前就把 `response.create` 脱敏，planner 看到的是占位符态、推不出“这一轮真的换了内容”。新增 `privacy::ChatPiiRedactionTurnMarker` 扩展：会话首轮（`session.rs`）与每一轮（`client.rs`）在脱敏命中时插入 planning parts，`mark_chat_pii_redaction` 见到扩展即写键。
3. **配额透明重试**：重试会重建 planning parts 并重放同一份已脱敏事件。事实存进 `LogicalTurn::client_event_pii_redacted`，`quota.rs` 重建 parts 时按它补标记，避免重试产生的记录漏标。

### 展示

- 管理端与用户端共用同一份记录 JSON：`admin_usage_record_json` / `admin_usage_active_request_json`（aether-admin）与 `user_me_usage` 的两个 builder 都输出顶层布尔 `chat_pii_redaction`。活跃（pending/streaming）行同样输出，标记在流式期间即可见。
- 前端：`UsageRecordsTable.vue` 在模型名旁显示绿色盾牌小图标（桌面与手机布局各一处，管理员/用户都可见），title 说明“发送给上游的内容已替换为占位符”；`RequestDetailDrawer.vue` 费用概览下给出说明句。轮询合并沿用 `has_fallback` 的“true 优先”规则。

## Alternatives considered

- **只在管理端显示标记** — 需求明确“用户自己的记录也要能确认脱敏是否生效”，且两端共用组件与 API，成本相同。否。
- **增加独立数据库列 / 结构化字段** — 需要迁移与读写路径改造；`request_metadata` 本来就是审计事实的扩展位（`has_fallback`、websocket 传输等都在此），投影白名单还能挡住非法形态。否。
- **增加筛选下拉（如“已脱敏”）** — 本次确认先做“小图标 + 详情说明”；筛选涉及后端 `status` 过滤分支与前端过滤策略，等有实际使用诉求再加。否（保留为后续增强）。
- **只要开启保护就标记（不要求命中）** — 会把“功能开着”误读成“这次脱敏了”，与排查场景直接冲突。否。
- **在 `resolve_provider_chat_pii_redaction` 内部直接改 report context** — 该函数拿不到 `extra_fields`，且 report context 由 payload builder 统一组装；在 builder 侧打标保持单一收口。否。
- **WS 用连接级 `redaction_restorer.has_sessions()` 推断** — 连接级为真会污染后续未命中的轮次（假阳性）；逻辑不该推断，应该记录每轮事实。否。

## Consequences

- **收益**：排查“脱敏是否生效”不再依赖聊天内容；记录可审计“上游请求体是占位符态”；管理员与用户看到同一事实。
- **代价与已知上限**：
  - WS 首轮之前的独立 `response.create` 走 `client.rs` 与 `session.rs` 两条插入路径，未来新增规划入口（新的 planner 调用点）必须一并插入 `ChatPiiRedactionTurnMarker`，否则该入口的记录会漏标；
  - 标记只表示“请求体替换过”，不表示响应侧占位符还原成功（还原由既有测试与另外的恢复链路保证）；
  - 历史记录没有该字段，前端必须按 `=== true` 判定，旧行自然不显示标记。
- **重访信号**：若产品要求“按是否脱敏筛选/统计”，在现有 `status` 过滤枚举上增加分支并同步前端 `useUsageFilters`；若支持新的客户端格式，确认 `ChatPiiRedactionRequestFormat::from_api_format` 已覆盖。

## Verification

- 投影白名单：`cargo test -p aether-data-contracts --lib` → `repository::usage::metadata_policy` 用例（保留布尔、丢弃字符串/数字形态）。
- 窄投影：`cargo test -p aether-usage-runtime --lib` → `request_metadata` 用例（生命周期压缩保留 `chat_pii_redaction`）。
- planner 打标：`cargo test -p aether-gateway --lib` → `ai_serving::planner::report_context` 用例（未命中不写、命中写 true、WS 扩展写 true）。
- 管理端记录：`cargo test -p aether-admin --lib` → `observability::usage` 用例（记录与活跃行都输出，非布尔为 false）。
- 用户端记录：`cargo test -p aether-gateway --lib` → `user_me_usage` 用例（列表与活跃 builder 都输出 true）。
- 既有回归：`cargo nextest run -p aether-gateway -E 'test(pii) | test(/redaction::tests/)'` 全绿。
- 展示层：`frontend` 里 `UsageRecordsTable.spec.ts` 断言管理端/用户端各渲染两个标记（桌面+手机）、未脱敏不渲染。
