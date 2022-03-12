pub mod rel {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_item, rbum_kind, rbum_rel};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelDetailResp, RbumRelModifyReq};

    pub async fn add_rbum_rel<'a>(rbum_rel_add_req: &RbumRelAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let rbum_rel_id: String = db
            .insert_one(
                rbum_rel::ActiveModel {
                    from_rbum_kind_id: Set(rbum_rel_add_req.from_rbum_kind_id.to_string()),
                    from_rbum_item_id: Set(rbum_rel_add_req.from_rbum_item_id.to_string()),
                    to_rbum_kind_id: Set(rbum_rel_add_req.to_rbum_kind_id.to_string()),
                    to_rbum_item_id: Set(rbum_rel_add_req.to_rbum_item_id.to_string()),
                    to_other_app_id: Set(rbum_rel_add_req.to_other_app_id.as_ref().unwrap_or(&"".to_string()).to_string()),
                    to_other_tenant_id: Set(rbum_rel_add_req.to_other_tenant_id.as_ref().unwrap_or(&"".to_string()).to_string()),
                    tags: Set(rbum_rel_add_req.tags.as_ref().unwrap_or(&"".to_string()).to_string()),
                    ext: Set(rbum_rel_add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_rel_id)
    }

    pub async fn modify_rbum_rel<'a>(id: &str, rbum_rel_modify_req: &RbumRelModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_rel = rbum_rel::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(tags) = &rbum_rel_modify_req.tags {
            rbum_rel.tags = Set(tags.to_string());
        }
        if let Some(ext) = &rbum_rel_modify_req.ext {
            rbum_rel.ext = Set(ext.to_string());
        }
        db.update_one(rbum_rel, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_rel<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_rel::Entity::find().filter(rbum_rel::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_rel<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelDetailResp> {
        let mut query = package_rbum_rel_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_rel) => Ok(rbum_rel),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_rels<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelDetailResp>> {
        let mut query = package_rbum_rel_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_rel::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_rel_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");
        let to_other_app_table = Alias::new("toOtherApp");
        let to_other_tenant_table = Alias::new("toOtherTenant");
        let from_rbum_kind_table = Alias::new("fromRbumKind");
        let from_rbum_item_table = Alias::new("fromRbumItem");
        let to_rbum_kind_table = Alias::new("toRbumKind");
        let to_rbum_item_table = Alias::new("toRbumItem");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_rel::Entity, rbum_rel::Column::Id),
                (rbum_rel::Entity, rbum_rel::Column::FromRbumKindId),
                (rbum_rel::Entity, rbum_rel::Column::FromRbumItemId),
                (rbum_rel::Entity, rbum_rel::Column::ToRbumKindId),
                (rbum_rel::Entity, rbum_rel::Column::ToRbumItemId),
                (rbum_rel::Entity, rbum_rel::Column::ToOtherAppId),
                (rbum_rel::Entity, rbum_rel::Column::ToOtherTenantId),
                (rbum_rel::Entity, rbum_rel::Column::Tags),
                (rbum_rel::Entity, rbum_rel::Column::Ext),
                (rbum_rel::Entity, rbum_rel::Column::RelAppId),
                (rbum_rel::Entity, rbum_rel::Column::RelTenantId),
                (rbum_rel::Entity, rbum_rel::Column::UpdaterId),
                (rbum_rel::Entity, rbum_rel::Column::CreateTime),
                (rbum_rel::Entity, rbum_rel::Column::UpdateTime),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_rel::Entity, rbum_rel::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }

        if is_detail {
            query
                .expr_as(Expr::tbl(from_rbum_kind_table.clone(), rbum_kind::Column::Name), Alias::new("from_rbum_kind_name"))
                .expr_as(Expr::tbl(from_rbum_item_table.clone(), rbum_item::Column::Name), Alias::new("from_rbum_item_name"))
                .expr_as(Expr::tbl(to_rbum_kind_table.clone(), rbum_kind::Column::Name), Alias::new("to_rbum_kind_name"))
                .expr_as(Expr::tbl(to_rbum_item_table.clone(), rbum_item::Column::Name), Alias::new("to_rbum_item_name"))
                .expr_as(Expr::tbl(to_other_app_table.clone(), rbum_item::Column::Name), Alias::new("to_other_app_name"))
                .expr_as(Expr::tbl(to_other_tenant_table.clone(), rbum_item::Column::Name), Alias::new("to_other_tenant_name"))
                .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
                .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
                .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
                .join_as(
                    JoinType::InnerJoin,
                    rbum_kind::Entity,
                    from_rbum_kind_table.clone(),
                    Expr::tbl(from_rbum_kind_table, rbum_kind::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::FromRbumKindId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    from_rbum_item_table.clone(),
                    Expr::tbl(from_rbum_item_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::FromRbumItemId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_kind::Entity,
                    to_rbum_kind_table.clone(),
                    Expr::tbl(to_rbum_kind_table, rbum_kind::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::ToRbumKindId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    to_rbum_item_table.clone(),
                    Expr::tbl(to_rbum_item_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::ToRbumItemId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    to_other_app_table.clone(),
                    Expr::tbl(to_other_app_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::ToOtherAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    to_other_tenant_table.clone(),
                    Expr::tbl(to_other_tenant_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::ToOtherTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_app_table.clone(),
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_rel::Entity, rbum_rel::Column::UpdaterId),
                );
        }

        query
    }
}

pub mod rel_attr {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_item, rbum_kind_attr, rbum_rel_attr};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_rel_attr_dto::{RbumRelAttrAddReq, RbumRelAttrDetailResp, RbumRelAttrModifyReq};

    pub async fn add_rbum_rel_attr<'a>(
        rel_rbum_rel_id: &str,
        rel_rbum_kind_attr_id: &str,
        rbum_rel_attr_add_req: &RbumRelAttrAddReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_rel_attr_id: String = db
            .insert_one(
                rbum_rel_attr::ActiveModel {
                    is_from: Set(rbum_rel_attr_add_req.is_from),
                    value: Set(rbum_rel_attr_add_req.value.to_string()),
                    rel_rbum_kind_attr_id: Set(rel_rbum_kind_attr_id.to_string()),
                    rel_rbum_rel_id: Set(rel_rbum_rel_id.to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_rel_attr_id)
    }

    pub async fn modify_rbum_rel_attr<'a>(id: &str, rbum_rel_attr_modify_req: &RbumRelAttrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_rel_attr = rbum_rel_attr::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(value) = &rbum_rel_attr_modify_req.value {
            rbum_rel_attr.value = Set(value.to_string());
        }
        db.update_one(rbum_rel_attr, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_rel_attr<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_rel_attr::Entity::find().filter(rbum_rel_attr::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_rel_attr<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelAttrDetailResp> {
        let mut query = package_rbum_rel_attr_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_rel_attr::Entity, rbum_rel_attr::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_rel_attr) => Ok(rbum_rel_attr),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_rel_attrs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAttrDetailResp>> {
        let mut query = package_rbum_rel_attr_query(true, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_rel_attr::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_rel_attr_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::Id),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::IsFrom),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::Value),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumKindAttrId),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumRelId),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::RelAppId),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::RelTenantId),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::UpdaterId),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::CreateTime),
                (rbum_rel_attr::Entity, rbum_rel_attr::Column::UpdateTime),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_rel_attr::Entity, rbum_rel_attr::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_rel_attr::Entity, rbum_rel_attr::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_rel_attr::Entity, rbum_rel_attr::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }

        if is_detail {
            query
                .expr_as(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::Name), Alias::new("rel_rbum_kind_attr_name"))
                .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
                .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
                .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
                .inner_join(
                    rbum_kind_attr::Entity,
                    Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::Id).equals(rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumKindAttrId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_app_table.clone(),
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_rel_attr::Entity, rbum_rel_attr::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_rel_attr::Entity, rbum_rel_attr::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_rel_attr::Entity, rbum_rel_attr::Column::UpdaterId),
                );
        }

        query
    }
}

