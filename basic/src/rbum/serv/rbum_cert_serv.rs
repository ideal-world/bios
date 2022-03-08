pub mod cert {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp};

    pub async fn add_rbum_cert<'a>(rbum_cert_add_req: &RbumCertAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_cert<'a>(id: &str, rbum_cert_modify_req: &RbumCertModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_cert<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_cert<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertSummaryResp> {
        Ok(RbumCertSummaryResp {})
    }

    pub async fn get_rbum_cert<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertDetailResp> {
        Ok(RbumCertDetailResp {})
    }

    pub async fn find_rbum_certs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
    }
}

pub mod cert_conf {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};

    pub async fn add_rbum_cert_conf<'a>(rbum_cert_conf_add_req: &RbumCertConfAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_cert_conf<'a>(id: &str, rbum_cert_conf_modify_req: &RbumCertConfModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_cert_conf<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_cert_conf<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfSummaryResp> {
        Ok(RbumCertConfSummaryResp {})
    }

    pub async fn get_rbum_cert_conf<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        Ok(RbumCertConfDetailResp {})
    }

    pub async fn find_rbum_cert_confs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
    }
}
