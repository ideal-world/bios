use serde::Deserialize;
use spacegate_plugins::register_lib_plugins;
use spacegate_shell::plugin::PluginRepository;
use spacegate_shell::BoxError;
use tardis::basic::tracing::TardisTracing;
use tardis::tokio;

#[derive(Deserialize, Debug)]
struct Config {
    tokio_worker_thread: Option<usize>,
    tokio_event_interval: Option<u32>,
    spacegate_ns: Option<String>,
    config_file: Option<String>,
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
    register_lib_plugins(PluginRepository::global());
    rt.block_on(async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                if let Some(config_file) = config.config_file {
                    let config_file_content = tokio::fs::read_to_string(config_file.clone()).await?;
                    let config = toml::from_str::<spacegate_shell::model::Config>(&config_file_content)?;
                    spacegate_shell::startup_static(config).await
                } else {
                    spacegate_shell::startup_k8s(namespaces.as_deref()).await
                }
            })
            .await
    })
}
