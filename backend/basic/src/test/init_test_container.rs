//! Init test container
use std::env;

use tardis::basic::result::TardisResult;
use tardis::testcontainers::core::{Mount, WaitFor};
use tardis::testcontainers::runners::AsyncRunner;
use tardis::testcontainers::{ContainerAsync, Image, ImageExt};
use tardis::TardisFuns;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::rabbitmq::RabbitMq;
use testcontainers_modules::redis::Redis;

pub struct LifeHold {
    pub reldb: ContainerAsync<Postgres>,
    pub redis: ContainerAsync<Redis>,
    pub rabbit: ContainerAsync<TestRabbitMq>,
    /// 本次测试自建的隔离网络（仅 `BIOS_TEST_DOCKER_NETWORK_ISOLATED` 开启时存在）；
    /// 声明在容器之后，drop 时最后回收，且只删除本测试创建的网络。
    #[allow(dead_code)]
    pub network: Option<IsolatedNetwork>,
}

/// 测试专用隔离 docker network：用 `com.docker.network.bridge.host_binding_ipv4=127.0.0.1`
/// 让本网络内发布到宿主机的端口只绑定 loopback（testcontainers 0.23 的 PortMapping 不支持
/// 指定 host ip，这是不改 registry/daemon 的隔离方案）。只影响本测试自建网络。
pub struct IsolatedNetwork {
    name: String,
}

impl IsolatedNetwork {
    fn create(prefix: &str) -> TardisResult<Self> {
        let name = format!("{prefix}net-{}", TardisFuns::field.nanoid());
        // 宿主默认地址池可能耗尽，允许显式指定一个空闲子网（如 10.63.0.0/24）。
        let subnet = env::var("BIOS_TEST_DOCKER_NETWORK_SUBNET").ok().filter(|subnet| !subnet.trim().is_empty());
        let mut args = vec!["network", "create", "--opt", "com.docker.network.bridge.host_binding_ipv4=127.0.0.1"];
        let subnet_arg;
        if let Some(subnet) = &subnet {
            subnet_arg = subnet.trim().to_string();
            args.extend_from_slice(&["--subnet", subnet_arg.as_str()]);
        }
        args.push(&name);
        let output = std::process::Command::new("docker")
            .args(&args)
            .output()
            .map_err(|error| tardis::basic::error::TardisError::internal_error(&format!("cannot run docker network create: {error}"), "test-container-network-create"))?;
        if !output.status.success() {
            return Err(tardis::basic::error::TardisError::internal_error(&format!("docker network create failed: {}", String::from_utf8_lossy(&output.stderr)), "test-container-network-create"));
        }
        eprintln!("[BIOS.Test] isolated loopback docker network ready: {name}");
        Ok(Self { name })
    }
}

impl Drop for IsolatedNetwork {
    fn drop(&mut self) {
        // 容器由 ContainerAsync 的异步 drop 回收，短暂重试以等待其脱离网络；只删自有网络。
        for _ in 0..10 {
            match std::process::Command::new("docker").args(["network", "rm", &self.name]).status() {
                Ok(status) if status.success() => return,
                _ => std::thread::sleep(std::time::Duration::from_millis(300)),
            }
        }
        eprintln!("[BIOS.Test] could not remove isolated network {}; remove it manually", self.name);
    }
}

/// RabbitMQ 测试镜像适配器：保留 module 的默认镜像，但不绑定特定版本的插件数量。
/// `rabbitmq:3-management` 同样是管理插件镜像，启动日志尾部的插件数量会随版本变动。
#[derive(Debug, Default, Clone)]
pub struct TestRabbitMq(RabbitMq);

impl Image for TestRabbitMq {
    fn name(&self) -> &str { self.0.name() }
    fn tag(&self) -> &str { self.0.tag() }
    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Server startup complete;")]
    }
}

/// 测试专用可选镜像 tag 覆盖：仅当显式设置 env 时生效，默认值保持既有行为不变。
/// 用于离线/受限网络环境复用本机已存在的镜像（如 `postgres:16-alpine`、`redis:7-alpine`、
/// `rabbitmq:3-management`），只影响测试容器，不做 retag，也不触碰任何部署配置。
fn test_image_tag(env_key: &str, default_tag: &str) -> String {
    env::var(env_key)
        .ok()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .unwrap_or_else(|| default_tag.to_string())
}

/// 测试专用可选容器名前缀（`BIOS_TEST_CONTAINER_NAME_PREFIX`）：缺省时不显式命名（沿用
/// testcontainers 的随机名）；设置后容器命名为 `<prefix><base>-<nanoid>`，便于联调时按前缀
/// 识别并只回收本次测试自建的容器，不触碰无关资源。
fn test_container_name(base: &str) -> Option<String> {
    let prefix = env::var("BIOS_TEST_CONTAINER_NAME_PREFIX")
        .ok()
        .map(|prefix| prefix.trim().to_string())
        .filter(|prefix| !prefix.is_empty())?;
    Some(format!("{prefix}{base}-{}", TardisFuns::field.nanoid()))
}

