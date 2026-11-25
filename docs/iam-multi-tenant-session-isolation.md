# BIOS IAM 多租户会话隔离技术方案

| 项目 | 内容 |
|---|---|
| 状态 | 已实现（待评审合入） |
| 适用模块 | `bios-iam`、`bios-auth`（Spacegate Auth 插件） |
| 目标读者 | IAM / 网关 / 前端通行证 |
| 分支 | `feat/tenant-scoped-session` |
| 日期 | 2026-11-24 |

> 引用约定：文中 `文件名:行号` 均为可点击源码链接。若代码演进导致行号漂移，以函数/常量为准。

---

## 一、背景与问题

### 1.1 现象

同一个账号在两个不同浏览器（或设备）上分别登录不同租户后，**先登录的会话会被后登录的会话「覆盖」**：A 浏览器停在租户 A，B 浏览器登录/切换到租户 B 后，A 浏览器刷新页面或下一次请求就变成租户 B 的身份与权限，或者直接返回 401。

表现为三类可观察结果：

1. 租户被静默覆盖：A 的后续请求拿到的是 B 的 `own_paths`、roles、groups。
2. 应用权限串味：A 访问应用时，命中的是 B 租户下的应用上下文，可能出现「本无权限却放行」或「本有权限却 401」。
3. 无法自愈：只能重新登录，且登录 A 之后 B 又被覆盖，两边来回抖动。

### 1.2 根因

会话身份只有 `account_id`，**没有租户维度**。

