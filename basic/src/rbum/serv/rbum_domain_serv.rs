use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{IdResp, TardisRelDBlConnection};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::domain::{rbum_cert_conf, rbum_domain, rbum_item};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_domain_dto::{RbumDomainAddReq, RbumDomainDetailResp, RbumDomainModifyReq, RbumDomainSummaryResp};
use crate::rbum::rbum_constants::RBUM_DOMAIN_ID_LEN;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};

pub struct RbumDomainServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_domain::ActiveModel, RbumDomainAddReq, RbumDomainModifyReq, RbumDomainSummaryResp, RbumDomainDetailResp> for RbumDomainServ {
    fn get_table_name() -> &'static str {
        rbum_domain::Entity.table_name()
    }

    async fn package_add(add_req: &RbumDomainAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_domain::ActiveModel> {
        Ok(rbum_domain::ActiveModel {
            id: Set(TardisFuns::field.nanoid_len(RBUM_DOMAIN_ID_LEN)),
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            scope_level: Set(add_req.scope_level),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumDomainModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_domain::ActiveModel> {
        let mut rbum_domain = rbum_domain::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(code) = &modify_req.code {
            rbum_domain.code = Set(code.to_string());
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
        if let Some(scope_level) = modify_req.scope_level {
            rbum_domain.scope_level = Set(scope_level);
        }
        Ok(rbum_domain)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(vec![
            (rbum_domain::Entity, rbum_domain::Column::Id),
            (rbum_domain::Entity, rbum_domain::Column::UriAuthority),
            (rbum_domain::Entity, rbum_domain::Column::Name),
            (rbum_domain::Entity, rbum_domain::Column::Note),
            (rbum_domain::Entity, rbum_domain::Column::Icon),
            (rbum_domain::Entity, rbum_domain::Column::Sort),
            (rbum_domain::Entity, rbum_domain::Column::OwnPaths),
            (rbum_domain::Entity, rbum_domain::Column::Owner),
            (rbum_domain::Entity, rbum_domain::Column::CreateTime),
            (rbum_domain::Entity, rbum_domain::Column::UpdateTime),
            (rbum_domain::Entity, rbum_domain::Column::ScopeLevel),
        ]);
        query.from(rbum_domain::Entity);

        if is_detail {
            query.query_with_safe(Self::get_table_name());
        }

        query.query_with_filter(Self::get_table_name(), filter, cxt);
        query.query_with_scope(Self::get_table_name(), cxt);

        Ok(query)
    }

    async fn before_add_rbum(add_req: &mut RbumDomainAddReq, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        if db
            .count(Query::select().column(rbum_domain::Column::Id).from(rbum_domain::Entity).and_where(Expr::col(rbum_domain::Column::UriAuthority).eq(add_req.code.0.as_str())))
            .await?
            > 0
        {
            return Err(TardisError::BadRequest(format!("URI authority {} already exists", add_req.code)));
        }
        Ok(())
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, db, cxt).await?;
        if db.count(Query::select().column(rbum_item::Column::Id).from(rbum_item::Entity).and_where(Expr::col(rbum_item::Column::RelRbumDomainId).eq(id))).await? > 0 {
            return Err(TardisError::BadRequest("can not delete rbum domain when there are rbum item".to_string()));
        }
        if db.count(Query::select().column(rbum_cert_conf::Column::Id).from(rbum_cert_conf::Entity).and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(id))).await? > 0
        {
            return Err(TardisError::BadRequest("can not delete rbum domain when there are rbum cerf conf".to_string()));
        }
        Ok(())
    }
}

impl<'a> RbumDomainServ {
    pub async fn get_rbum_domain_id_by_code(code: &str, funs: &TardisFunsInst<'a>) -> TardisResult<Option<String>> {
        let resp = db
            .get_dto::<IdResp>(Query::select().column(rbum_domain::Column::Id).from(rbum_domain::Entity).and_where(Expr::col(rbum_domain::Column::UriAuthority).eq(code)))
            .await?
            .map(|r| r.id);
        Ok(resp)
    }
}