/// 独立的 Redis 测试初始化器：默认镜像与 `TardisTestContainer::redis_custom` 一致，
/// 可用 `BIOS_TEST_REDIS_TAG` 覆盖（如 `7-alpine`）。
pub async fn redis_custom() -> TardisResult<ContainerAsync<Redis>> {
    redis_custom_on(None).await
}

async fn redis_custom_on(network: Option<&str>) -> TardisResult<ContainerAsync<Redis>> {
    let mut redis = Redis::default().with_tag(test_image_tag("BIOS_TEST_REDIS_TAG", "5.0"));
    if let Some(name) = test_container_name("redis") {
        redis = redis.with_container_name(name);
    }
    if let Some(network) = network {
        redis = redis.with_network(network);
    }
    let container = redis.start().await?;
    eprintln!("[BIOS.Test] Redis isolated container ready");
    Ok(container)
}

/// 独立的 RabbitMQ 测试初始化器：默认镜像与 `TardisTestContainer::rabbit_custom` 一致，
/// 可用 `BIOS_TEST_RABBITMQ_TAG` 覆盖（如 `3-management`）。
pub async fn rabbit_custom() -> TardisResult<ContainerAsync<TestRabbitMq>> {
    rabbit_custom_on(None).await
}

async fn rabbit_custom_on(network: Option<&str>) -> TardisResult<ContainerAsync<TestRabbitMq>> {
    let mut rabbit = TestRabbitMq::default().with_tag(test_image_tag("BIOS_TEST_RABBITMQ_TAG", "3.8.22-management"));
    if let Some(name) = test_container_name("rabbitmq") {
        rabbit = rabbit.with_container_name(name);
    }
    if let Some(network) = network {
        rabbit = rabbit.with_network(network);
    }
    let container = rabbit.start().await?;
    eprintln!("[BIOS.Test] RabbitMQ isolated container ready");
    Ok(container)
}

pub async fn postgres_custom(init_script_path: Option<&str>) -> TardisResult<ContainerAsync<Postgres>> {
    postgres_custom_on(init_script_path, None).await
}

async fn postgres_custom_on(init_script_path: Option<&str>, network: Option<&str>) -> TardisResult<ContainerAsync<Postgres>> {
    // 默认 tag `alpine`（即 `postgres:alpine`）与既有行为一致；离线环境可用
    // `BIOS_TEST_POSTGRES_TAG=16-alpine` 指定本机已有镜像，避免联网拉取。
    let mut postgres = Postgres::default()
        .with_tag(test_image_tag("BIOS_TEST_POSTGRES_TAG", "alpine"))
        .with_env_var("POSTGRES_PASSWORD", "123456")
        .with_env_var("POSTGRES_DB", "test");
    if let Some(name) = test_container_name("postgres") {
        postgres = postgres.with_container_name(name);
    }
    if let Some(network) = network {
        postgres = postgres.with_network(network);
    }
    postgres = if let Some(init_script_path) = init_script_path {
        let path = env::current_dir()
            .expect("[Tardis.Test_Container] Current path get error")
            .join(std::path::Path::new(init_script_path))
            .to_str()
            .unwrap_or_else(|| panic!("[Tardis.Test_Container] Script Path [{init_script_path}] get error"))
            .to_string();
        postgres.with_mount(Mount::volume_mount(path, "/docker-entrypoint-initdb.d/"))
    } else {
        postgres
    };
    let postgres = postgres.start().await?;
    eprintln!("[BIOS.Test] PostgreSQL isolated container ready");
    Ok(postgres)
}

pub async fn init(sql_init_path: Option<String>) -> TardisResult<LifeHold> {
    let network = if env::var_os("BIOS_TEST_DOCKER_NETWORK_ISOLATED").is_some() {
        let prefix = env::var("BIOS_TEST_CONTAINER_NAME_PREFIX").ok().filter(|prefix| !prefix.trim().is_empty()).unwrap_or_else(|| "wf-renewal-smoke-".to_string());
        Some(IsolatedNetwork::create(prefix.trim())?)
    } else {
        None
    };
    let network_name = network.as_ref().map(|network| network.name.as_str());
    let reldb_container = postgres_custom_on(sql_init_path.as_deref(), network_name).await?;
    let port = reldb_container.get_host_port_ipv4(5432).await?;
    let url = format!("postgres://postgres:123456@127.0.0.1:{port}/test");
    env::set_var("TARDIS_FW.DB.URL", url);
    env::set_var("TARDIS_FW.DB.PORT", port.to_string());

    let redis_container = redis_custom_on(network_name).await?;
    let port = redis_container.get_host_port_ipv4(6379).await?;
    let url = format!("redis://127.0.0.1:{port}/0");
    env::set_var("TARDIS_FW.CACHE.URL", url);

    // TODO remove
    let rabbit_container = rabbit_custom_on(network_name).await?;
    let port = rabbit_container.get_host_port_ipv4(5672).await?;
    let url = format!("amqp://guest:guest@127.0.0.1:{port}/%2f");
    env::set_var("TARDIS_FW.MQ.URL", url);

    TardisFuns::init(Some("tests/config")).await?;
    Ok(LifeHold {
        reldb: reldb_container,
        redis: redis_container,
        rabbit: rabbit_container,
        network,
    })
}