| # | 结论 | 代码位置 |
|---|---|---|
| 1 | token 只映射到 `kind,account_id[,renewal_sec]`，不含租户 | [auth_kernel_serv.rs:319](../backend/supports/auth/src/serv/auth_kernel_serv.rs#L319) |
| 2 | 账号上下文 hash 的 field `""` 是账号级唯一的「当前租户」槽位，所有 token 共享 | [iam_key_cache_serv.rs:430](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L430) |
| 3 | 应用上下文 hash 的 field 是 `{app_id}`，同一应用在所有租户共用一个槽位 | [iam_key_cache_serv.rs:430](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L430) |
| 4 | Auth 侧只按 `account_id` 读取，天然读到「最后一次写入」的租户 | [auth_kernel_serv.rs:347](../backend/supports/auth/src/serv/auth_kernel_serv.rs#L347) |
| 5 | 切换租户直接改写这份共享状态，token 本身不变 | [iam_cp_cert_api.rs:592](../backend/supports/iam/src/console_passport/api/iam_cp_cert_api.rs#L592) |
| 6 | 默认允许多会话并存（`coexist_num = 5`），两个浏览器不会互相踢下线 | [iam_constants.rs:57](../backend/supports/iam/src/iam_constants.rs#L57) |

第 6 条是关键前提：多会话并存本身是对的，错的是**多个会话共用同一份租户状态**。

### 1.3 为什么前端分仓解决不了

在 localStorage / cookie 上按租户分 key，只能改「前端拿哪个 token」，后端 Redis 里的租户槽位仍然是账号级唯一。只要 B 浏览器登录一次，A 的 token 在服务端解析出的租户就已经变了。因此必须在服务端把会话作用域补成 `(account_id, tenant_id)`。

---

## 二、设计目标

| # | 目标 | 落点 |
|---|---|---|
| 1 | 一个登录会话 = `(account_id, tenant_id)`，租户由 token 决定 | §3.1、§3.2 |
| 2 | 切换租户只影响当前 token，不影响同账号其他会话 | §3.3 |
| 3 | 租户权限、应用权限的计算逻辑不变，只改存储/读取位置 | §4 |
| 4 | 旧 token / 滚动发布期间可平滑兼容 | §3.4、§5 |
| 5 | 切换租户必须校验目标租户可达性 | §3.3 |
| 6 | OAuth2 访问令牌同样带租户作用域 | §3.5 |

---

## 三、方案设计

### 3.1 扩展 token 值时带上租户

`iam:cache:token:info:{token}` 的值从两/三段扩展为四段：

```text
{token_kind},{account_id},{expire},{tenant_id}
```

- `expire`：续期秒数，允许为空（不续期）；
- `tenant_id`：会话所属租户，允许为空。**段存在但为空 = 绑定平台上下文**，段缺失 = 旧数据未绑定；
- 只有 2 段或 3 段的旧值仍可读取，此时视为未绑定，回退账号级共享上下文；
- 生成与解析统一走两侧同名工具函数 `build_token_info` / `parse_token_info`。

配置项：两侧共用既有 `cache_key_token_info` / `cache_key_token_info_`，**不新增 Redis key**。

工具函数：

- Auth：[auth_constants.rs](../backend/supports/auth/src/auth_constants.rs)
- IAM：[iam_constants.rs](../backend/supports/iam/src/iam_constants.rs)

写入链路：登录 → `package_tardis_context_and_resp` → `IamCertTokenServ::add_cert`（[iam_cert_token_serv.rs:100](../backend/supports/iam/src/basic/serv/iam_cert_token_serv.rs#L100)）→ `IamIdentCacheServ::add_token`（[iam_key_cache_serv.rs:41](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L41)），租户从登录参数一路透传下去。

> ⚠️ 发布顺序：旧版 Auth 在 `account_info.len() > 2` 时会把第 3 段当续期秒数解析。因此必须**先升级网关（Auth 插件）**，再让 IAM 写入四段值，否则滚动期间未升级的网关会对新 token 返回 500。

### 3.2 账号上下文按租户分片

`iam:cache:account:info:{account_id}` 的 hash field 约定变为：

| field | 内容 | 状态 |
|---|---|---|
| `tenant:{tenant_id}` | 该租户的租户上下文（roles/groups/own_paths） | 新增，正式读取 |
| `app:{tenant_id}:{app_id}` | 该租户下该应用的上下文 | 新增，正式读取 |
| `""` | 旧版租户上下文 | 保留，仅兼容旧 token |
| `{app_id}` | 旧版应用上下文 | 保留，仅兼容旧 token |
| `is_global` | 是否全局账号 | 不变 |

field 命名函数在两侧保持一致：

- Auth：[auth_constants.rs:13](../backend/supports/auth/src/auth_constants.rs#L13)
- IAM：[iam_constants.rs:68](../backend/supports/iam/src/iam_constants.rs#L68)

写入：[iam_key_cache_serv.rs:430](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L430)
读取：[iam_key_cache_serv.rs:484](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L484)、[auth_kernel_serv.rs:347](../backend/supports/auth/src/serv/auth_kernel_serv.rs#L347)

写入时还会**清理本租户已失效的应用 field**（先 `hkeys` 再按前缀比对），避免取消应用授权后仍命中旧权限。

### 3.3 切换租户只重绑当前 token

`PUT /iam/cp/token/switch`（[iam_cp_cert_api.rs:592](../backend/supports/iam/src/console_passport/api/iam_cp_cert_api.rs#L592)）语义保持「token 不变」，新增两点：

1. **前置校验**：目标租户启用 + 账号对目标租户可达（[iam_account_serv.rs:1074](../backend/supports/iam/src/basic/serv/iam_account_serv.rs#L1074) `can_access_tenant`）。可达性判定依次为：
   - 目标为空（平台上下文）或账号是全局账号 → 允许；
   - 账号自身所属租户 == 目标租户 → 允许；
   - 账号在目标租户存在角色关系（用全局作用域查询，避免被当前 token 的租户限制）→ 允许;
   - 账号在目标租户或平台层拥有 Apps 集合授权 → 允许。
2. **只重绑当前 token**：`rebind_token_tenant`（[iam_key_cache_serv.rs:113](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L113)）直接改写 `token:info` 值的第 4 段，TTL 沿用原 token 剩余有效期（`ttl` 为 `-1/-2` 时退化为无过期写入）。

改完之后，A、B 两个浏览器各持不同 token，各自读写自己的租户上下文，互不影响。

### 3.4 兼容与迁移

| 场景 | 行为 |
|---|---|
| 旧 token（2 段 / 3 段） | 解析出 `tenant_id = None`，认证读取 field `""` / `{app_id}`，与改动前一致 |
| 新 token（4 段，租户非空） | 读取 `tenant:{tenant_id}` / `app:{tenant_id}:{app_id}` |
| 新 token（4 段，租户为空） | 视为平台上下文，读取 `tenant:` 分片 |
| 滚动发布（新 IAM + 旧 Auth） | ⚠️ 旧 Auth 会把第 3 段当续期秒数；**必须先升级网关再升级 IAM** |
| 旧 IAM + 新 Auth | IAM 仍写旧格式 → `tenant_id = None` → 回退旧 field，等价改动前行为 |
| token 续期 | 按第 3 段 `set_ex` 原值，租户段随值保留（[auth_kernel_serv.rs:319](../backend/supports/auth/src/serv/auth_kernel_serv.rs#L319)） |

**收尾**：等旧 token 全部自然过期（默认 7 天，见 `RBUM_CERT_CONF_TOKEN_EXPIRE_SEC`）后，可移除 `add_contexts` 中的旧 field 双写，以及读取侧的 `None` 回退分支。

### 3.5 OAuth2 访问令牌

[iam_cert_oauth2_service_serv.rs](../backend/supports/iam/src/basic/serv/iam_cert_oauth2_service_serv.rs)：

- 授权码换 token 时，从授权时保存的 `ctx.own_paths` 推导租户并绑定（[:468](../backend/supports/iam/src/basic/serv/iam_cert_oauth2_service_serv.rs#L468)）；
- refresh token 记录新增 `tenant_id` 字段，带 `#[serde(default)]` 兼容线上旧数据（[:170](../backend/supports/iam/src/basic/serv/iam_cert_oauth2_service_serv.rs#L170)）；
- 刷新签发的新 access token 继承同一租户（[:579](../backend/supports/iam/src/basic/serv/iam_cert_oauth2_service_serv.rs#L579) 的 `add_oauth2_token`）。

---

## 四、对租户权限与应用权限的影响

结论：**权限集合的计算逻辑不变，变的是「每个 token 只读自己租户/应用的上下文」**。对权限而言这是修复漏洞，不是收紧或放宽。

### 4.1 租户权限（roles / groups）

- roles/groups 仍由 `get_account_detail_aggs` 以目标租户 `ctx.own_paths` 计算，逻辑未动；
- 改动前：A 的 token 会读到 B 租户的 roles/groups，属于**跨租户串权**；
- 改动后：每个 token 只合并自己租户的 roles/groups。

### 4.2 应用权限

- 改动前：hash field 是 `{app_id}`，**同一应用在所有租户共用一个槽位**，后登录的租户会覆盖前一租户的应用角色；
- 改动后：`app:{tenant_id}:{app_id}`，A 只能读到 `A/app` 的角色；
- app 请求合并租户角色时，从同一个 `tenant:{tenant_id}` 取，不会再出现「A 的应用上下文 + B 的租户角色」。

### 4.3 不受影响的链路

| 链路 | 原因 |
|---|---|
| AK/SK 认证 | AK 记录本身带 `tenant_id`，不读账号上下文 |
| `TokenTask` 委托令牌 | 本身就是 `iam:task-token:{token}` 按 token 存完整 ctx |
| 权限判定 | rbum 资源/角色匹配仍是基于 `TardisContext.own_paths/roles/groups`，只是 ctx 来源更准确 |

### 4.4 同步改造的权限相关点

| # | 点 | 说明 | 位置 |
|---|---|---|---|
| 1 | 刷新要带租户 | 角色/应用变更时按实际租户刷新，不再按「最后一次写入的租户」 | [iam_key_cache_serv.rs:398](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L398)、[iam_rel_serv.rs:83](../backend/supports/iam/src/basic/serv/iam_rel_serv.rs#L83) |
| 2 | 平台上下文单独 field | 全局账号写入 `tenant:`（tenant_id 为空串），不与兼容用的 `""` 混用 | [iam_key_cache_serv.rs:430](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L430) |
| 3 | switch 成员校验 | 防止 token 绑到无成员关系的租户 | [iam_cp_cert_api.rs:620](../backend/supports/iam/src/console_passport/api/iam_cp_cert_api.rs#L620) |
| 4 | 临时产品授权校验租户 | 临时授权必须属于当前会话绑定的租户，不能跨租户放行 | [auth_kernel_serv.rs:347](../backend/supports/auth/src/serv/auth_kernel_serv.rs#L347) |
| 5 | webhook AK 分支 | 以查询参数 `own_paths` 首段作为租户读取上下文 | [auth_kernel_serv.rs:263](../backend/supports/auth/src/serv/auth_kernel_serv.rs#L263) |
| 6 | 租户禁用只清本租户会话 | `delete_tokens_by_tenant` 按 token 绑定过滤 | [iam_key_cache_serv.rs:371](../backend/supports/iam/src/basic/serv/iam_key_cache_serv.rs#L371) |
| 7 | 内部取上下文接口 | `/iam/ci/account/{id}/ctx` 新增 `tenant_id` 查询参数，不传保持旧行为 | [iam_ci_account_api.rs:414](../backend/supports/iam/src/console_interface/api/iam_ci_account_api.rs#L414) |

> 注意：平台级角色变更需要遍历该账号所有租户上下文 + 平台上下文才能全部刷新。这是本次改造暴露出的既有缺口（改动前只会刷新最后一个租户），建议在后续迭代把「账号所属租户列表」纳入刷新范围。

---

## 五、缓存结构总览

```text
iam:cache:token:info:{token}       -> "{token_kind},{account_id},{expire},{tenant_id}"
                                     expire / tenant_id 可为空；2~3 段为旧格式  既有 key，值扩展
iam:cache:account:info:{account_id}  (hash)
    tenant:{tenant_id}             -> 租户上下文                                    新增
    app:{tenant_id}:{app_id}       -> 应用上下文                                    新增
    ""                             -> 旧版租户上下文                                 兼容
    {app_id}                       -> 旧版应用上下文                                 兼容
    is_global                      -> "true" / "false"                              既有
```

---

## 六、验证

### 6.1 编译

```bash
cargo check --workspace
cargo check -p bios-auth --all-targets --features web-server
cargo check -p bios-iam --all-targets
```

### 6.2 新增聚焦用例

| 用例 | 覆盖点 | 位置 |
|---|---|---|
| 值格式兼容（单元） | 2/3 段旧值、`expire` 为空、租户段为空（平台）、`build`/`parse` 往返 | [auth_constants.rs](../backend/supports/auth/src/auth_constants.rs)、[iam_constants.rs](../backend/supports/iam/src/iam_constants.rs) |
| Auth 双 token 双租户 | 两个 token 分别读到各自租户的 roles；应用上下文不串；旧 token 回退 | [test_tenant_session.rs:11](../backend/supports/auth/tests/test_tenant_session.rs#L11) |
| IAM 会话隔离 | 写入/读取隔离；只重绑一个 token 不影响另一个；旧 token 回退 | [test_tenant_session.rs:13](../backend/supports/iam/tests/test_tenant_session.rs#L13) |
| Auth 端到端 | 走 `/auth` 接口验证上下文与权限合并 | [test_auth_req.rs:297](../backend/supports/auth/tests/test_auth_req.rs#L297) |
| IAM 端到端 | 登录 → 缓存 → 切换 → 读取全链路 | [test_key_cache.rs:1191](../backend/supports/iam/tests/test_key_cache.rs#L1191) |

### 6.3 建议人工回归

1. A 浏览器登录租户 A，B 浏览器登录租户 B，A 反复刷新，租户与权限保持 A。
2. B 在 A 已登录的情况下切租户，A 不受影响。
3. 账号在 A 有应用权限、B 无：B 访问该应用应被拒，且不会拿到 A 的应用角色。
4. A 内变更角色 → 只刷新 A；B 上下文不变。
5. `/token/switch` 切到无成员关系的租户 → 403。
6. 租户禁用 → 只下线该租户的会话。

---

## 七、已知限制与后续

| # | 限制 | 说明 / 建议 |
|---|---|---|
| 1 | 同一浏览器多标签页仍共享 token | 多标签页共享 localStorage，会一起切租户；如需支持，前端改为 `sessionStorage` 或按租户分 key |
| 2 | 平台级角色变更的刷新范围 | 需要遍历账号所有租户上下文，见 §4.4 注 |
| 3 | 兼容字段的清理 | 等旧 token 过期（默认 7 天）后移除双写与回退分支 |
| 4 | 缓存 key 规模 | 无新增 key：租户随 `token:info` 的值一起存储，量级不变 |
