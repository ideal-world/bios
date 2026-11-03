# BiosIam Provider 凭据的并发访问

同一账号的登录凭据写入、普通 token 读取、显式刷新、task-grant 创建前校验和后台兑换，共用 Redis Provider 租约。不同账号使用不同的租约键。

## 争锁行为

- 争锁时在 IAM 内等待，间隔从 25ms 递增，最多 200ms；不再立即把普通竞争返回给调用方。
- token 读取、显式刷新、task-grant 创建前校验及后台兑换的等待和 Provider 操作共用 40 秒截止时间。后台兑换不再使用独立的 5 秒等待窗口。
- 取得租约后重新读取当前凭据并执行原有配置、账号、外部身份和 scope 校验。普通读取复用已提交的刷新结果，避免并发使用旧 refresh token。
- 显式刷新在请求开始时读取以持久化记录为准的凭据快照，快照阶段不回填缓存；若等待期间其他请求已更新凭据且新凭据未过期，则复用新结果。后续独立的显式刷新仍调用 Provider。
- 争锁超时继续返回 `503` / `OAuth credential refresh is in progress; retry later`；Provider 操作超时返回原有的 `504`。缓存错误和真实授权错误直接返回，不作为争锁重试。
- Redis 租约仍为 120 秒，释放时核对持有者，保持跨实例互斥和 refresh token 持久化顺序。

## 交付与验证

无需 SQL 迁移，不改变 HTTP 接口或 Hub 的 task-grant 契约。修复需要发布新的 BIOS IAM 服务，或由内嵌 IAM 的宿主升级 BIOS revision 后重新构建。仅发布独立 BIOS 不会更新宿主内嵌的 IAM。

Hub 的有界 HTTP 重试可以继续保留，用于旧 BIOS 和暂时竞争；BIOS 内部直接等待可减少失败响应和重复 HTTP 请求。

Docker 集成回归：

```bash
cargo test -p bios-iam --test test_iam_oauth2_provider_concurrency -- --test-threads=1
```

该测试使用隔离 PostgreSQL、Redis、RabbitMQ 和模拟 Provider，覆盖有效凭据争锁、不同账号独立推进、6 秒慢刷新仅调用一次 Provider、并发显式刷新合并、后续独立刷新，以及 Provider 故障后的租约释放和恢复。单元测试另覆盖等待上限、缓存请求卡住及缓存错误不重试。真实外部 Provider 与部署环境仍需独立验收。

现有 task-grant 集成 fixture 已补上新 `IamRoleAddReq.perm_kind` 的 `None` 默认值，使该回归能在当前 BIOS main 上编译；不改变角色权限逻辑。
