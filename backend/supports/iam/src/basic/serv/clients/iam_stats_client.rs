use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq},
    serv::{
        rbum_crud_serv::RbumCrudOperation,
        rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ},
    },
};
use bios_sdk_invoke::{
    clients::{spi_kv_client::SpiKvClient, spi_stats_client::SpiStatsClient},
    dto::stats_record_dto::StatsFactRecordLoadReq,
};

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::json,
    tokio, TardisFunsInst,
};

use crate::{
    iam_config::{IamBasicConfigApi, IamConfig},
    iam_constants,
    iam_enumeration::IamSetKind,
};

use super::iam_kv_client::IamKvClient;

pub struct IamStatsClient;

impl IamStatsClient {
    pub async fn async_org_fact_record_load(org_cate_id: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let org_cate = RbumSetCateServ::get_rbum(
            &org_cate_id,
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let mock_ctx = TardisContext {
            own_paths: org_cate.own_paths,
            ..ctx_clone.clone()
        };
        let set = RbumSetServ::get_rbum(
            &org_cate.rel_rbum_set_id,
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        // 只允许组织 Set 继续执行
        if set.kind != IamSetKind::Org.to_string() {
            return Ok(());
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
                rel_rbum_set_cate_ids: Some(vec![org_cate_id.clone()]),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_account_id()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            &ctx_clone,
        )
        .await?
        .into_iter()
        .map(|resp| resp.rel_rbum_item_id)
        .collect();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = Self::org_fact_record_load(org_cate_id.clone(), account_ids, &funs, &mock_ctx).await;
                    let _ = IamKvClient::async_add_or_modify_key_name(funs.conf::<IamConfig>().spi.kv_orgs_prefix.clone(), org_cate_id, org_cate.name, &funs, &mock_ctx).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn org_fact_record_load(org_id: String, account_id: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let add_req = StatsFactRecordLoadReq {
            own_paths: ctx.own_paths.clone(),
            ct: tardis::chrono::Utc::now(),
            data: json!({
                "org_id": org_id,
                "account_ids": account_id,
                "account_num": account_id.len(),
            }),
            ext: None,
            idempotent_id: None,
            ignore_updates: None,
        };
        SpiStatsClient::fact_record_load(&funs.conf::<IamConfig>().spi.stats_orgs_prefix.clone(), &org_id, add_req, funs, ctx).await?;
        Ok(())
    }

    pub async fn async_org_fact_record_remove(org_id: String, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = Self::org_fact_record_remove(org_id.clone(), &funs, &ctx_clone).await;
                    let _ = SpiKvClient::delete_item(&format!("{}:{}", funs.conf::<IamConfig>().spi.kv_orgs_prefix.clone(), org_id), &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn org_fact_record_remove(org_id: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        SpiStatsClient::fact_record_delete(&funs.conf::<IamConfig>().spi.stats_orgs_prefix.clone(), &org_id, funs, ctx).await?;
        Ok(())
    }
}
