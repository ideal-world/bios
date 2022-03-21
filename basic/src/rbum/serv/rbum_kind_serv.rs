use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{IdResp, TardisRelDBlConnection};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::constants::RBUM_KIND_ID_LEN;
use crate::rbum::domain::{rbum_kind, rbum_kind_attr};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use crate::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindDetailResp, RbumKindModifyReq, RbumKindSummaryResp};
use crate::rbum::enumeration::RbumScopeKind;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};

pub struct RbumKindServ;
pub struct RbumKindAttrServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_kind::ActiveModel, RbumKindAddReq, RbumKindModifyReq, RbumKindSummaryResp, RbumKindDetailResp> for RbumKindServ {
    fn get_table_name() -> &'static str {
        rbum_kind::Entity.table_name()
    }

    async fn package_add(add_req: &RbumKindAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_kind::ActiveModel> {
        Ok(rbum_kind::ActiveModel {
            id: Set(TardisFuns::field.nanoid_len(RBUM_KIND_ID_LEN)),
            uri_scheme: Set(add_req.uri_scheme.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ext_table_name: Set(add_req.ext_table_name.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumKindModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_kind::ActiveModel> {
        let mut rbum_kind = rbum_kind::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(uri_scheme) = &modify_req.uri_scheme {
            rbum_kind.uri_scheme = Set(uri_scheme.to_string());
        }
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
        if let Some(scope_kind) = &modify_req.scope_kind {
            rbum_kind.scope_kind = Set(scope_kind.to_string());
        }
        Ok(rbum_kind)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(vec![
            (rbum_kind::Entity, rbum_kind::Column::Id),
            (rbum_kind::Entity, rbum_kind::Column::UriScheme),
            (rbum_kind::Entity, rbum_kind::Column::Name),
            (rbum_kind::Entity, rbum_kind::Column::Note),
            (rbum_kind::Entity, rbum_kind::Column::Icon),
            (rbum_kind::Entity, rbum_kind::Column::Sort),
            (rbum_kind::Entity, rbum_kind::Column::ExtTableName),
            (rbum_kind::Entity, rbum_kind::Column::RelAppCode),
            (rbum_kind::Entity, rbum_kind::Column::UpdaterCode),
            (rbum_kind::Entity, rbum_kind::Column::CreateTime),
            (rbum_kind::Entity, rbum_kind::Column::UpdateTime),
            (rbum_kind::Entity, rbum_kind::Column::ScopeKind),
        ]);
        query.from(rbum_kind::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }
}

impl<'a> RbumKindServ {
    pub async fn get_rbum_kind_id_by_uri_scheme(uri_scheme: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<Option<String>> {
        let resp = db
            .get_dto::<IdResp>(
                Query::select()
                    .column(rbum_kind::Column::Id)
                    .from(rbum_kind::Entity)
                    .and_where(Expr::col(rbum_kind::Column::UriScheme).eq(uri_scheme))
                    .query_with_scope(Self::get_table_name(), cxt),
            )
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

    async fn package_add(add_req: &RbumKindAttrAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_kind_attr::ActiveModel> {
        Ok(rbum_kind_attr::ActiveModel {
            id: Set(format!("{}{}", add_req.rel_rbum_kind_id, TardisFuns::field.nanoid())),
            name: Set(add_req.name.to_string()),
            label: Set(add_req.label.to_string()),
            data_type_kind: Set(add_req.data_type_kind.to_string()),
            widget_type: Set(add_req.widget_type.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            main_column: Set(add_req.main_column.unwrap_or(false)),
            position: Set(add_req.position.unwrap_or(false)),
            capacity: Set(add_req.capacity.unwrap_or(false)),
            overload: Set(add_req.overload.unwrap_or(false)),
            default_value: Set(add_req.default_value.as_ref().unwrap_or(&"".to_string()).to_string()),
            options: Set(add_req.options.as_ref().unwrap_or(&"".to_string()).to_string()),
            required: Set(add_req.required.unwrap_or(false)),
            min_length: Set(add_req.min_length.unwrap_or(0)),
            max_length: Set(add_req.max_length.unwrap_or(0)),
            action: Set(add_req.action.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_kind_id: Set(add_req.rel_rbum_kind_id.to_string()),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumKindAttrModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_kind_attr::ActiveModel> {
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
        if let Some(data_type_kind) = &modify_req.data_type_kind {
            rbum_kind_attr.data_type_kind = Set(data_type_kind.to_string());
        }
        if let Some(widget_type) = &modify_req.widget_type {
            rbum_kind_attr.widget_type = Set(widget_type.to_string());
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
        if let Some(scope_kind) = &modify_req.scope_kind {
            rbum_kind_attr.scope_kind = Set(scope_kind.to_string());
        }
        Ok(rbum_kind_attr)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
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
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DataTypeKind),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::WidgetType),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DefaultValue),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Options),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Required),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MinLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MaxLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Action),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::RelRbumKindId),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::RelAppCode),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdaterCode),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::CreateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::ScopeKind),
            ])
            .from(rbum_kind_attr::Entity);

        if is_detail {
            query
                .expr_as(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Name), Alias::new("rel_rbum_kind_name"))
                .inner_join(
                    rbum_kind::Entity,
                    Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_kind_attr::Entity, rbum_kind_attr::Column::RelRbumKindId),
                )
                .query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }

    async fn before_add_rbum(add_req: &mut RbumKindAttrAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_kind_id, RbumKindServ::get_table_name(), db, cxt).await
    }
}
