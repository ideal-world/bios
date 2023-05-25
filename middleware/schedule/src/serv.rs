/// # 任务调度服务 / Task scheduling service
/// ## 在kv缓存中存储的数据结构 / Data structure stored in kv cache
/// ### 所有注册的任务 / All registered tasks
/// `{KV_KEY_CODE}{code}`: `ScheduleJobAddOrModifyReq`
/// - 每个实例和本地缓存对比，对于本地缺失的添加，对于本地多余的删除 / Compare with local cache, add for missing, delete for extra
/// - 检查周期为配置中所配置的 / Check cycle is configured in the configuration
/// ### 已被执行的任务 / Executed tasks
/// `{config.distributed_lock_key_prefix}{code}`: `'executing'`
/// - 已被其他实例执行的任务，加分布式锁 / Tasks that have been executed by other instances, add distributed lock
/// - 不再执行已加锁的任务 / No longer execute locked tasks
/// ## 优先级 / Priority
/// - 远程缓存 > 本地缓存 / Remote cache > Local cache
/// - 写的时候先写远程缓存，再写本地缓存 / Write to remote cache first, then write to local cache
/// - 读的时候同一代码以远程为准，远程没有再读本地 / When reading, the remote is the same code, and the local is read if the remote is not
pub mod schedule_job_serv;
