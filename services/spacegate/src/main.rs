use serde::Deserialize;
use spacegate_lib::register_lib_plugins;
use spacegate_shell::plugin::SgPluginRepository;
use spacegate_shell::BoxError;
use tardis::basic::tracing::TardisTracing;
use tardis::tokio;

#[derive(Deserialize, Debug)]
struct Config {
    tokio_worker_thread: Option<usize>,
    tokio_event_interval: Option<u32>,
    spacegate_ns: Option<String>,
}

fn main() -> Result<(), BoxError> {
    let config = envy::from_env::<Config>()?;
    TardisTracing::initializer().with_env_layer().with_fmt_layer().init();
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all().thread_name("spacegate-bios");
    if let Some(tokio_worker_thread) = config.tokio_worker_thread {
        builder.worker_threads(tokio_worker_thread);
    }
    if let Some(tokio_event_interval) = config.tokio_event_interval {
        builder.event_interval(tokio_event_interval);
    }
    let rt = builder.build().expect("fail to build runtime");
    let namespaces = std::env::args().nth(1).or(config.spacegate_ns);
    register_lib_plugins(SgPluginRepository::global());
    rt.block_on(async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                let join_handle = spacegate_shell::startup_k8s(namespaces.as_deref()).await.expect("fail to start spacegate");
                join_handle.await.expect("join handle error")
            })
            .await
    })
}
