use async_trait::async_trait;
use bios_basic::rbum::{
    dto::rbum_filer_dto::RbumBasicFilterReq,
    serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{
        reldb_client::IdResp,
        sea_orm::{sea_query::*, EntityName, Set},
    },
    TardisFuns, TardisFunsInst,
};

use crate::{
    basic::{
        domain::iam_config,
        dto::{
            iam_config_dto::{IamConfigAddReq, IamConfigAggOrModifyReq, IamConfigDetailResp, IamConfigModifyReq, IamConfigSummaryResp},
            iam_filer_dto::IamConfigFilterReq,
        },
    },
    iam_enumeration::IamConfigKind,
};

use super::clients::spi_log_client::{LogParamTag, SpiLogClient};

pub struct IamConfigServ;

#[async_trait]
impl RbumCrudOperation<iam_config::ActiveModel, IamConfigAddReq, IamConfigModifyReq, IamConfigSummaryResp, IamConfigDetailResp, IamConfigFilterReq> for IamConfigServ {
    fn get_table_name() -> &'static str {
        iam_config::Entity.table_name()
    }

    async fn package_add(add_req: &IamConfigAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_config::ActiveModel> {
        Ok(iam_config::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.as_ref().unwrap_or(&"".to_string()).to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            value1: Set(add_req.value1.as_ref().unwrap_or(&"".to_string()).to_string()),
            value2: Set(add_req.value2.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_item_id: Set(add_req.rel_item_id.to_string()),
            disabled: Set(add_req.disabled.unwrap_or(false)),
            data_type: Set(add_req.data_type.to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut IamConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if Self::config_exist(&add_req.code, &add_req.rel_item_id, funs, ctx).await? {
            return Err(funs.err().conflict(
                Self::get_table_name(),
                "add",
                &format!("{}.{} config already exists", add_req.code, add_req.rel_item_id),
                "409-iam-config-exist",
            ));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &IamConfigModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_config::ActiveModel> {
        let mut iam_config = iam_config::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &modify_req.name {
            iam_config.name = Set(name.to_string());
        }
        if let Some(data_type) = &modify_req.data_type {
            iam_config.data_type = Set(data_type.to_string());
        }
        if let Some(note) = &modify_req.note {
            iam_config.note = Set(note.to_string());
        }
        if let Some(value1) = &modify_req.value1 {
            iam_config.value1 = Set(value1.to_string());
        }
        if let Some(value2) = &modify_req.value2 {
            iam_config.value2 = Set(value2.to_string());
        }
        if let Some(ext) = &modify_req.ext {
            iam_config.ext = Set(ext.to_string());
        }
        if let Some(disabled) = &modify_req.disabled {
            iam_config.disabled = Set(*disabled);
        }
        Ok(iam_config)
    }

    async fn package_query(is_detail: bool, filter: &IamConfigFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (iam_config::Entity, iam_config::Column::Id),
                (iam_config::Entity, iam_config::Column::Code),
                (iam_config::Entity, iam_config::Column::Name),
                (iam_config::Entity, iam_config::Column::Note),
                (iam_config::Entity, iam_config::Column::Value1),
                (iam_config::Entity, iam_config::Column::Value2),
                (iam_config::Entity, iam_config::Column::Ext),
                (iam_config::Entity, iam_config::Column::Disabled),
                (iam_config::Entity, iam_config::Column::DataType),
                (iam_config::Entity, iam_config::Column::RelItemId),
                (iam_config::Entity, iam_config::Column::OwnPaths),
                (iam_config::Entity, iam_config::Column::Owner),
                (iam_config::Entity, iam_config::Column::CreateTime),
                (iam_config::Entity, iam_config::Column::UpdateTime),
            ])
            .from(iam_config::Entity);
        if let Some(code) = &filter.code {
            query.and_where(Expr::col(iam_config::Column::Code).eq(code));
        }
        if let Some(rel_item_id) = &filter.rel_item_id {
            query.and_where(Expr::col(iam_config::Column::RelItemId).eq(rel_item_id));
        }
        if let Some(disabled) = &filter.disabled {
            query.and_where(Expr::col(iam_config::Column::Disabled).eq(*disabled));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl IamConfigServ {
    pub async fn add_or_modify_batch(rel_item_id: &str, reqs: Vec<IamConfigAggOrModifyReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for config in reqs {
            let config_id = Self::get_config_id_by_code_and_item_id(&config.code, rel_item_id, funs).await?;
            let (op_describe, op_kind) = match &config.code {
                IamConfigKind::AccountInactivityLock => (
                    format!("设置{}个月未使用的账号进行锁定或将其转为休眠账号", config.value1.clone().unwrap_or(0.to_string())),
                    "SetUpUnusedAccountsToLock".to_string(),
                ),
                IamConfigKind::TokenExpire => (
                    format!("设置{}分钟不活动则会话失效", config.value1.clone().unwrap_or(0.to_string())),
                    "SetUpSessionInvalidation".to_string(),
                ),
                IamConfigKind::AccountTemporaryExpire => (
                    format!("设置临时账号使用期限为{}个月", config.value1.clone().unwrap_or(0.to_string())),
                    "SetUpUnusedAccountsToLock".to_string(),
                ),
                _ => ("".to_string(), "".to_string()),
            };
            if !op_describe.is_empty() {
                let _ = SpiLogClient::add_ctx_task(LogParamTag::SecurityAlarm, None, op_describe, Some(op_kind), ctx).await;
            }
            if let Some(id) = config_id {
                Self::modify_rbum(
                    &id,
                    &mut IamConfigModifyReq {
                        name: config.name,
                        data_type: Some(config.data_type),
                        note: config.note,
                        value1: config.value1,
                        value2: config.value2,
                        ext: config.ext,
                        disabled: config.disabled,
                    },
                    funs,
                    ctx,
                )
                .await?;
            } else {
                Self::add_rbum(
                    &mut IamConfigAddReq {
                        code: config.code,
                        name: config.name,
                        data_type: config.data_type,
                        note: config.note,
                        value1: config.value1,
                        value2: config.value2,
                        ext: config.ext,
                        disabled: config.disabled,
                        rel_item_id: rel_item_id.to_string(),
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        Ok(())
    }

    pub async fn get_config_by_code_and_item_id(code: &IamConfigKind, rel_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamConfigSummaryResp>> {
        if let Some(config_id) = Self::get_config_id_by_code_and_item_id(code, rel_item_id, funs).await? {
            let resp = Self::peek_rbum(
                &config_id,
                &IamConfigFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            Ok(Some(resp))
        } else {
            Ok(None)
        }
    }

    pub async fn get_config_id_by_code_and_item_id(code: &IamConfigKind, rel_item_id: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let resp = funs
            .db()
            .get_dto::<IdResp>(
                Query::select()
                    .column(iam_config::Column::Id)
                    .from(iam_config::Entity)
                    .and_where(Expr::col(iam_config::Column::Code).eq(code.to_string()))
                    .and_where(Expr::col(iam_config::Column::RelItemId).eq(rel_item_id)),
            )
            .await?
            .map(|r| r.id);
        Ok(resp)
    }

    pub async fn config_exist(code: &IamConfigKind, rel_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        Self::exist_rbum(
            &IamConfigFilterReq {
                code: Some(code.to_string()),
                rel_item_id: Some(rel_item_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }
}
