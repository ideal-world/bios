use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_domain_serv::RbumDomainServ, rbum_kind_serv::RbumKindServ},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::TardisActiveModel,
    web::web_server::TardisWebServer,
    TardisFuns,
};

use crate::{
    api,
    config::ReachConfig,
    consts::{get_tardis_inst, DOMAIN_CODE, DOMAIN_REACH_ID, RBUM_EXT_TABLE_REACH_MESSAGE, RBUM_KIND_CODE_REACH_MESSAGE},
    serv::ReachTriggerSceneService,
};

pub async fn init_db() -> TardisResult<()> {
    let mut funs = get_tardis_inst();
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<ReachConfig>().rbum.clone()).await?;
    funs.begin().await?;
    let ctx = TardisContext {
        owner: "ReachInit".to_string(),
        ..Default::default()
    };
    // add kind code
    RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: RBUM_KIND_CODE_REACH_MESSAGE.into(),
            name: RBUM_KIND_CODE_REACH_MESSAGE.into(),
            note: None,
            icon: None,
            sort: None,
            module: None,
            ext_table_name: Some(RBUM_EXT_TABLE_REACH_MESSAGE.to_owned()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;
    // add domain
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: DOMAIN_CODE.into(),
            name: DOMAIN_CODE.into(),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;
    DOMAIN_REACH_ID.set(domain_id).expect("fail to set DOMAIN_REACH_ID");
    let db_kind = TardisFuns::reldb().backend();
    let compatible_type = TardisFuns::reldb().compatible_type();
    funs.db().init(crate::domain::message_log::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
    funs.db().init(crate::domain::message_signature::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
    funs.db().init(crate::domain::message_template::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
    funs.db().init(crate::domain::message::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
    funs.db().init(crate::domain::trigger_global_config::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
    funs.db().init(crate::domain::trigger_instance_config::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
    funs.db().init(crate::domain::trigger_scene::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
    ReachTriggerSceneService::init(&funs, &ctx).await?;
    funs.commit().await?;
    Ok(())
}

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    init_db().await?;
    api::init(web_server).await?;
    Ok(())
}
