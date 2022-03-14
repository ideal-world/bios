use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;

use crate::rbum::domain::rbum_domain;
use crate::rbum::domain::rbum_domain::ActiveModel;
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_domain_dto::{RbumDomainAddReq, RbumDomainDetailResp, RbumDomainModifyReq, RbumDomainSummaryResp};
use crate::rbum::enumeration::RbumScopeKind;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};

pub struct RbumDomainServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_domain::ActiveModel, RbumDomainAddReq, RbumDomainModifyReq, RbumDomainSummaryResp, RbumDomainDetailResp> for RbumDomainServ {
    fn get_table_name() -> &'static str {
        rbum_domain::Entity.table_name()
    }

    fn has_scope() -> bool {
        true
    }

    fn package_add(add_req: &RbumDomainAddReq, _: &TardisContext) -> rbum_domain::ActiveModel {
        rbum_domain::ActiveModel {
            uri_authority: Set(add_req.uri_authority.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            ..Default::default()
        }
    }

    fn package_modify(id: &str, modify_req: &RbumDomainModifyReq, _: &TardisContext) -> ActiveModel {
        let mut rbum_domain = rbum_domain::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(uri_authority) = &modify_req.uri_authority {
            rbum_domain.uri_authority = Set(uri_authority.to_string());
        }
        if let Some(name) = &modify_req.name {
            rbum_domain.name = Set(name.to_string());
        }
        if let Some(note) = &modify_req.note {
            rbum_domain.note = Set(note.to_string());
        }
        if let Some(icon) = &modify_req.icon {
            rbum_domain.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_domain.sort = Set(sort);
        }
        if let Some(scope_kind) = &modify_req.scope_kind {
            rbum_domain.scope_kind = Set(scope_kind.to_string());
        }
        rbum_domain
    }

    fn package_delete(id: &str, _: &TardisContext) -> Select<rbum_domain::Entity> {
        rbum_domain::Entity::find().filter(rbum_domain::Column::Id.eq(id))
    }

    fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let mut query = Query::select();
        query.columns(vec![
            (rbum_domain::Entity, rbum_domain::Column::Id),
            (rbum_domain::Entity, rbum_domain::Column::UriAuthority),
            (rbum_domain::Entity, rbum_domain::Column::Name),
            (rbum_domain::Entity, rbum_domain::Column::Note),
            (rbum_domain::Entity, rbum_domain::Column::Icon),
            (rbum_domain::Entity, rbum_domain::Column::Sort),
            (rbum_domain::Entity, rbum_domain::Column::RelAppId),
            (rbum_domain::Entity, rbum_domain::Column::RelTenantId),
            (rbum_domain::Entity, rbum_domain::Column::UpdaterId),
            (rbum_domain::Entity, rbum_domain::Column::CreateTime),
            (rbum_domain::Entity, rbum_domain::Column::UpdateTime),
            (rbum_domain::Entity, rbum_domain::Column::ScopeKind),
        ]);
        query.from(rbum_domain::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);

        query
    }
}
