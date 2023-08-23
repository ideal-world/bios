use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq, RbumSetCateFilterReq, RbumSetTreeFilterReq},
        rbum_rel_agg_dto::RbumRelAggAddReq,
        rbum_rel_dto::{RbumRelAddReq, RbumRelDetailResp},
        rbum_set_dto::RbumSetTreeResp,
    },
    rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind},
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_rel_serv::RbumRelServ, rbum_set_serv::RbumSetServ},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::{
    basic::{
        dto::iam_set_dto::IamSetCateAddReq,
        serv::{iam_cert_serv::IamCertServ, iam_set_serv::IamSetServ},
    },
    iam_enumeration::{IamRelKind, IamSetKind},
};

pub struct IamCsOrgServ;

impl IamCsOrgServ {
    pub async fn find_rel_tenant_org(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let rel_vec = RbumRelServ::find_rbums(
            &RbumRelFilterReq {
                tag: Some(IamRelKind::IamOrgRel.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::SetCate),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok(rel_vec.into_iter().map(|r| r.to_rbum_item_id).collect::<Vec<String>>())
    }

    /// 绑定 平台 set_cate_id to 租户id 修改
    pub async fn bind_cate_with_tenant(set_cate_id: &str, tenant_id: &str, kind: &IamSetKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let set_id = IamSetServ::get_default_set_id_by_ctx(kind, funs, ctx).await?;
        let tenant_set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(kind, tenant_id), true, funs, ctx).await?;
        let rel_kind: IamRelKind = match kind {
            IamSetKind::Org => IamRelKind::IamOrgRel,
            _ => {
                return Err(funs.err().not_implemented(
                    "cate",
                    "bind",
                    &format!("bind cate kind:{kind} is not implemented"),
                    "501-bind-cate-kind-is-not-implemented",
                ))
            }
        };
        if let Some(old_rel) = RbumRelServ::find_one_rbum(
            &RbumRelFilterReq {
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::SetCate),
                from_rbum_id: Some(set_cate_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            return Err(funs.err().conflict(
                "cate_rel",
                "bind",
                &format!("from_rbum_id:{} have old bind rel {:?}", set_cate_id, old_rel),
                "409-iam-bind-conflict",
            ));
        };
        if let Some(old_rel) = RbumRelServ::find_one_rbum(
            &RbumRelFilterReq {
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::SetCate),
                to_rbum_item_id: Some(tenant_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            return Err(funs.err().conflict(
                "cate_rel",
                "bind",
                &format!("to_rbum_item_id:{tenant_id} have old bind rel {:?}", old_rel),
                "409-iam-bind-conflict",
            ));
        };

        RbumRelServ::add_rel(
            &mut RbumRelAggAddReq {
                rel: RbumRelAddReq {
                    tag: rel_kind.to_string(),
                    from_rbum_kind: RbumRelFromKind::SetCate,
                    from_rbum_id: set_cate_id.to_string(),
                    to_rbum_item_id: tenant_id.to_string(),
                    to_own_paths: tenant_id.to_string(),
                    note: None,
                    to_is_outside: true,
                    ext: None,
                },
                attrs: vec![],
                envs: vec![],
            },
            funs,
            ctx,
        )
        .await?;
        //如果平台绑定的节点下有其他节点，那么全部剪切到租户层
        let platform_cates: RbumSetTreeResp = RbumSetServ::get_tree(
            &set_id,
            &RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                sys_code_query_depth: Some(1),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id.to_owned(),
            ..ctx.clone()
        };
        IamSetServ::cut_tree_to_new_set(&platform_cates, &tenant_set_id, Some(set_cate_id.to_owned()), None, funs, ctx, &mock_ctx).await?;
        if let Some(tree_main) = platform_cates.main.iter().find(|main| main.id == set_cate_id) {
            IamSetServ::add_prefix_to_sys_code_of_set(&tree_main.sys_code, &tenant_set_id, funs, &mock_ctx).await?;
        };

        Ok(())
    }

    /// 解绑的时候需要拷贝一份去平台，并且保留租户的节点 租户id to 平台 set_cate_id
    pub async fn unbind_cate_with_tenant(old_rel: RbumRelDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mock_rel_ctx = TardisContext {
            own_paths: old_rel.to_own_paths,
            ..ctx.clone()
        };
        let curr_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, ctx).await?;
        let old_rel_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, &mock_rel_ctx).await?;

        let old_rel_tree: RbumSetTreeResp = RbumSetServ::get_tree(
            &old_rel_set_id,
            &RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                ..Default::default()
            },
            funs,
            &mock_rel_ctx,
        )
        .await?;
        IamSetServ::copy_tree_to_new_set(&old_rel_tree, &curr_set_id, None, Some(old_rel.from_rbum_id.clone()), funs, ctx).await?;

        if let Some(sys_code) = IamSetServ::get_sys_code_by_cate_id(&curr_set_id, &old_rel.from_rbum_id, funs, ctx).await? {
            IamSetServ::delete_prefix_sys_code_of_set(&sys_code, &old_rel_set_id, funs, &mock_rel_ctx).await?;
        }
        RbumRelServ::delete_rbum(&old_rel.id, funs, ctx).await?;
        Ok(())
    }

    pub async fn add_set_cate(tenant_id: Option<String>, add_req: &mut IamSetCateAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut tenant_id = tenant_id;
        if let Some(pid) = add_req.rbum_parent_cate_id.clone() {
            if let Some(set_rel) = RbumRelServ::find_one_rbum(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some(ctx.own_paths.clone()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tag: Some(IamRelKind::IamOrgRel.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::SetCate),
                    from_rbum_id: Some(pid),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            {
                add_req.rbum_parent_cate_id = None;
                tenant_id = Some(set_rel.to_own_paths);
            }
        };
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.clone(), tenant_id)?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, funs, &ctx).await?;
        IamSetServ::add_set_cate(&set_id, add_req, funs, &ctx).await
    }
}
