use lazy_static::lazy_static;
use std::collections::HashMap;
use std::vec;
use tardis::db::sea_orm::IdenStatic;
use tardis::regex::Regex;

use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::IdResp;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::*;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::rbum::domain::{rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel_attr};
use crate::rbum::dto::rbum_filer_dto::RbumKindAttrFilterReq;
use crate::rbum::dto::rbum_filer_dto::RbumKindFilterReq;
use crate::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use crate::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindDetailResp, RbumKindModifyReq, RbumKindSummaryResp};
use crate::rbum::rbum_enumeration::RbumScopeLevelKind;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage, R_URL_PART_CODE};
use crate::rbum::serv::rbum_item_serv::{RbumItemAttrServ, RbumItemServ};
use crate::rbum::serv::rbum_rel_serv::RbumRelAttrServ;

pub struct RbumKindServ;
pub struct RbumKindAttrServ;

lazy_static! {
    pub static ref EXTRACT_R: Regex = Regex::new(r"\{(.+?)\}").expect("extract {} error");
}

#[async_trait]
impl RbumCrudOperation<rbum_kind::ActiveModel, RbumKindAddReq, RbumKindModifyReq, RbumKindSummaryResp, RbumKindDetailResp, RbumKindFilterReq> for RbumKindServ {
    fn get_table_name() -> &'static str {
        rbum_kind::Entity.table_name()
    }

    async fn package_add(add_req: &RbumKindAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_kind::ActiveModel> {
        Ok(rbum_kind::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            module: Set(add_req.module.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext_table_name: Set(add_req.ext_table_name.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumKindAddReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        if !R_URL_PART_CODE.is_match(add_req.code.as_str()) {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", &format!("code {} is invalid", add_req.code), "400-rbum-*-code-illegal"));
        }
        if funs.db().count(Query::select().column(rbum_kind::Column::Id).from(rbum_kind::Entity).and_where(Expr::col(rbum_kind::Column::Code).eq(add_req.code.as_str()))).await?
            > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add", &format!("code {} already exists", add_req.code), "409-rbum-*-code-exist"));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumKindModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_kind::ActiveModel> {
        let mut rbum_kind = rbum_kind::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &modify_req.name {
            rbum_kind.name = Set(name.to_string());
        }
        if let Some(note) = &modify_req.note {
            rbum_kind.note = Set(note.to_string());
        }
        if let Some(icon) = &modify_req.icon {
            rbum_kind.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_kind.sort = Set(sort);
        }
        if let Some(module) = &modify_req.module {
            rbum_kind.module = Set(module.to_string());
        }
        if let Some(ext_table_name) = &modify_req.ext_table_name {
            rbum_kind.ext_table_name = Set(ext_table_name.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_kind.scope_level = Set(scope_level.to_int());
        }
        Ok(rbum_kind)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumKindDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
        Self::check_exist_before_delete(id, RbumKindAttrServ::get_table_name(), rbum_kind_attr::Column::RelRbumKindId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumItemServ::get_table_name(), rbum_item::Column::RelRbumKindId.as_str(), funs).await?;
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumKindFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(vec![
            (rbum_kind::Entity, rbum_kind::Column::Id),
            (rbum_kind::Entity, rbum_kind::Column::Code),
            (rbum_kind::Entity, rbum_kind::Column::Name),
            (rbum_kind::Entity, rbum_kind::Column::Note),
            (rbum_kind::Entity, rbum_kind::Column::Icon),
            (rbum_kind::Entity, rbum_kind::Column::Sort),
            (rbum_kind::Entity, rbum_kind::Column::Module),
            (rbum_kind::Entity, rbum_kind::Column::ExtTableName),
            (rbum_kind::Entity, rbum_kind::Column::OwnPaths),
            (rbum_kind::Entity, rbum_kind::Column::Owner),
            (rbum_kind::Entity, rbum_kind::Column::CreateTime),
            (rbum_kind::Entity, rbum_kind::Column::UpdateTime),
            (rbum_kind::Entity, rbum_kind::Column::ScopeLevel),
        ]);
        if let Some(module) = &filter.module {
            query.and_where(Expr::col((rbum_kind::Entity, rbum_kind::Column::Module)).eq(module.to_string()));
        }
        query.from(rbum_kind::Entity).with_filter(Self::get_table_name(), &filter.basic, is_detail, true, ctx);
        Ok(query)
    }
}

impl RbumKindServ {
    pub async fn get_rbum_kind_id_by_code(code: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let resp = funs
            .db()
            .get_dto::<IdResp>(Query::select().column(rbum_kind::Column::Id).from(rbum_kind::Entity).and_where(Expr::col(rbum_kind::Column::Code).eq(code)))
            .await?
            .map(|r| r.id);
        Ok(resp)
    }
}

#[async_trait]
impl RbumCrudOperation<rbum_kind_attr::ActiveModel, RbumKindAttrAddReq, RbumKindAttrModifyReq, RbumKindAttrSummaryResp, RbumKindAttrDetailResp, RbumKindAttrFilterReq>
    for RbumKindAttrServ
{
    fn get_table_name() -> &'static str {
        rbum_kind_attr::Entity.table_name()
    }

    async fn package_add(add_req: &RbumKindAttrAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_kind_attr::ActiveModel> {
        Ok(rbum_kind_attr::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            name: Set(add_req.name.to_string()),
            module: Set(add_req.module.as_ref().unwrap_or(&TrimString("".to_string())).to_string()),
            label: Set(add_req.label.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            main_column: Set(add_req.main_column.unwrap_or(false)),
            position: Set(add_req.position.unwrap_or(false)),
            capacity: Set(add_req.capacity.unwrap_or(false)),
            overload: Set(add_req.overload.unwrap_or(false)),
            hide: Set(add_req.hide.unwrap_or(false)),
            secret: Set(add_req.secret.unwrap_or(false)),
            show_by_conds: Set(add_req.show_by_conds.as_ref().unwrap_or(&"".to_string()).to_string()),
            idx: Set(add_req.idx.unwrap_or(false)),
            data_type: Set(add_req.data_type.to_string()),
            widget_type: Set(add_req.widget_type.to_string()),
            widget_columns: Set(add_req.widget_columns.unwrap_or(0)),
            default_value: Set(add_req.default_value.as_ref().unwrap_or(&"".to_string()).to_string()),
            dyn_default_value: Set(add_req.dyn_default_value.as_ref().unwrap_or(&"".to_string()).to_string()),
            options: Set(add_req.options.as_ref().unwrap_or(&"".to_string()).to_string()),
            dyn_options: Set(add_req.dyn_options.as_ref().unwrap_or(&"".to_string()).to_string()),
            required: Set(add_req.required.unwrap_or(false)),
            min_length: Set(add_req.min_length.unwrap_or(0)),
            max_length: Set(add_req.max_length.unwrap_or(0)),
            action: Set(add_req.action.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            parent_attr_name: Set(add_req.parent_attr_name.as_ref().unwrap_or(&TrimString("".to_string())).to_string()),
            rel_rbum_kind_id: Set(add_req.rel_rbum_kind_id.to_string()),
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumKindAttrAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), funs, ctx).await?;
        // TODO This check does not consider scope level
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_kind_attr::Column::Id)
                    .from(rbum_kind_attr::Entity)
                    .and_where(Expr::col(rbum_kind_attr::Column::Name).eq(add_req.name.as_str()))
                    .and_where(Expr::col(rbum_kind_attr::Column::Module).eq(add_req.module.as_ref().unwrap_or(&TrimString("".to_string())).as_str()))
                    .and_where(Expr::col(rbum_kind_attr::Column::RelRbumKindId).eq(add_req.rel_rbum_kind_id.as_str()))
                    .and_where(Expr::col(rbum_kind_attr::Column::OwnPaths).like(format!("{}%", ctx.own_paths).as_str())),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add", &format!("name {} already exists", add_req.name), "409-rbum-*-name-exist"));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumKindAttrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_kind_attr::ActiveModel> {
        let mut rbum_kind_attr = rbum_kind_attr::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(label) = &modify_req.label {
            rbum_kind_attr.label = Set(label.to_string());
        }
        if let Some(note) = &modify_req.note {
            rbum_kind_attr.note = Set(note.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_kind_attr.sort = Set(sort);
        }
        if let Some(main_column) = modify_req.main_column {
            rbum_kind_attr.main_column = Set(main_column);
        }
        if let Some(position) = modify_req.position {
            rbum_kind_attr.position = Set(position);
        }
        if let Some(capacity) = modify_req.capacity {
            rbum_kind_attr.capacity = Set(capacity);
        }
        if let Some(overload) = modify_req.overload {
            rbum_kind_attr.overload = Set(overload);
        }
        if let Some(hide) = modify_req.hide {
            rbum_kind_attr.hide = Set(hide);
        }
        if let Some(secret) = modify_req.secret {
            rbum_kind_attr.secret = Set(secret);
        }
        if let Some(show_by_conds) = &modify_req.show_by_conds {
            rbum_kind_attr.show_by_conds = Set(show_by_conds.to_string());
        }
        if let Some(idx) = modify_req.idx {
            rbum_kind_attr.idx = Set(idx);
        }
        if let Some(data_type) = &modify_req.data_type {
            rbum_kind_attr.data_type = Set(data_type.to_string());
        }
        if let Some(widget_type) = &modify_req.widget_type {
            rbum_kind_attr.widget_type = Set(widget_type.to_string());
        }
        if let Some(widget_columns) = modify_req.widget_columns {
            rbum_kind_attr.widget_columns = Set(widget_columns);
        }
        if let Some(default_value) = &modify_req.default_value {
            rbum_kind_attr.default_value = Set(default_value.to_string());
        }
        if let Some(dyn_default_value) = &modify_req.dyn_default_value {
            rbum_kind_attr.dyn_default_value = Set(dyn_default_value.to_string());
        }
        if let Some(options) = &modify_req.options {
            rbum_kind_attr.options = Set(options.to_string());
        }
        if let Some(dyn_options) = &modify_req.dyn_options {
            rbum_kind_attr.dyn_options = Set(dyn_options.to_string());
        }
        if let Some(required) = modify_req.required {
            rbum_kind_attr.required = Set(required);
        }
        if let Some(min_length) = modify_req.min_length {
            rbum_kind_attr.min_length = Set(min_length);
        }
        if let Some(max_length) = modify_req.max_length {
            rbum_kind_attr.max_length = Set(max_length);
        }
        if let Some(action) = &modify_req.action {
            rbum_kind_attr.action = Set(action.to_string());
        }
        if let Some(ext) = &modify_req.ext {
            rbum_kind_attr.ext = Set(ext.to_string());
        }
        if let Some(parent_attr_name) = &modify_req.parent_attr_name {
            rbum_kind_attr.parent_attr_name = Set(parent_attr_name.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_kind_attr.scope_level = Set(scope_level.to_int());
        }
        Ok(rbum_kind_attr)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumKindAttrDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
        Self::check_exist_before_delete(id, RbumItemAttrServ::get_table_name(), rbum_item_attr::Column::RelRbumKindAttrId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumRelAttrServ::get_table_name(), rbum_rel_attr::Column::RelRbumKindAttrId.as_str(), funs).await?;
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumKindAttrFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Id),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Name),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Module),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Label),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Note),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Sort),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MainColumn),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Position),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Capacity),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Overload),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Hide),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Secret),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::ShowByConds),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Idx),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DataType),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::WidgetType),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::WidgetColumns),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DefaultValue),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DynDefaultValue),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Options),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DynOptions),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Required),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MinLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MaxLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Action),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Ext),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::ParentAttrName),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::RelRbumKindId),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::OwnPaths),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Owner),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::CreateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::ScopeLevel),
            ])
            .from(rbum_kind_attr::Entity);
        if let Some(secret) = filter.secret {
            query.and_where(Expr::col(rbum_kind_attr::Column::Secret).eq(secret));
        }
        if let Some(parent_attr_name) = &filter.parent_attr_name {
            query.and_where(Expr::col(rbum_kind_attr::Column::ParentAttrName).eq(parent_attr_name.to_string()));
        }
        if is_detail {
            query.expr_as(Expr::col((rbum_kind::Entity, rbum_kind::Column::Name)), Alias::new("rel_rbum_kind_name")).inner_join(
                rbum_kind::Entity,
                Expr::col((rbum_kind::Entity, rbum_kind::Column::Id)).equals((rbum_kind_attr::Entity, rbum_kind_attr::Column::RelRbumKindId)),
            );
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, true, ctx);
        Ok(query)
    }
}

impl RbumKindAttrServ {
    pub fn url_replace(uri: &str, values: HashMap<String, String>) -> TardisResult<String> {
        let mut new_uri = uri.to_string();
        for mat in EXTRACT_R.captures_iter(uri) {
            let old_key = mat.get(0).unwrap().as_str();
            let key = &old_key[1..old_key.len() - 1];
            if let Some(value) = values.get(key) {
                new_uri = new_uri.replace(old_key, value);
            }
        }
        Ok(new_uri)
    }

    pub fn url_match(uri: &str) -> TardisResult<bool> {
        Ok(!EXTRACT_R.is_match(uri))
    }
}
