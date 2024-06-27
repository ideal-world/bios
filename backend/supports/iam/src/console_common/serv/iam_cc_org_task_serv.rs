use crate::{
    basic::serv::clients::iam_stats_client::IamStatsClient,
    iam_config::IamConfig,
    iam_constants,
    iam_enumeration::IamSetKind,
    iam_initializer::{default_iam_send_avatar, ws_iam_send_client},
};
use bios_basic::{
    process::task_processor::TaskProcessor,
    rbum::{
        dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq, RbumSetFilterReq},
        serv::{
            rbum_crud_serv::RbumCrudOperation,
            rbum_set_serv::{RbumSetCateServ, RbumSetServ},
        },
    },
};

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

pub struct IamCcOrgTaskServ;

impl IamCcOrgTaskServ {
    pub async fn execute_org_task(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let task_ctx = ctx.clone();
        TaskProcessor::execute_task_with_ctx(
            &funs.conf::<IamConfig>().cache_key_async_task_status,
            move |_task_id| async move {
                let mut funs = iam_constants::get_tardis_inst();
                funs.begin().await?;
                let base_org_set_ids = RbumSetServ::find_id_rbums(
                    &RbumSetFilterReq {
                        basic: RbumBasicFilterReq {
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        kind: Some(IamSetKind::Org.to_string()),
                        ..Default::default()
                    },
                    None,
                    None,
                    &funs,
                    &task_ctx,
                )
                .await?;
                for org_set_id in base_org_set_ids {
                    let base_org_set_cate_ids = RbumSetCateServ::find_id_rbums(
                        &RbumSetCateFilterReq {
                            basic: RbumBasicFilterReq {
                                own_paths: Some("".to_string()),
                                with_sub_own_paths: true,
                                ..Default::default()
                            },
                            rel_rbum_set_id: Some(org_set_id.clone()),
                            ..Default::default()
                        },
                        None,
                        None,
                        &funs,
                        &task_ctx,
                    )
                    .await?;
                    let mut num = 0;
                    for org_set_cate_id in base_org_set_cate_ids {
                        num += 1;
                        if num % 100 == 0 {
                            tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                        IamStatsClient::async_org_fact_record_load(org_set_cate_id, &funs, &task_ctx).await?;
                    }
                }
                funs.commit().await?;
                task_ctx.execute_task().await?;
                Ok(())
            },
            &funs.cache(),
            ws_iam_send_client().await.clone(),
            default_iam_send_avatar().await.clone(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(None)
    }
}
