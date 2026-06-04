# Auth / IAM 审查报告

- 仓库：`ideal-world/bios`
- 分支：`review/full-workspace-2026-04-18`
- 日期：`2026-04-18`
- 范围：`backend/supports/auth`、`backend/supports/iam`
- 本轮修复提交：`96d7b2d1`

## 审查结论

本轮重点围绕 **安全风险** 与 **性能/稳定性风险** 对 `auth`、`iam` 两个模块进行了深度审查，并直接修复了已确认的 P0/P1 问题。

## 已修复项

### P0（严重）

| #   | 文件                                                        | 问题                                                       | 修复                                  |
| --- | ----------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------- |
| 1   | `backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs` | `trace!("add aksk: ak={},sk={}", ak, sk)` 明文记录 SK      | 改为仅记录 `sk_len`                   |
| 2   | `backend/supports/auth/src/serv/auth_kernel_serv.rs`        | webhook 签名仅校验未来时间，不校验过期，历史签名可无限重放 | 补充 `head_date_interval_ms` 过期检查 |

### P1（高）

| #   | 文件                                                                           | 问题                                                                                              | 修复                                               |
| --- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| 3   | `backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs`                    | trace 日志泄露会话 token 明文                                                                     | 改为仅记录 `token_len`                             |
| 4   | `backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs`                    | delete 路径同样泄露 token 明文                                                                    | 改为仅记录 `token_len`                             |
| 5   | `backend/supports/iam/src/iam_initializer.rs`                                  | `info!("Initial password: {}", pwd)` 将初始化密码写入日志系统                                     | 不再写日志，密码仅通过返回值下发                   |
| 6   | `backend/supports/iam/src/basic/serv/oauth2_spi/iam_cert_oauth2_spi_github.rs` | GitHub OAuth `client_secret` 放在 URL query string 中                                             | 迁移到 `application/x-www-form-urlencoded` 请求体  |
| 7   | `backend/supports/auth/src/serv/auth_res_serv.rs`                              | `get_apis_json()` 内部使用 `block_on(fetch_public_key())`，从异步 handler 进入时存在阻塞/死锁风险 | 改为 `async fn`，移除 `block_on`                   |
| 8   | `backend/supports/auth/src/api/auth_kernel_api.rs`                             | 旧调用点仍为同步风格                                                                              | 改为 `.await?`                                     |
| 9   | `backend/supports/auth/src/serv/auth_crypto_serv.rs`                           | `.expect("sm keys none")` 在未初始化时会 panic                                                    | 改为返回显式错误 `500-auth-crypto-not-initialized` |
| 10  | `backend/supports/auth/tests/test_auth_init.rs`                                | `get_apis_json()` 调用未跟随 async 化调整                                                         | 测试调用点同步更新为 `.await?`                     |

## 变更说明

### auth 模块

- `parsing_base_ak`：webhook 分支补齐“过去时间窗口”校验，避免泄露的旧签名长期有效。
- `auth_res_serv::get_apis_json`：去除 `futures::executor::block_on`，避免在异步链路里把 executor 线程卡成“人工单线程模式”。
- `auth_crypto_serv::decrypt_req`：SM2 密钥未初始化时不再 panic，而是返回明确错误，避免单请求触发进程级异常。

### iam 模块

- `iam_key_cache_serv`：AK/SK、token 不再以明文进入 trace 日志。
- `iam_initializer`：系统管理员初始密码不再出现在 `info!` 日志中，防止被日志平台、代理或审计系统长期留存。
- `iam_cert_oauth2_spi_github`：GitHub OAuth 的 `client_secret` 从 URL 搬到 POST 表单体中，减少 access log / proxy log 泄露面。

## 保留建议项

| #   | 等级 | 位置                                                                              | 建议                                                                                                      |
| --- | ---- | --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| 1   | P1   | `backend/supports/auth/src/serv/auth_kernel_serv.rs`                              | 当前 AK/SK 仍缺少 nonce 防重放；建议后续引入 Redis `SETNX` + TTL 方案                                     |
| 2   | P2   | `backend/supports/iam/src/basic/serv/oauth2_spi/iam_cert_oauth2_spi_wechat_mp.rs` | WeChat `jscode2session` 的 `secret` 位于 query string 属接口规范限制，建议在运维层禁记该 URL 的完整查询串 |
| 3   | P2   | `backend/supports/auth/src/serv/auth_res_serv.rs`                                 | `RES_CONTAINER` / `RES_APIS` 仍为 `std::sync::RwLock`，后续可评估替换为异步锁或更细粒度方案               |
| 4   | P2   | `backend/supports/iam/src/iam_config.rs`                                          | `BASIC_INFO.lock().unwrap_or_else(                                                                        | e | panic!(...))` 仍存在 panic 风险，建议后续改为错误返回 |
| 5   | P2   | `backend/supports/auth/src/serv/auth_kernel_serv.rs`                              | 非 webhook AK/SK 请求虽然有时间窗限制，但时间窗内仍可重放；建议与 nonce 方案一并处理                      |
| 6   | P3   | `backend/supports/iam/src/basic/serv/iam_cert_oauth2_serv.rs`                     | OAuth2 绑定账户使用的内部密码随机性仍偏弱，建议后续提升熵值                                               |

## 验证结果

已执行并通过：

- `cargo check -p bios-iam --lib`
- `cargo check -p bios-auth --lib --features web-server`

说明：

- `bios-auth` 的若干测试目标存在 **既有 feature 开关问题**（如 `web_resp` 位于 `web-server` feature 后），这不是本次改动引入的问题。
- 因当前主机环境缺少部分全局依赖，workspace 级完整校验不适合作为本轮唯一验收标准，因此采用按 package 校验的方式完成验证。

## 结论

本轮已经完成 auth / iam 两个模块中 **已确认 P0/P1 安全与性能问题** 的直接修复，并完成针对性编译验证。整体风险显著下降，尤其是：

- 凭据泄露面收敛
- webhook 历史签名重放风险关闭
- 异步链路中的 `block_on` 风险移除
- 密钥未初始化时的 panic 改为可观测错误

后续建议优先推进 **nonce 防重放** 与 **锁/配置 panic 改造** 两项，以把剩余 P1/P2 风险继续压平。
