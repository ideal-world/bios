use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAggResp, RbumRelEnvAggAddReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelAddReq;
use bios_basic::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::iam_enumeration::IamRelKind;

pub struct IamRelServ;

impl<'a> IamRelServ {
    pub async fn add_rel(
        rel_kind: IamRelKind,
        from_iam_item_id: &str,
        to_iam_item_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: rel_kind.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_iam_item_id.to_string(),
                to_rbum_item_id: to_iam_item_id.to_string(),
                to_own_paths: cxt.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            attrs: vec![],
            envs: if start_timestamp.is_some() || end_timestamp.is_some() {
                vec![RbumRelEnvAggAddReq {
                    kind: RbumRelEnvKind::DatetimeRange,
                    value1: start_timestamp.unwrap_or(i64::MAX).to_string(),
                    value2: Some(end_timestamp.unwrap_or(i64::MAX).to_string()),
                }]
            } else {
                vec![]
            },
        };
        RbumRelServ::add_rel(req, funs, cxt).await?;
        Ok(())
    }

    pub async fn count_from_rels(rel_kind: IamRelKind, with_sub_own_paths: bool, from_iam_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_from_rels(&rel_kind.to_string(), &RbumRelFromKind::Item, with_sub_own_paths, from_iam_item_id, funs, cxt).await
    }

    pub async fn find_from_rels(
        rel_kind: IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        RbumRelServ::find_from_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn paginate_from_rels(
        rel_kind: IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        RbumRelServ::paginate_from_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn count_to_rels(rel_kind: IamRelKind, to_iam_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_to_rels(&rel_kind.to_string(), to_iam_item_id, funs, cxt).await
    }

    pub async fn find_to_rels(
        rel_kind: IamRelKind,
        to_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        RbumRelServ::find_to_rels(&rel_kind.to_string(), to_iam_item_id, desc_sort_by_create, desc_sort_by_update, funs, cxt).await
    }

    pub async fn paginate_to_rels(
        rel_kind: IamRelKind,
        to_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        RbumRelServ::paginate_to_rels(
            &rel_kind.to_string(),
            to_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_rel(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumRelServ::delete_rbum(&id, funs, cxt).await?;
        Ok(())
    }
}
