use std::{sync::Arc, time::Duration, vec};

use asteroid_mq::{
    prelude::{DurableService, Node, NodeConfig, NodeId, TopicConfig},
    protocol::node::{
        edge::auth::EdgeAuthService,
        raft::cluster::{K8sClusterProvider, StaticClusterProvider},
    },
};
use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_domain_serv::RbumDomainServ, rbum_kind_serv::RbumKindServ},
};
use bios_sdk_invoke::clients::event_client::SPI_RPC_TOPIC;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::reldb_client::TardisActiveModel,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::{
        ca::{event_connect_api, event_register_api},
        ci::event_topic_api,
    },
    domain::{event_auth, event_message, event_topic},
    event_config::{EventConfig, EventInfo, EventInfoManager},
    event_constants::{DOMAIN_CODE, KIND_CODE},
    mq_adapter::{BiosDurableAdapter, BiosEdgeAuthAdapter},
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let config = funs.conf::<EventConfig>();

    init_api(web_server).await?;
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    funs.begin().await?;
    init_db(DOMAIN_CODE.to_string(), KIND_CODE.to_string(), &funs, &ctx).await?;

    funs.commit().await?;
    if config.enable {
        let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
        init_mq_cluster(&config, funs, ctx).await?;
    }
    Ok(())
}

async fn init_db(domain_code: String, kind_code: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // Initialize event component RBUM item table and indexs
    let _ = funs.db().init(event_topic::ActiveModel::init(TardisFuns::reldb().backend(), None, TardisFuns::reldb().compatible_type())).await;
    let _ = funs.db().init(event_message::ActiveModel::init(TardisFuns::reldb().backend(), None, TardisFuns::reldb().compatible_type())).await;
    let _ = funs.db().init(event_auth::ActiveModel::init(TardisFuns::reldb().backend(), None, TardisFuns::reldb().compatible_type())).await;
    if let Some(domain_id) = RbumDomainServ::get_rbum_domain_id_by_code(&domain_code, funs).await? {
        let kind_id = RbumKindServ::get_rbum_kind_id_by_code(&kind_code, funs).await?.expect("missing event kind");
        EventInfoManager::set(EventInfo { kind_id, domain_id })?;
        return Ok(());
    }

    // funs.db()
    //     .init(event_persistent::ActiveModel::init(
    //         TardisFuns::reldb().backend(),
    //         None,
    //         TardisFuns::reldb().compatible_type(),
    //     ))
    //     .await?;
    // Initialize event component RBUM domain data
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(domain_code.to_string()),
            name: TrimString(domain_code.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await?;
    // Initialize event component RBUM kind data
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(kind_code.to_string()),
            name: TrimString(kind_code.to_string()),
            note: None,
            icon: None,
            sort: None,
            module: None,
            ext_table_name: Some("mq_topic".to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await?;
    EventInfoManager::set(EventInfo { kind_id, domain_id })?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            DOMAIN_CODE,
            (
                event_topic_api::EventTopicApi,
                event_connect_api::EventConnectApi::default(),
                event_register_api::EventRegisterApi::default(),
            ),
        )
        .await;
    Ok(())
}

async fn init_mq_cluster(config: &EventConfig, funs: TardisFunsInst, ctx: TardisContext) -> TardisResult<()> {
    use bios_sdk_invoke::clients::event_client::mq_error;
    let mq_node = init_mq_node(config, funs, ctx).await;
    mq_node.load_from_durable_service().await.map_err(mq_error)?;
    // it's important to ensure the SPI_RPC_TOPIC is created, many other components depend on it
    if mq_node.get_topic(&SPI_RPC_TOPIC).is_none() {
        mq_node
            .load_topic(
                TopicConfig {
                    code: SPI_RPC_TOPIC,
                    overflow_config: None,
                    blocking: false,
                },
                vec![],
            )
            .await
            .map_err(mq_error)?;
    }
    Ok(())
}

pub async fn init_mq_node(config: &EventConfig, funs: TardisFunsInst, ctx: TardisContext) -> asteroid_mq::prelude::Node {
    let timeout = Duration::from_secs(config.startup_timeout);
    const ENV_POD_UID: &str = "POD_UID";
    if let Some(node) = TardisFuns::store().get_singleton::<asteroid_mq::prelude::Node>() {
        node
    } else {
        let funs = Arc::new(funs);
        let node = match config.cluster.as_deref() {
            Some(EventConfig::CLUSTER_K8S) => {
                let uid = std::env::var(ENV_POD_UID).expect("POD_UID is required");
                let node = Node::new(NodeConfig {
                    id: NodeId::sha256(uid.as_bytes()),
                    raft: config.raft.clone(),
                    durable: config.durable.then_some(DurableService::new(BiosDurableAdapter::new(funs.clone(), ctx.clone()))),
                    edge_auth: Some(EdgeAuthService::new(BiosEdgeAuthAdapter::new(funs.clone(), ctx))),
                    ..Default::default()
                });
                let cluster_provider = K8sClusterProvider::new(config.svc.clone(), asteroid_mq::DEFAULT_TCP_PORT).await;
                node.init_raft(cluster_provider).await.expect("fail to init raft");
                node
            }
            Some(EventConfig::NO_CLUSTER) | None => {
                let node = Node::new(NodeConfig {
                    id: NodeId::snowflake(),
                    raft: config.raft.clone(),
                    durable: config.durable.then_some(DurableService::new(BiosDurableAdapter::new(funs.clone(), ctx.clone()))),
                    edge_auth: Some(EdgeAuthService::new(BiosEdgeAuthAdapter::new(funs.clone(), ctx))),
                    ..Default::default()
                });
                // singleton mode
                let cluster_provider = StaticClusterProvider::singleton(node.config());
                node.init_raft(cluster_provider).await.expect("fail to init raft");
                node
            }
            Some(unknown_cluster) => {
                panic!("unknown cluster provider {unknown_cluster}")
            }
        };
        node.raft().await.wait(Some(timeout)).metrics(|rm| rm.state.is_leader() || rm.state.is_follower(), "raft ready").await.expect("fail to wait raft ready");
        TardisFuns::store().insert_singleton(node.clone());
        node
    }
}
