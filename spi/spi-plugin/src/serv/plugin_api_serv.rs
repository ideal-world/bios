use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, CODE_FIELD, CREATE_TIME_FIELD};
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Alias, Expr, SelectStatement};
use tardis::db::sea_orm::{EntityName, Set};
use tardis::web::poem_openapi::types::Type;
use tardis::TardisFunsInst;

use crate::domain::plugin_api;
use crate::dto::plugin_api_dto::{PluginApiAddOrModifyReq, PluginApiDetailResp, PluginApiFilterReq, PluginApiSummaryResp};

pub struct PluginApiServ;

#[async_trait]
impl RbumItemCrudOperation<plugin_api::ActiveModel, PluginApiAddOrModifyReq, PluginApiAddOrModifyReq, PluginApiSummaryResp, PluginApiDetailResp, PluginApiFilterReq>
    for PluginApiServ
{
    fn get_ext_table_name() -> &'static str {
        plugin_api::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        None
    }

    fn get_rbum_domain_id() -> Option<String> {
        None
    }

    async fn package_item_add(add_req: &PluginApiAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        if Self::count_items(
            &PluginApiFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    code: Some(add_req.code.to_string()),
                    rbum_kind_id: Some(add_req.kind_id.to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            > 1
        {
            Err(funs.err().conflict(&Self::get_obj_name(), "add_api", "api code is exist", "404-spi-*-obj-is-exist"))?;
        }
        let domain_id = RbumDomainServ::get_rbum_domain_id_by_code(funs.module_code(), funs)
            .await?
            .ok_or_else(|| funs.err().not_found(&Self::get_obj_name(), "add_api", "not found domain", "404-spi-*-obj-not-exist"))?;
        Ok(RbumItemKernelAddReq {
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            rel_rbum_kind_id: Some(add_req.kind_id.to_string()),
            rel_rbum_domain_id: Some(domain_id),
            scope_level: Some(RbumScopeLevelKind::Root),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &PluginApiAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<plugin_api::ActiveModel> {
        Ok(plugin_api::ActiveModel {
            id: Set(id.to_string()),
            callback: Set(add_req.callback.clone()),
            content_type: Set(add_req.content_type.clone()),
            ext: Set(add_req.ext.clone()),
            timeout: Set(add_req.timeout),
            http_method: Set(add_req.http_method.to_string()),
            kind: Set(add_req.kind.clone()),
            path_and_query: Set(add_req.path_and_query.clone()),
            save_message: Set(add_req.save_message),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &PluginApiAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: Some(modify_req.name.clone()),
            scope_level: Some(RbumScopeLevelKind::Root),
            disabled: None,
        }))
    }

    async fn package_ext_modify(_: &str, modify_req: &PluginApiAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<plugin_api::ActiveModel>> {
        let plugin_api = plugin_api::ActiveModel {
            callback: Set(modify_req.callback.clone()),
            content_type: Set(modify_req.content_type.clone()),
            timeout: Set(modify_req.timeout),
            ext: Set(modify_req.ext.clone()),
            http_method: Set(modify_req.http_method.to_string()),
            kind: Set(modify_req.kind.clone()),
            path_and_query: Set(modify_req.path_and_query.clone()),
            save_message: Set(modify_req.save_message),
            ..Default::default()
        };
        Ok(Some(plugin_api))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &PluginApiFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query
            .column((plugin_api::Entity, plugin_api::Column::Kind))
            .column((plugin_api::Entity, plugin_api::Column::PathAndQuery))
            .column((plugin_api::Entity, plugin_api::Column::ContentType))
            .column((plugin_api::Entity, plugin_api::Column::Timeout))
            .column((plugin_api::Entity, plugin_api::Column::Callback))
            .column((plugin_api::Entity, plugin_api::Column::HttpMethod))
            .column((plugin_api::Entity, plugin_api::Column::Ext))
            .column((plugin_api::Entity, plugin_api::Column::SaveMessage));
        if let Some(path_and_query) = &filter.path_and_query {
            query.and_where(Expr::col(plugin_api::Column::PathAndQuery).like(format!("%{path_and_query}%").as_str()));
        }
        if let Some(code) = &filter.code {
            query.and_where(Expr::col((Alias::new(RbumItemServ::get_table_name()), CODE_FIELD.clone())).eq(code.as_str()));
        }
        if let Some(create_start) = &filter.create_start {
            query.and_where(Expr::col((Alias::new(RbumItemServ::get_table_name()), CREATE_TIME_FIELD.clone())).gte(*create_start));
        }
        if let Some(create_end) = &filter.create_end {
            query.and_where(Expr::col((Alias::new(RbumItemServ::get_table_name()), CREATE_TIME_FIELD.clone())).lte(*create_end));
        }
        Ok(())
    }
}

impl PluginApiServ {
    pub async fn add_or_modify_item(add_modify_req: &mut PluginApiAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = Self::get_id_by_code(&add_modify_req.code, funs, ctx).await?;
        if id.is_none() {
            Self::add_item(add_modify_req, funs, ctx).await?;
        } else {
            Self::modify_item(id.unwrap().as_str(), add_modify_req, funs, ctx).await?;
        }
        Ok(add_modify_req.code.to_string())
    }

    pub async fn delete_by_code(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let id = Self::get_id_by_code(code, funs, ctx).await?;
        if id.is_none() {
            Err(funs.err().not_found(&Self::get_obj_name(), "delete", "", ""))
        } else {
            Self::delete_item(id.unwrap().as_str(), funs, ctx).await
        }
    }

    pub async fn get_by_code(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<PluginApiDetailResp>> {
        let resp = Self::find_one_detail_item(
            &PluginApiFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                code: Some(code.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(resp)
    }

    pub async fn get_id_by_code(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let resp = Self::find_one_detail_item(
            &PluginApiFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(code.to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .map(|r| r.id);
        Ok(resp)
    }
}
