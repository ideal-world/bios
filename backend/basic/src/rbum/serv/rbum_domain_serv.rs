use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::IdResp;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::IdenStatic;
use tardis::db::sea_orm::*;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::rbum::domain::{rbum_cert_conf, rbum_domain, rbum_item};
use crate::rbum::dto::rbum_domain_dto::{RbumDomainAddReq, RbumDomainDetailResp, RbumDomainModifyReq, RbumDomainSummaryResp};
use crate::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use crate::rbum::rbum_enumeration::RbumScopeLevelKind;
use crate::rbum::serv::rbum_cert_serv::RbumCertConfServ;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage, R_URL_PART_CODE};
use crate::rbum::serv::rbum_item_serv::RbumItemServ;

pub struct RbumDomainServ;

#[async_trait]
impl RbumCrudOperation<rbum_domain::ActiveModel, RbumDomainAddReq, RbumDomainModifyReq, RbumDomainSummaryResp, RbumDomainDetailResp, RbumBasicFilterReq> for RbumDomainServ {
    fn get_table_name() -> &'static str {
        rbum_domain::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut RbumDomainAddReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        if !R_URL_PART_CODE.is_match(add_req.code.as_str()) {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", &format!("code {} is invalid", add_req.code), "400-rbum-*-code-illegal"));
        }
        if funs
            .db()
            .count(Query::select().column(rbum_domain::Column::Id).from(rbum_domain::Entity).and_where(Expr::col(rbum_domain::Column::Code).eq(add_req.code.as_str())))
            .await?
            > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add", &format!("code {} already exists", add_req.code), "409-rbum-*-code-exist"));
        }
        Ok(())
    }

    async fn package_add(add_req: &RbumDomainAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_domain::ActiveModel> {
        Ok(rbum_domain::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumDomainModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_domain::ActiveModel> {
        let mut rbum_domain = rbum_domain::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
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
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_domain.scope_level = Set(scope_level.to_int());
        }
        Ok(rbum_domain)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumDomainDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
        Self::check_exist_before_delete(id, RbumItemServ::get_table_name(), rbum_item::Column::RelRbumDomainId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumCertConfServ::get_table_name(), rbum_cert_conf::Column::RelRbumDomainId.as_str(), funs).await?;
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumBasicFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(vec![
            (rbum_domain::Entity, rbum_domain::Column::Id),
            (rbum_domain::Entity, rbum_domain::Column::Code),
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
        query.from(rbum_domain::Entity).with_filter(Self::get_table_name(), filter, is_detail, true, ctx);
        Ok(query)
    }
}

impl RbumDomainServ {
    /// Get domain id by code
    ///
    /// 根据编码获取域id
    ///
    /// # Parameters
    /// - `code` - Domain code
    /// - `funs` - TardisFunsInst
    ///
    /// # Return
    /// - `Option<String>` - Domain id
    pub async fn get_rbum_domain_id_by_code(code: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let resp = funs
            .db()
            .get_dto::<IdResp>(Query::select().column(rbum_domain::Column::Id).from(rbum_domain::Entity).and_where(Expr::col(rbum_domain::Column::Code).eq(code)))
            .await?
            .map(|r| r.id);
        Ok(resp)
    }
}
