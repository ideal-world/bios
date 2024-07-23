use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_domain_serv::RbumDomainServ, rbum_item_serv::RbumItemCrudOperation, rbum_kind_serv::RbumKindServ},
};
use bios_sdk_invoke::clients::event_client::{BiosEventCenter, EventCenter, EventCenterConfig, TOPIC_BIOS_PUB_SUB, TOPIC_BIOS_WORKER_QUEUE};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::reldb_client::TardisActiveModel,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::{event_listener_api, event_proc_api, event_topic_api},
    domain::{event_persistent, event_topic},
    dto::event_dto::EventTopicAddOrModifyReq,
    event_config::{EventConfig, EventInfo, EventInfoManager},
    event_constants::{DOMAIN_CODE, KIND_CODE},
    serv::{self, event_proc_serv::CreateRemoteSenderHandler, event_topic_serv::EventDefServ},
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let config = funs.conf::<EventConfig>();
    if !config.enable {
        return Ok(());
    }
    create_event_center()?;
    init_api(web_server).await?;
    init_cluster_resource().await;
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
    EventDefServ::init(&funs, &ctx).await?;
    init_topic(&funs, &ctx).await?;
    funs.commit().await?;
    init_scan_and_resend_task();
    Ok(())
}

async fn init_topic(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let config = funs.conf::<EventConfig>();
    // create bios worker queue topic
    let _result = serv::event_topic_serv::EventDefServ::add_item(
        &mut EventTopicAddOrModifyReq {
            code: TOPIC_BIOS_WORKER_QUEUE.into(),
            name: TOPIC_BIOS_WORKER_QUEUE.into(),
            save_message: false,
            need_mgr: false,
            queue_size: 1024,
            use_sk: Some(config.event_bus_sk.clone()),
            mgr_sk: None,
        },
        funs,
        ctx,
    )
    .await;
    // create bios pub sub topic
    let _result = serv::event_topic_serv::EventDefServ::add_item(
        &mut EventTopicAddOrModifyReq {
            code: TOPIC_BIOS_PUB_SUB.into(),
            name: TOPIC_BIOS_PUB_SUB.into(),
            save_message: false,
            need_mgr: false,
            queue_size: 1024,
            use_sk: Some(config.event_bus_sk.clone()),
            mgr_sk: None,
        },
        funs,
        ctx,
    )
    .await;
    Ok(())
}

async fn init_db(domain_code: String, kind_code: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    if let Some(domain_id) = RbumDomainServ::get_rbum_domain_id_by_code(&domain_code, funs).await? {
        let kind_id = RbumKindServ::get_rbum_kind_id_by_code(&kind_code, funs).await?.expect("missing event kind");
        EventInfoManager::set(EventInfo { kind_id, domain_id })?;
        return Ok(());
    }

    funs.db()
        .init(event_persistent::ActiveModel::init(
            TardisFuns::reldb().backend(),
            None,
            TardisFuns::reldb().compatible_type(),
        ))
        .await?;
    // Initialize event component RBUM item table and indexs
    funs.db().init(event_topic::ActiveModel::init(TardisFuns::reldb().backend(), None, TardisFuns::reldb().compatible_type())).await?;
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
            ext_table_name: Some("event_topic".to_lowercase()),
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
            (event_topic_api::EventTopicApi, event_proc_api::EventProcApi, event_listener_api::EventListenerApi),
        )
        .await;
    Ok(())
}

async fn init_cluster_resource() {
    use crate::serv::event_listener_serv::{listeners, mgr_listeners};
    use crate::serv::event_topic_serv::topics;
    use tardis::cluster::cluster_processor::subscribe;
    subscribe(listeners().clone()).await;
    subscribe(mgr_listeners().clone()).await;
    subscribe(topics().clone()).await;
    subscribe(CreateRemoteSenderHandler).await;
}

fn init_scan_and_resend_task() {
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);

    let config = funs.conf::<EventConfig>();
    let Some(interval_sec) = config.resend_interval_sec else {
        return;
    };
    let mut interval = tardis::tokio::time::interval(tardis::tokio::time::Duration::from_secs(interval_sec as u64));
    tardis::tokio::spawn(async move {
        loop {
            interval.tick().await;
            let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
            let _ = crate::serv::event_proc_serv::scan_and_resend(funs.into()).await;
        }
    });
}

fn create_event_center() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<EventConfig>(DOMAIN_CODE);
    let pubsub_config = EventCenterConfig {
        base_url: config.base_url.clone(),
        topic_sk: config.event_bus_sk.clone(),
        topic_code: TOPIC_BIOS_PUB_SUB.to_owned(),
        subscribe: true,
        avatars: config.avatars.clone(),
    };
    let pubsub = BiosEventCenter::from_config(pubsub_config);
    pubsub.init()?;
    pubsub.set_as_worker_queue();

    let wq_config = EventCenterConfig {
        base_url: config.base_url.clone(),
        topic_sk: config.event_bus_sk.clone(),
        topic_code: TOPIC_BIOS_WORKER_QUEUE.to_owned(),
        subscribe: false,
        avatars: config.avatars.clone(),
    };
    let wq = BiosEventCenter::from_config(wq_config);
    wq.init()?;
    wq.set_as_worker_queue();
    Ok(())
}
