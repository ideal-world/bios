use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::IdResp;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::tokio::count;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel_attr};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use crate::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindDetailResp, RbumKindModifyReq, RbumKindSummaryResp};
use crate::rbum::rbum_constants::RBUM_KIND_ID_LEN;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};

pub struct RbumKindServ;
pub struct RbumKindAttrServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_kind::ActiveModel, RbumKindAddReq, RbumKindModifyReq, RbumKindSummaryResp, RbumKindDetailResp> for RbumKindServ {
    fn get_table_name() -> &'static str {
        rbum_kind::Entity.table_name()
    }

    async fn package_add(add_req: &RbumKindAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_kind::ActiveModel> {
        Ok(rbum_kind::ActiveModel {
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ext_table_name: Set(add_req.ext_table_name.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_level: Set(add_req.scope_level.to_int()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumKindAddReq, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        if funs.db().count(Query::select().column(rbum_kind::Column::Id).from(rbum_kind::Entity).and_where(Expr::col(rbum_kind::Column::Code).eq(add_req.code.0.as_str()))).await?
            > 0
        {
            return Err(TardisError::BadRequest(format!("code {} already exists", add_req.code)));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumKindModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_kind::ActiveModel> {
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
        if let Some(ext_table_name) = &modify_req.ext_table_name {
            rbum_kind.ext_table_name = Set(ext_table_name.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_kind.scope_level = Set(scope_level.to_int());
        }
        Ok(rbum_kind)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await?;
        if funs
            .db()
            .count(Query::select().column(rbum_kind_attr::Column::Id).from(rbum_kind_attr::Entity).and_where(Expr::col(rbum_kind_attr::Column::RelRbumKindId).eq(id)))
            .await?
            > 0
        {
            return Err(TardisError::BadRequest("can not delete rbum kind when there are rbum kind attr".to_string()));
        }
        if funs.db().count(Query::select().column(rbum_item::Column::Id).from(rbum_item::Entity).and_where(Expr::col(rbum_item::Column::RelRbumKindId).eq(id))).await? > 0 {
            return Err(TardisError::BadRequest("can not delete rbum kind when there are rbum item".to_string()));
        }
        Ok(())
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(vec![
            (rbum_kind::Entity, rbum_kind::Column::Id),
            (rbum_kind::Entity, rbum_kind::Column::Code),
            (rbum_kind::Entity, rbum_kind::Column::Name),
            (rbum_kind::Entity, rbum_kind::Column::Note),
            (rbum_kind::Entity, rbum_kind::Column::Icon),
            (rbum_kind::Entity, rbum_kind::Column::Sort),
            (rbum_kind::Entity, rbum_kind::Column::ExtTableName),
            (rbum_kind::Entity, rbum_kind::Column::OwnPaths),
            (rbum_kind::Entity, rbum_kind::Column::Owner),
            (rbum_kind::Entity, rbum_kind::Column::CreateTime),
            (rbum_kind::Entity, rbum_kind::Column::UpdateTime),
            (rbum_kind::Entity, rbum_kind::Column::ScopeLevel),
        ]);
        query.from(rbum_kind::Entity).with_filter(Self::get_table_name(), filter, !is_detail, false, cxt);
        Ok(query)
    }
}

impl<'a> RbumKindServ {
    pub async fn get_rbum_kind_id_by_code(code: &str, funs: &TardisFunsInst<'a>) -> TardisResult<Option<String>> {
        let resp = funs
            .db()
            .get_dto::<IdResp>(Query::select().column(rbum_kind::Column::Id).from(rbum_kind::Entity).and_where(Expr::col(rbum_kind::Column::Code).eq(code)))
            .await?
            .map(|r| r.id);
        Ok(resp)
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_kind_attr::ActiveModel, RbumKindAttrAddReq, RbumKindAttrModifyReq, RbumKindAttrSummaryResp, RbumKindAttrDetailResp> for RbumKindAttrServ {
    fn get_table_name() -> &'static str {
        rbum_kind_attr::Entity.table_name()
    }

    async fn package_add(add_req: &RbumKindAttrAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_kind_attr::ActiveModel> {
        Ok(rbum_kind_attr::ActiveModel {
            name: Set(add_req.name.to_string()),
            label: Set(add_req.label.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            main_column: Set(add_req.main_column.unwrap_or(false)),
            position: Set(add_req.position.unwrap_or(false)),
            capacity: Set(add_req.capacity.unwrap_or(false)),
            overload: Set(add_req.overload.unwrap_or(false)),
            data_type: Set(add_req.data_type.to_string()),
            widget_type: Set(add_req.widget_type.to_string()),
            default_value: Set(add_req.default_value.as_ref().unwrap_or(&"".to_string()).to_string()),
            options: Set(add_req.options.as_ref().unwrap_or(&"".to_string()).to_string()),
            required: Set(add_req.required.unwrap_or(false)),
            min_length: Set(add_req.min_length.unwrap_or(0)),
            max_length: Set(add_req.max_length.unwrap_or(0)),
            action: Set(add_req.action.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_kind_id: Set(add_req.rel_rbum_kind_id.to_string()),
            scope_level: Set(add_req.scope_level.to_int()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumKindAttrAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), funs, cxt).await
    }

    async fn package_modify(id: &str, modify_req: &RbumKindAttrModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_kind_attr::ActiveModel> {
        let mut rbum_kind_attr = rbum_kind_attr::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &modify_req.name {
            rbum_kind_attr.name = Set(name.to_string());
        }
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
        if let Some(data_type) = &modify_req.data_type {
            rbum_kind_attr.data_type = Set(data_type.to_string());
        }
        if let Some(widget_type) = &modify_req.widget_type {
            rbum_kind_attr.widget_type = Set(widget_type.to_string());
        }
        if let Some(default_value) = &modify_req.default_value {
            rbum_kind_attr.default_value = Set(default_value.to_string());
        }
        if let Some(options) = &modify_req.options {
            rbum_kind_attr.options = Set(options.to_string());
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
            rbum_kind_attr.default_value = Set(action.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_kind_attr.scope_level = Set(scope_level.to_int());
        }
        Ok(rbum_kind_attr)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await?;
        if funs
            .db()
            .count(Query::select().column(rbum_item_attr::Column::Id).from(rbum_item_attr::Entity).and_where(Expr::col(rbum_item_attr::Column::RelRbumKindAttrId).eq(id)))
            .await?
            > 0
        {
            return Err(TardisError::BadRequest("can not delete rbum kind attr when there are rbum item attr".to_string()));
        }
        if funs
            .db()
            .count(Query::select().column(rbum_rel_attr::Column::Id).from(rbum_rel_attr::Entity).and_where(Expr::col(rbum_rel_attr::Column::RelRbumKindAttrId).eq(id)))
            .await?
            > 0
        {
            return Err(TardisError::BadRequest("can not delete rbum kind attr when there are rbum rel attr".to_string()));
        }
        Ok(())
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Id),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Name),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Label),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Note),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Sort),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MainColumn),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Position),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Capacity),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Overload),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DataType),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::WidgetType),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DefaultValue),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Options),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Required),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MinLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MaxLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Action),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::RelRbumKindId),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::OwnPaths),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Owner),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::CreateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::ScopeLevel),
            ])
            .from(rbum_kind_attr::Entity);

        if is_detail {
            query.expr_as(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Name), Alias::new("rel_rbum_kind_name")).inner_join(
                rbum_kind::Entity,
                Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_kind_attr::Entity, rbum_kind_attr::Column::RelRbumKindId),
            );
        }
        query.with_filter(Self::get_table_name(), filter, !is_detail, false, cxt);
        Ok(query)
    }
}
