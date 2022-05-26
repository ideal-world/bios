use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::chrono::{Duration, Utc};
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelEnvAggAddReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp};
use bios_basic::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::serv::iam_key_cache_serv::{IamCacheResRelAddOrModifyReq, IamCacheResRelDeleteReq, IamResCacheServ};
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

pub struct IamRelServ;

impl<'a> IamRelServ {
    pub async fn add_simple_rel(
        rel_kind: IamRelKind,
        from_iam_item_id: &str,
        to_iam_item_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        let value1 = start_timestamp.unwrap_or_else(|| Utc::now().timestamp());
        let value2 = end_timestamp.unwrap_or_else(|| (Utc::now() + Duration::days(365 * 100)).timestamp());
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
                    value1: value1.to_string(),
                    value2: Some(value2.to_string()),
                }]
            } else {
                vec![]
            },
        };
        RbumRelServ::add_rel(req, funs, cxt).await?;
        match rel_kind {
            IamRelKind::IamResRole => {
                let iam_res = IamResServ::peek_item(
                    from_iam_item_id,
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    cxt,
                )
                .await?;

                let uri = format!(
                    "{}://{}/{}",
                    iam_constants::RBUM_KIND_CODE_IAM_RES.to_lowercase(),
                    iam_constants::COMPONENT_CODE.to_lowercase(),
                    iam_res.code
                );
                IamResCacheServ::add_or_modify_res_rel(
                    &uri,
                    &iam_res.method,
                    &IamCacheResRelAddOrModifyReq {
                        st: if start_timestamp.is_some() { Some(value1) } else { None },
                        et: if end_timestamp.is_some() { Some(value2) } else { None },
                        accounts: vec![],
                        roles: vec![to_iam_item_id.to_string()],
                        groups: vec![],
                        apps: vec![],
                        tenants: vec![],
                    },
                    funs,
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn delete_simple_rel(rel_kind: IamRelKind, from_iam_item_id: &str, to_iam_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_iam_item_id.to_string()),
                to_rbum_item_id: Some(to_iam_item_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            cxt,
        )
        .await?;
        for rel_id in rel_ids {
            RbumRelServ::delete_rbum(&rel_id, funs, cxt).await?;
        }
        match rel_kind {
            IamRelKind::IamResRole => {
                let iam_res = IamResServ::peek_item(
                    from_iam_item_id,
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    cxt,
                )
                .await?;

                let uri = format!(
                    "{}://{}/{}",
                    iam_constants::RBUM_KIND_CODE_IAM_RES.to_lowercase(),
                    iam_constants::COMPONENT_CODE.to_lowercase(),
                    iam_res.code
                );
                IamResCacheServ::delete_res_rel(
                    &uri,
                    &iam_res.method,
                    &IamCacheResRelDeleteReq {
                        accounts: vec![],
                        roles: vec![to_iam_item_id.to_string()],
                        groups: vec![],
                        apps: vec![],
                        tenants: vec![],
                    },
                    funs,
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn count_from_rels(rel_kind: IamRelKind, with_sub_own_paths: bool, from_iam_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_from_rels(&rel_kind.to_string(), &RbumRelFromKind::Item, with_sub_own_paths, from_iam_item_id, funs, cxt).await
    }

    pub async fn find_from_simple_rels(
        rel_kind: IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_from_simple_rels(
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

    pub async fn paginate_from_simple_rels(
        rel_kind: IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        RbumRelServ::paginate_from_simple_rels(
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

    pub async fn find_to_simple_rels(
        rel_kind: IamRelKind,
        to_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_to_simple_rels(&rel_kind.to_string(), to_iam_item_id, desc_sort_by_create, desc_sort_by_update, funs, cxt).await
    }

    pub async fn paginate_to_simple_rels(
        rel_kind: IamRelKind,
        to_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        RbumRelServ::paginate_to_simple_rels(
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
}
