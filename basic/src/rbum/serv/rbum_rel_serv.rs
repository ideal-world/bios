pub mod rel {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelDetailResp, RbumRelModifyReq, RbumRelSummaryResp};

    pub async fn add_rbum_rel<'a>(rbum_rel_add_req: &RbumRelAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_rel<'a>(id: &str, rbum_rel_modify_req: &RbumRelModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_rel<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_rel<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelSummaryResp> {
        Ok(RbumRelSummaryResp {})
    }

    pub async fn get_rbum_rel<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelDetailResp> {
        Ok(RbumRelDetailResp {})
    }

    pub async fn find_rbum_rels<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
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

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_rel_attr_dto::{RbumRelAttrAddReq, RbumRelAttrDetailResp, RbumRelAttrModifyReq, RbumRelAttrSummaryResp};

    pub async fn add_rbum_rel_attr<'a>(rbum_rel_attr_add_req: &RbumRelAttrAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_rel_attr<'a>(id: &str, rbum_rel_attr_modify_req: &RbumRelAttrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_rel_attr<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_rel_attr<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelAttrSummaryResp> {
        Ok(RbumRelAttrSummaryResp {})
    }

    pub async fn get_rbum_rel_attr<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelAttrDetailResp> {
        Ok(RbumRelAttrDetailResp {})
    }

    pub async fn find_rbum_rel_attrs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAttrDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
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

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_rel_env_dto::{RbumRelEnvAddReq, RbumRelEnvDetailResp, RbumRelEnvModifyReq, RbumRelEnvSummaryResp};

    pub async fn add_rbum_rel_env<'a>(rbum_rel_env_add_req: &RbumRelEnvAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_rel_env<'a>(id: &str, rbum_rel_env_modify_req: &RbumRelEnvModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_rel_env<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_rel_env<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelEnvSummaryResp> {
        Ok(RbumRelEnvSummaryResp {})
    }

    pub async fn get_rbum_rel_env<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumRelEnvDetailResp> {
        Ok(RbumRelEnvDetailResp {})
    }

    pub async fn find_rbum_rel_envs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelEnvDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
    }
}
