pub mod domain {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_item, rbum_kind};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_domain_dto::{RbumDomainAddReq, RbumDomainDetailResp, RbumDomainModifyReq, RbumDomainSummaryResp};

    pub async fn add_rbum_domain<'a>(rbum_domain_add_req: &RbumDomainAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_domain<'a>(id: &str, rbum_domain_modify_req: &RbumDomainModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_domain<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_domain<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumDomainSummaryResp> {
        Ok(RbumDomainSummaryResp {})
    }

    pub async fn get_rbum_domain<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumDomainDetailResp> {
        Ok(RbumDomainDetailResp {})
    }

    pub async fn find_rbum_domains<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumDomainDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
    }
}