pub mod rel_env {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_item, rbum_rel_env};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_rel_env_dto::{RbumRelEnvAddReq, RbumRelEnvDetailResp, RbumRelEnvModifyReq};

    pub async fn add_rbum_rel_env<'a>(
        rel_rbum_rel_id: &str,
        rbum_rel_env_add_req: &RbumRelEnvAddReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_rel_env_id: String = db
            .insert_one(
                rbum_rel_env::ActiveModel {
                    kind: Set(rbum_rel_env_add_req.kind.to_string()),
                    value1: Set(rbum_rel_env_add_req.value1.to_string()),
                    value2: Set(rbum_rel_env_add_req.value2.as_ref().unwrap_or(&"".to_string()).to_string()),
                    rel_rbum_rel_id: Set(rel_rbum_rel_id.to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_rel_env_id)
    }

    pub async fn modify_rbum_rel_env<'a>(id: &str, rbum_rel_env_modify_req: &RbumRelEnvModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_rel_env = rbum_rel_env::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(value1) = &rbum_rel_env_modify_req.value1 {
            rbum_rel_env.value1 = Set(value1.to_string());
        }
        if let Some(value2) = &rbum_rel_env_modify_req.value2 {
            rbum_rel_env.value2 = Set(value2.to_string());
        }
        db.update_one(rbum_rel_env, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_rel_env<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_rel_env::Entity::find().filter(rbum_rel_env::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_rel_env<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelEnvDetailResp> {
        let mut query = package_rbum_rel_env_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_rel_env::Entity, rbum_rel_env::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_rel_env) => Ok(rbum_rel_env),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_rel_envs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelEnvDetailResp>> {
        let mut query = package_rbum_rel_env_query(true, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_rel_env::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_rel_env_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_rel_env::Entity, rbum_rel_env::Column::Id),
                (rbum_rel_env::Entity, rbum_rel_env::Column::Kind),
                (rbum_rel_env::Entity, rbum_rel_env::Column::Value1),
                (rbum_rel_env::Entity, rbum_rel_env::Column::Value2),
                (rbum_rel_env::Entity, rbum_rel_env::Column::RelRbumRelId),
                (rbum_rel_env::Entity, rbum_rel_env::Column::RelAppId),
                (rbum_rel_env::Entity, rbum_rel_env::Column::RelTenantId),
                (rbum_rel_env::Entity, rbum_rel_env::Column::UpdaterId),
                (rbum_rel_env::Entity, rbum_rel_env::Column::CreateTime),
                (rbum_rel_env::Entity, rbum_rel_env::Column::UpdateTime),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_rel_env::Entity, rbum_rel_env::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_rel_env::Entity, rbum_rel_env::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_rel_env::Entity, rbum_rel_env::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }

        if is_detail {
            query
                .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
                .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
                .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_app_table.clone(),
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_rel_env::Entity, rbum_rel_env::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_rel_env::Entity, rbum_rel_env::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_rel_env::Entity, rbum_rel_env::Column::UpdaterId),
                );
        }

        query
    }
}
