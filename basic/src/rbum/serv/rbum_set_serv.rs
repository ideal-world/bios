pub mod set {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetDetailResp, RbumSetModifyReq, RbumSetSummaryResp};

    pub async fn add_rbum_set<'a>(rbum_set_add_req: &RbumSetAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_set<'a>(id: &str, rbum_set_modify_req: &RbumSetModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_set<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_set<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetSummaryResp> {
        Ok(RbumSetSummaryResp {})
    }

    pub async fn get_rbum_set<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetDetailResp> {
        Ok(RbumSetDetailResp {})
    }

    pub async fn find_rbum_sets<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumSetDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
    }
}


pub mod set_cate {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateDetailResp, RbumSetCateModifyReq, RbumSetCateSummaryResp};

    pub async fn add_rbum_set_cate<'a>(rbum_set_cate_add_req: &RbumSetCateAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_set_cate<'a>(id: &str, rbum_set_cate_modify_req: &RbumSetCateModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_set_cate<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_set_cate<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetCateSummaryResp> {
        Ok(RbumSetCateSummaryResp {})
    }

    pub async fn get_rbum_set_cate<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetCateDetailResp> {
        Ok(RbumSetCateDetailResp {})
    }

    pub async fn find_rbum_set_cates<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumSetCateDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
    }
}

pub mod set_item {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq, RbumSetItemSummaryResp};

    pub async fn add_rbum_set_item<'a>(rbum_set_item_add_req: &RbumSetItemAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        Ok(String::from(""))
    }

    pub async fn modify_rbum_set_item<'a>(id: &str, rbum_set_item_modify_req: &RbumSetItemModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub async fn delete_rbum_set_item<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        Ok(0)
    }

    pub async fn peek_rbum_set_item<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetItemSummaryResp> {
        Ok(RbumSetItemSummaryResp {})
    }

    pub async fn get_rbum_set_item<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetItemDetailResp> {
        Ok(RbumSetItemDetailResp {})
    }

    pub async fn find_rbum_set_items<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumSetItemDetailResp>> {
        Ok(TardisPage {
            page_size,
            page_number,
            total_size: 0,
            records: vec![],
        })
    }
}
