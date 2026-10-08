# 提供商客户端画像

网关统一管理客户端身份，但不把所有提供商伪装成同一个客户端。
客户端发行版本、SDK/运行时模板、浏览器构建和账户/会话标识是不同维度。
本次改造保留已有认证、请求体转换、原生端点和显式配置优先级。

## 策略

| 客户端 | 版本来源 | 检查间隔 | 不随发行版本改变的内容 |
| --- | --- | --- | --- |
| Codex CLI/Core | 官方 `@openai/codex` npm latest | 每日 | `codex_cli_rs` originator、现有 OS/架构 UA 模板 |
| Claude Code | 官方 `@anthropic-ai/claude-code` npm latest | 每日 | 已校准的 Stainless SDK、运行时字段和 beta/能力模板 |
| Grok CLI | `https://x.ai/cli/stable`，失败回退官方 npm latest | 3 小时 | 现有 CLI 身份字段、OAuth/API-key 路由与自定义网关优先级 |
| Gemini CLI | 官方 `@google/gemini-cli` npm latest | 每日 | 现有 `(Windows; AMD64)` UA 模板与 Code Assist 请求协议 |
| Antigravity、Windsurf | 经核验的固定模板 | 不自动更新 | IDE UA、请求体版本及现有平台模板 |
| Kiro | 已有 OAuth SDK 模板及显式配置 | 不自动更新 | SDK、Node 和系统版本；保留旧模型查询兼容回退 |
| ChatGPT Web、Grok Web | 固定浏览器构建或已有可配置浏览器画像 | 不自动更新 | 浏览器 UA、Client Hints、TLS/HTTP 指纹及 Web 构建信息 |
| 原生/其他提供商 | 原有协议和配置 | 无 CLI 版本刷新 | 不注入无依据的 CLI、SDK 或浏览器指纹 |

Codex Desktop 不仅凭客户端名称切换 originator 或 SDK 字段；沿用已确认的
Codex Core 身份。代理客户端传入的版本不是官方发行版本来源。
不会为 Rust CLI 强行添加 JavaScript Stainless 字段。

## 集群刷新与本地同步

- `apps/aether-gateway/src/cli_client_profile.rs` 保存四个动态 CLI 描述和共同引擎。
- 启动预热和后台 singleton 共用发行查询互斥锁。新鲜的已验证缓存避免多个节点
  启动时重复查询官方源。原有缓存 key 和字段保持兼容。
- 各进程独立运行缓存同步任务，每 60 秒同步一次，包括仅承接请求的 frontdoor 节点。
  此任务不查询发行源、不竞争 singleton；启动守卫随主程序退出而中止任务。
- 多节点必须使用同一个共享运行时缓存。内存后端只提供进程内的缓存和互斥语义。
- 自动恢复和刷新不接受比本地画像更旧的稳定版本。外部查询等待期间若共享缓存已升级，
  发布前再次恢复并检查版本，拒绝过期结果。
- 官方请求仅允许 HTTPS，禁止重定向及系统环境代理；连接、请求和响应大小均有上限。
  npm 元数据需要包名、稳定版本以及相应原生平台依赖一致。Gemini CLI 的官方 JS 包
  不要求不存在的原生平台依赖。
- 发行源、校验或缓存读取失败不清空现有画像。缓存写入失败保留本地已验证画像并告警；
  其他节点只能继续使用最近成功持久化的画像，因此需关注缓存失败日志并恢复共享缓存。

## 配置

| 客户端 | 固定版本 | 禁用外部刷新 |
| --- | --- | --- |
| Codex | `AETHER_CODEX_CLIENT_VERSION` | `AETHER_CODEX_CLIENT_PROFILE_REFRESH=false` |
| Claude Code | `AETHER_CLAUDE_CODE_CLIENT_VERSION` | `AETHER_CLAUDE_CODE_CLIENT_PROFILE_REFRESH=false` |
| Grok CLI | `AETHER_XAI_CLIENT_VERSION` | `AETHER_XAI_CLIENT_PROFILE_REFRESH=false` |
| Gemini CLI | `AETHER_GEMINI_CLI_CLIENT_VERSION` | `AETHER_GEMINI_CLI_CLIENT_PROFILE_REFRESH=false` |

有效固定版本优先于缓存、内置值和外部发行检查，可用于显式回退；该覆盖只改变当前
进程，不写入集群缓存。需要集群固定版本时应为各节点配置同一个值。
禁用刷新仅停止外部查询，仍允许从共享缓存同步。未配置固定版本或刷新开关时，
默认使用缓存/内置值并启用外部检查。

## 请求边界

`aether-ai-formats::client_profile::ClientProfileStore` 用不可变 `Arc` 快照发布画像。
请求已经持有的快照不会被后台更新修改。

- Grok CLI 默认版本头和 UA 从同一个快照产生；保留既有显式头覆盖规则。
- Gemini CLI 的推理、配额和模型发现读取同一进程内画像来源，而不是各自维护版本常量。
- Claude Code 在共同出站策略边界重新对齐固定头、beta 和 billing `cc_version`，
  使用一次取得的画像完成头/体校准。保留内容协商和流式行为；count-tokens 不执行
  消息请求体改写。
- Codex 使用现有受保护认证头、Core UA/originator 和可选会话指纹收敛策略；不改变
  Responses-lite、compact、agent-identity 和原生记忆端点的契约。
- 客户端的 `x-envoy-internal` 不能透传为上游可信内部标记；同格式、跨格式、
  passthrough 和共同出站策略均包含过滤。
- 不把用户 Bearer、Cookie、账号 ID、安装/窗口 ID 或 SDK 字段保存为发行画像。

静态模板集中在 `crates/aether-provider/transport/src/client_identity.rs`；
Kiro 复用 OAuth 的 SDK 模板，Grok Web 继续复用既有浏览器画像。
静态版本只能按核验后的整套模板更新，不能直接套用 npm 的 CLI 版本。

## 回归验证

```bash
cargo fmt --all -- --check
cargo test -p aether-ai-formats --locked
cargo test -p aether-provider-transport --locked
cargo test -p aether-provider-pool --locked
cargo test -p aether-model-fetch --locked
cargo check -p aether-gateway --locked
cargo test -p aether-gateway --lib client_profile --locked
```

回归覆盖不可变快照、头/体版本一致性、稳定源回退、npm 平台校验、缓存损坏和防回退、
刷新期间共享缓存升级、独立节点同步、固定版本优先级，以及保留原有提供商协议行为。
验证使用本地数据和模拟回调，不需要个人凭据或付费模型调用。

## 已执行验证

- Transport：550 项通过；formats：993 项通过；pool：71 项通过；model-fetch：83 项通过。
- `cargo check -p aether-gateway --locked` 通过。
- 网关专项：client_profile 21 项、Codex 249 项、Gemini CLI 39 项通过；
  Claude Code 13 项通过，1 项失败。共 2019 项通过。
- 格式与 `git diff --check` 通过。
- 已知失败：`handlers::shared::catalog::tests::provider_key_status_snapshot_payload_backfills_claude_code_usage_windows`。
  在独立 worktree 的改造前提交 `716c35bf5` 上单独运行，同样在 `catalog.rs:4373`
  得到 `exhausted=false` 与预期 `true` 的断言失败。该配额展示逻辑未在本次改造中修改，
  此失败不能计作通过，也不是画像改造引入的回归。

相关说明：[Codex CLI 对齐](codex-cli-alignment.md)、[xAI 行为](xai-provider.md)。
