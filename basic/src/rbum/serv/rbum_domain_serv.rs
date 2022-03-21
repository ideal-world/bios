use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{IdResp, TardisRelDBlConnection};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::constants::RBUM_DOMAIN_ID_LEN;
use crate::rbum::domain::rbum_domain;
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

    async fn package_add(add_req: &RbumDomainAddReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_domain::ActiveModel> {
        Ok(rbum_domain::ActiveModel {
            id: Set(TardisFuns::field.nanoid_len(RBUM_DOMAIN_ID_LEN)),
            uri_authority: Set(add_req.uri_authority.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            scope_kind: Set(add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumDomainModifyReq, _: &TardisRelDBlConnection<'a>, _: &TardisContext) -> TardisResult<rbum_domain::ActiveModel> {
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
        Ok(rbum_domain)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(vec![
            (rbum_domain::Entity, rbum_domain::Column::Id),
            (rbum_domain::Entity, rbum_domain::Column::UriAuthority),
            (rbum_domain::Entity, rbum_domain::Column::Name),
            (rbum_domain::Entity, rbum_domain::Column::Note),
            (rbum_domain::Entity, rbum_domain::Column::Icon),
            (rbum_domain::Entity, rbum_domain::Column::Sort),
            (rbum_domain::Entity, rbum_domain::Column::RelAppCode),
            (rbum_domain::Entity, rbum_domain::Column::UpdaterCode),
            (rbum_domain::Entity, rbum_domain::Column::CreateTime),
            (rbum_domain::Entity, rbum_domain::Column::UpdateTime),
            (rbum_domain::Entity, rbum_domain::Column::ScopeKind),
        ]);
        query.from(rbum_domain::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }
}

impl<'a> RbumDomainServ {
    pub async fn get_rbum_domain_id_by_uri_authority(uri_authority: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<Option<String>> {
        let resp = db
            .get_dto::<IdResp>(
                Query::select()
                    .column(rbum_domain::Column::Id)
                    .from(rbum_domain::Entity)
                    .and_where(Expr::col(rbum_domain::Column::UriAuthority).eq(uri_authority))
                    .query_with_scope(Self::get_table_name(), cxt),
            )
            .await?
            .map(|r| r.id);
        Ok(resp)
    }
}
