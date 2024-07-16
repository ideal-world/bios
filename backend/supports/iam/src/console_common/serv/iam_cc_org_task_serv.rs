use crate::{
    basic::serv::clients::iam_stats_client::IamStatsClient,
    iam_config::{IamBasicConfigApi, IamConfig},
    iam_constants,
    iam_enumeration::IamSetKind,
};
use bios_basic::{
    process::task_processor::TaskProcessor,
    rbum::{
        dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq},
        serv::{
            rbum_crud_serv::RbumCrudOperation,
            rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ},
        },
    },
};

use bios_sdk_invoke::clients::spi_kv_client::SpiKvClient;
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
                    let base_org_set_cates = RbumSetCateServ::find_rbums(
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
                    for org_set_cate in base_org_set_cates {
                        let mock_ctx = TardisContext {
                            own_paths: org_set_cate.own_paths,
                            ..task_ctx.clone()
                        };
                        num += 1;
                        if num % 50 == 0 {
                            tardis::tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        }
                        let account_ids = RbumSetItemServ::find_rbums(
                            &RbumSetItemFilterReq {
                                basic: RbumBasicFilterReq {
                                    with_sub_own_paths: true,
                                    own_paths: Some("".to_owned()),
                                    ..Default::default()
                                },
                                rel_rbum_item_disabled: Some(false),
                                // rel_rbum_set_id: Some(org_set_id),
                                rel_rbum_set_cate_ids: Some(vec![org_set_cate.id.clone()]),
                                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_account_id()]),
                                ..Default::default()
                            },
                            None,
                            None,
                            &funs,
                            &task_ctx,
                        )
                        .await?
                        .into_iter()
                        .map(|resp| resp.rel_rbum_item_id)
                        .collect();
                        IamStatsClient::org_fact_record_load(org_set_cate.id.clone(), account_ids, &funs, &mock_ctx).await?;
                        SpiKvClient::add_or_modify_key_name(
                            &format!("{}:{}", funs.conf::<IamConfig>().spi.kv_orgs_prefix.clone(), org_set_cate.id),
                            &org_set_cate.name,
                            &funs,
                            &mock_ctx,
                        )
                        .await?;
                    }
                }
                funs.commit().await?;
                task_ctx.execute_task().await?;
                Ok(())
            },
            &funs.cache(),
            default_iam_send_avatar().await.clone(),
            Some(vec![format!("account/{}", ctx.owner)]),
            ctx,
        )
        .await?;
        Ok(None)
    }
}
