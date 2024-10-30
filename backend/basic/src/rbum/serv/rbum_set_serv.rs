use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::rbum::domain::{rbum_cert, rbum_item, rbum_rel, rbum_set, rbum_set_cate, rbum_set_item};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq, RbumSetTreeFilterReq};
use crate::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateDetailResp, RbumSetCateModifyReq, RbumSetCateSummaryResp};
use crate::rbum::dto::rbum_set_dto::{
    RbumSetAddReq, RbumSetDetailResp, RbumSetModifyReq, RbumSetPathResp, RbumSetSummaryResp, RbumSetTreeExtResp, RbumSetTreeNodeResp, RbumSetTreeResp,
};
use crate::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq, RbumSetItemRelInfoResp, RbumSetItemSummaryResp};
use crate::rbum::rbum_config::RbumConfigApi;
use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use crate::rbum::serv::rbum_cert_serv::RbumCertServ;
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_kind_serv::RbumKindServ;
use crate::rbum::serv::rbum_rel_serv::RbumRelServ;
use async_recursion::async_recursion;
use async_trait::async_trait;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_orm::{self, IdenStatic};
use tardis::tokio::time::sleep;
use tardis::{TardisFuns, TardisFunsInst};

pub struct RbumSetServ;

pub struct RbumSetCateServ;

pub struct RbumSetItemServ;

#[async_trait]
impl RbumCrudOperation<rbum_set::ActiveModel, RbumSetAddReq, RbumSetModifyReq, RbumSetSummaryResp, RbumSetDetailResp, RbumSetFilterReq> for RbumSetServ {
    fn get_table_name() -> &'static str {
        rbum_set::Entity.table_name()
    }

    async fn package_add(add_req: &RbumSetAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
        Ok(rbum_set::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            kind: Set(add_req.kind.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
            disabled: Set(add_req.disabled.unwrap_or(false)),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_set::ActiveModel> {
        let mut rbum_set = rbum_set::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &modify_req.name {
            rbum_set.name = Set(name.to_string());
        }
        if let Some(note) = &modify_req.note {
            rbum_set.note = Set(note.to_string());
        }
        if let Some(icon) = &modify_req.icon {
            rbum_set.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_set.sort = Set(sort);
        }
        if let Some(ext) = &modify_req.ext {
            rbum_set.ext = Set(ext.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_set.scope_level = Set(scope_level.to_int());
        }
        if let Some(disabled) = modify_req.disabled {
            rbum_set.disabled = Set(disabled);
        }
        Ok(rbum_set)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumSetDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
        // Cannot be deleted when there are categories (nodes), associated resource items, associated relationships, and associated resource credentials.
        //
        // 存在分类（节点）、关联资源项、关联关系、关联资源凭证时不能删除。
        Self::check_exist_before_delete(id, RbumSetCateServ::get_table_name(), rbum_set_cate::Column::RelRbumSetId.as_str(), funs).await?;
        Self::check_exist_before_delete(id, RbumSetItemServ::get_table_name(), rbum_set_item::Column::RelRbumSetId.as_str(), funs).await?;
        Self::check_exist_with_cond_before_delete(
            RbumRelServ::get_table_name(),
            any![
                all![
                    Expr::col(rbum_rel::Column::FromRbumKind).eq(RbumRelFromKind::Set.to_int()),
                    Expr::col(rbum_rel::Column::FromRbumId).eq(id)
                ],
                Expr::col(rbum_rel::Column::ToRbumItemId).eq(id)
            ],
            funs,
        )
        .await?;
        Self::check_exist_with_cond_before_delete(
            RbumCertServ::get_table_name(),
            all![
                Expr::col(rbum_cert::Column::RelRbumKind).eq(RbumCertRelKind::Set.to_int()),
                Expr::col(rbum_cert::Column::RelRbumId).eq(id)
            ],
            funs,
        )
        .await?;
        // 删除缓存
        let result = Self::peek_rbum(
            id,
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let key = &format!("{}{}", funs.rbum_conf_cache_key_set_code_(), result.code);
        funs.cache().del(key).await?;
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumSetFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set::Entity, rbum_set::Column::Id),
                (rbum_set::Entity, rbum_set::Column::Code),
                (rbum_set::Entity, rbum_set::Column::Kind),
                (rbum_set::Entity, rbum_set::Column::Name),
                (rbum_set::Entity, rbum_set::Column::Note),
                (rbum_set::Entity, rbum_set::Column::Icon),
                (rbum_set::Entity, rbum_set::Column::Sort),
                (rbum_set::Entity, rbum_set::Column::Ext),
                (rbum_set::Entity, rbum_set::Column::Disabled),
                (rbum_set::Entity, rbum_set::Column::OwnPaths),
                (rbum_set::Entity, rbum_set::Column::Owner),
                (rbum_set::Entity, rbum_set::Column::CreateTime),
                (rbum_set::Entity, rbum_set::Column::UpdateTime),
                (rbum_set::Entity, rbum_set::Column::ScopeLevel),
            ])
            .from(rbum_set::Entity);
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col((rbum_set::Entity, rbum_set::Column::Kind)).eq(kind.to_string()));
        }
        if let Some(rbum_item_rel_filter_req) = &filter.rel {
            if rbum_item_rel_filter_req.rel_by_from {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).equals((rbum_set::Entity, rbum_set::Column::Id)),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).eq(rel_item_id.to_string()));
                }
            } else {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).equals((rbum_set::Entity, rbum_set::Column::Id)),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).eq(rel_item_id.to_string()));
                }
            }
            if let Some(tag) = &rbum_item_rel_filter_req.tag {
                query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Tag)).eq(tag.to_string()));
            }
            if let Some(from_rbum_kind) = &rbum_item_rel_filter_req.from_rbum_kind {
                query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).eq(from_rbum_kind.to_int()));
            }
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, true, ctx);
        Ok(query)
    }
}

impl RbumSetServ {
    /// Fetch resource tree
    ///
    /// 获取资源树
    pub async fn get_tree(rbum_set_id: &str, filter: &RbumSetTreeFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumSetTreeResp> {
        Self::check_scope(rbum_set_id, RbumSetServ::get_table_name(), funs, ctx).await?;
        // check filter.filter_cate_sys_codes scope
        if let Some(sys_codes) = &filter.sys_codes {
            let sys_code_vec: Vec<String> = sys_codes.iter().filter(|sys_code| !sys_code.is_empty()).map(|sys_code| sys_code.to_string()).collect();
            if !sys_code_vec.is_empty() {
                let tmp_set_ids = vec![rbum_set_id.to_string()];
                let values = HashMap::from([("rel_rbum_set_id".to_string(), &tmp_set_ids), ("sys_code".to_string(), &sys_code_vec)]);
                Self::check_scopes(values, sys_code_vec.len() as u64, RbumSetCateServ::get_table_name(), funs, ctx).await?;
            }
        }
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let mut rbum_set_cates = RbumSetCateServ::find_rbums(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_set_id: Some(rbum_set_id.to_string()),
                sys_codes: filter.sys_codes.clone(),
                sys_code_query_kind: filter.sys_code_query_kind.clone(),
                sys_code_query_depth: filter.sys_code_query_depth,
                cate_exts: filter.cate_exts.clone(),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        rbum_set_cates.sort_by(|a, b| a.sys_code.cmp(&b.sys_code));
        rbum_set_cates.sort_by(|a, b| a.sort.cmp(&b.sort));
        let mut tree_main = rbum_set_cates
            .iter()
            .map(|r| RbumSetTreeNodeResp {
                id: r.id.to_string(),
                sys_code: r.sys_code.to_string(),
                bus_code: r.bus_code.to_string(),
                name: r.name.to_string(),
                icon: r.icon.to_string(),
                sort: r.sort,
                ext: r.ext.to_string(),
                own_paths: r.own_paths.to_string(),
                owner: r.owner.to_string(),
                scope_level: r.scope_level.clone(),
                pid: rbum_set_cates.iter().find(|i| i.sys_code == r.sys_code[..r.sys_code.len() - set_cate_sys_code_node_len]).map(|i| i.id.to_string()),
                rel: None,
                create_time: r.create_time,
                update_time: r.update_time,
            })
            .collect();
        if !filter.fetch_cate_item {
            return Ok(RbumSetTreeResp { main: tree_main, ext: None });
        }
        let rel_rbum_item_disabled = if filter.hide_item_with_disabled { Some(false) } else { None };
        let rbum_set_items = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    // Set cate item is only used for connection,
                    // it will do scope checking on both ends of the connection when it is created,
                    // so we can release the own paths restriction here to avoid query errors.
                    // E.g.
                    // A tenant creates a set cate with scope_level=0 and associates a item,
                    // if the permission is not released, all app will not query the data.
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_set_id: Some(rbum_set_id.to_string()),
                rel_rbum_item_can_not_exist: Some(true),
                sys_code_query_kind: filter.sys_code_query_kind.clone(),
                sys_code_query_depth: filter.sys_code_query_depth,
                rel_rbum_set_cate_sys_codes: filter.sys_codes.clone(),
                rel_rbum_item_ids: filter.rel_rbum_item_ids.clone(),
                rel_rbum_item_kind_ids: filter.rel_rbum_item_kind_ids.clone(),
                rel_rbum_item_domain_ids: filter.rel_rbum_item_domain_ids.clone(),
                rel_rbum_item_disabled,
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if filter.hide_cate_with_empty_item {
            // Filter out nodes without associated resource items
            //
            // 过滤掉没有关联资源项的节点
            let cate_ids_has_item =
                tree_main.iter().filter(|cate| cate.pid.is_none()).flat_map(|cate| Self::filter_exist_items(&tree_main, &cate.id, &rbum_set_items)).collect::<Vec<String>>();
            tree_main.retain(|cate| cate_ids_has_item.contains(&cate.id));
        }
        let mut items = tree_main
            .iter()
            .map(|cate| {
                (
                    cate.id.clone(),
                    rbum_set_items
                        .iter()
                        .filter(|i| i.rel_rbum_set_cate_id.clone().unwrap_or_default() == cate.id)
                        .map(|i| RbumSetItemRelInfoResp {
                            id: i.id.to_string(),
                            sort: i.sort,
                            rel_rbum_item_id: i.rel_rbum_item_id.to_string(),
                            rel_rbum_item_code: i.rel_rbum_item_code.to_string(),
                            rel_rbum_item_name: i.rel_rbum_item_name.to_string(),
                            rel_rbum_item_kind_id: i.rel_rbum_item_kind_id.to_string(),
                            rel_rbum_item_domain_id: i.rel_rbum_item_domain_id.to_string(),
                            rel_rbum_item_owner: i.rel_rbum_item_owner.to_string(),
                            rel_rbum_item_create_time: i.rel_rbum_item_create_time,
                            rel_rbum_item_update_time: i.rel_rbum_item_update_time,
                            rel_rbum_item_disabled: i.rel_rbum_item_disabled,
                            rel_rbum_item_scope_level: i.rel_rbum_item_scope_level.clone(),
                            own_paths: i.own_paths.to_string(),
                            owner: i.owner.to_string(),
                        })
                        .collect(),
                )
            })
            .collect::<HashMap<String, Vec<RbumSetItemRelInfoResp>>>();
        items.insert(
            "".to_string(),
            rbum_set_items
                .iter()
                .filter(|i| i.rel_rbum_set_cate_id.is_none())
                .map(|i| RbumSetItemRelInfoResp {
                    id: i.id.to_string(),
                    sort: i.sort,
                    rel_rbum_item_id: i.rel_rbum_item_id.to_string(),
                    rel_rbum_item_code: i.rel_rbum_item_code.to_string(),
                    rel_rbum_item_name: i.rel_rbum_item_name.to_string(),
                    rel_rbum_item_kind_id: i.rel_rbum_item_kind_id.to_string(),
                    rel_rbum_item_domain_id: i.rel_rbum_item_domain_id.to_string(),
                    rel_rbum_item_owner: i.rel_rbum_item_owner.to_string(),
                    rel_rbum_item_create_time: i.rel_rbum_item_create_time,
                    rel_rbum_item_update_time: i.rel_rbum_item_update_time,
                    rel_rbum_item_disabled: i.rel_rbum_item_disabled,
                    rel_rbum_item_scope_level: i.rel_rbum_item_scope_level.clone(),
                    own_paths: i.own_paths.to_string(),
                    owner: i.owner.to_string(),
                })
                .collect(),
        );
        let mut item_number_agg = tree_main
            .iter()
            .map(|cate| {
                (
                    cate.id.to_string(),
                    tree_main
                        .iter()
                        .filter(|c| c.sys_code.starts_with(&cate.sys_code))
                        .flat_map(|c| items.get(&c.id).expect("ignore"))
                        .chunk_by(|c| c.rel_rbum_item_kind_id.clone())
                        .into_iter()
                        .map(|(g, c)| (g, c.map(|i| i.rel_rbum_item_id.clone()).collect::<HashSet<String>>().len() as u64))
                        .collect::<HashMap<String, u64>>(),
                )
            })
            .collect::<HashMap<String, HashMap<String, u64>>>();
        // Add an aggregate to root
        item_number_agg.insert(
            "".to_string(),
            items
                .values()
                .flat_map(|item| item.iter())
                .chunk_by(|c| c.rel_rbum_item_kind_id.clone())
                .into_iter()
                .map(|(g, c)| (g, c.map(|i| i.rel_rbum_item_id.clone()).collect::<HashSet<String>>().len() as u64))
                .collect::<HashMap<String, u64>>(),
        );
        let kind_ids = Vec::from_iter(items.values().flat_map(|items| items.iter().map(|item| item.rel_rbum_item_kind_id.clone())).collect::<HashSet<String>>());
        let item_kinds = RbumKindServ::find_rbums(
            &RbumKindFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(kind_ids),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|kind| (kind.id.clone(), kind))
        .collect();

        let domain_ids: Vec<String> = Vec::from_iter(items.values().flat_map(|items| items.iter().map(|item| item.rel_rbum_item_domain_id.clone())).collect::<HashSet<String>>());
        let item_domains = RbumDomainServ::find_rbums(
            &RbumBasicFilterReq {
                ids: Some(domain_ids),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|domain| (domain.id.clone(), domain))
        .collect();

        Ok(RbumSetTreeResp {
            main: tree_main,
            ext: Some(RbumSetTreeExtResp {
                items,
                item_number_agg,
                item_kinds,
                item_domains,
            }),
        })
    }

    /// From the first layer node, recursively find the leaf node,
    /// if the leaf node does not have a related resource item, return empty, otherwise return all node ids from the first layer node to the leaf node.
    ///
    /// 从第一层节点递归查找到叶子节点，如果叶子节点没有关联资源项，则返回空，反之返回从第一层节点到叶子节点的所有节点id。
    fn filter_exist_items(tree_main: &Vec<RbumSetTreeNodeResp>, cate_id: &str, rbum_set_items: &Vec<RbumSetItemDetailResp>) -> Vec<String> {
        let mut sub_cates = tree_main
            .iter()
            .filter(|cate| cate.pid == Some(cate_id.to_string()))
            .flat_map(|cate| Self::filter_exist_items(tree_main, &cate.id, rbum_set_items))
            .collect::<Vec<String>>();
        let cate = tree_main.iter().find(|cate| cate.id == cate_id).expect("ignore");
        if sub_cates.is_empty() {
            // leaf node
            if !rbum_set_items.iter().any(|item| item.rel_rbum_set_cate_id.clone().unwrap_or_default() == cate.id) {
                vec![]
            } else {
                vec![cate.id.to_string()]
            }
        } else {
            sub_cates.insert(0, cate.id.to_string());
            sub_cates
        }
    }

    /// Get resource set id by resource set code
    ///
    /// 根据资源集编码获取对应的id
    pub async fn get_rbum_set_id_by_code(code: &str, with_sub: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        let key = &format!("{}{}", funs.rbum_conf_cache_key_set_code_(), code);
        if let Some(cached_id) = funs.cache().get(key).await? {
            Ok(Some(cached_id))
        } else if let Some(rbum_set) = Self::find_one_rbum(
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(code.to_string()),
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            funs.cache().set_ex(key, &rbum_set.id, funs.rbum_conf_cache_key_set_code_expire_sec() as u64).await?;
            Ok(Some(rbum_set.id))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl RbumCrudOperation<rbum_set_cate::ActiveModel, RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryResp, RbumSetCateDetailResp, RbumSetCateFilterReq>
    for RbumSetCateServ
{
    fn get_table_name() -> &'static str {
        rbum_set_cate::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut RbumSetCateAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), funs, ctx).await?;
        if let Some(rbum_parent_cate_id) = &add_req.rbum_parent_cate_id {
            Self::check_scope(rbum_parent_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
        }
        Ok(())
    }

    async fn package_add(add_req: &RbumSetCateAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
        let sys_code = Self::package_sys_code(&add_req.rel_rbum_set_id, add_req.rbum_parent_cate_id.as_deref(), funs, ctx).await?;
        Ok(rbum_set_cate::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            sys_code: Set(sys_code),
            bus_code: Set(add_req.bus_code.to_string()),
            name: Set(add_req.name.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            scope_level: Set(add_req.scope_level.as_ref().unwrap_or(&RbumScopeLevelKind::Private).to_int()),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetCateModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<rbum_set_cate::ActiveModel> {
        let mut rbum_set_cate = rbum_set_cate::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(rbum_parent_cate_id) = &modify_req.rbum_parent_cate_id {
            if let Some(detail) = Self::find_one_detail_rbum(
                &RbumSetCateFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(vec![rbum_parent_cate_id.to_string()]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            {
                let sys_code = Self::package_sys_code(&detail.rel_rbum_set_id, Some(rbum_parent_cate_id.as_str()), funs, ctx).await?;
                rbum_set_cate.sys_code = Set(sys_code.to_string());
            }
        }
        if let Some(bus_code) = &modify_req.bus_code {
            rbum_set_cate.bus_code = Set(bus_code.to_string());
        }
        if let Some(name) = &modify_req.name {
            rbum_set_cate.name = Set(name.to_string());
        }
        if let Some(icon) = &modify_req.icon {
            rbum_set_cate.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            rbum_set_cate.sort = Set(sort);
        }
        if let Some(ext) = &modify_req.ext {
            rbum_set_cate.ext = Set(ext.to_string());
        }
        if let Some(scope_level) = &modify_req.scope_level {
            rbum_set_cate.scope_level = Set(scope_level.to_int());
        }
        Ok(rbum_set_cate)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumSetCateDetailResp>> {
        Self::check_ownership(id, funs, ctx).await?;
        if funs
            .db()
            .count(
                Query::select()
                    .column((rbum_set_item::Entity, rbum_set_item::Column::Id))
                    .from(rbum_set_item::Entity)
                    .inner_join(
                        rbum_set_cate::Entity,
                        Condition::all()
                            .add(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).equals((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)))
                            .add(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId)).equals((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId))),
                    )
                    .inner_join(
                        rbum_item::Entity,
                        Expr::col((rbum_item::Entity, rbum_item::Column::Id)).equals((rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId)),
                    )
                    .and_where(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::Id)).eq(id))
                    .and_where(Expr::col((rbum_item::Entity, rbum_item::Column::Disabled)).eq(false)),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                &format!("can not delete {}.{} when there are associated by set_item", Self::get_obj_name(), id),
                "409-rbum-*-delete-conflict",
            ));
        }
        let set_cate = Self::peek_rbum(
            id,
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_set_cate::Column::Id)
                    .from(rbum_set_cate::Entity)
                    .and_where(Expr::col(rbum_set_cate::Column::Id).ne(id))
                    .and_where(Expr::col(rbum_set_cate::Column::RelRbumSetId).eq(set_cate.rel_rbum_set_id.as_str()))
                    .and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{}%", set_cate.sys_code.as_str()).as_str())),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                &format!("can not delete {}.{} when there are associated by sub set_cate", Self::get_obj_name(), id),
                "409-rbum-*-delete-conflict",
            ));
        }
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumSetCateFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                (rbum_set_cate::Entity, rbum_set_cate::Column::SysCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Name),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Icon),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Sort),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Ext),
                (rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::OwnPaths),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Owner),
                (rbum_set_cate::Entity, rbum_set_cate::Column::CreateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::ScopeLevel),
            ])
            .from(rbum_set_cate::Entity);
        if let Some(rel_rbum_set_id) = &filter.rel_rbum_set_id {
            query.and_where(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId)).eq(rel_rbum_set_id.to_string()));
        }
        if let Some(sys_codes) = &filter.sys_codes {
            let query_kind = filter.sys_code_query_kind.clone().unwrap_or(RbumSetCateLevelQueryKind::Current);
            if sys_codes.is_empty() {
                if query_kind == RbumSetCateLevelQueryKind::Current {
                    // query the first level
                    query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq(funs.rbum_conf_set_cate_sys_code_node_len() as i32));
                }
            } else {
                let mut cond = Cond::any();
                match query_kind {
                    RbumSetCateLevelQueryKind::CurrentAndSub => {
                        if let Some(depth) = filter.sys_code_query_depth {
                            for sys_code in sys_codes {
                                cond = cond.add(all![
                                    Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).like(format!("{sys_code}%").as_str()),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode)))
                                        .lte((sys_code.len() + funs.rbum_conf_set_cate_sys_code_node_len() * depth as usize) as i32),
                                ]);
                            }
                        } else {
                            for sys_code in sys_codes {
                                cond = cond.add(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).like(format!("{sys_code}%").as_str()));
                            }
                        }
                    }
                    RbumSetCateLevelQueryKind::Sub => {
                        if let Some(depth) = filter.sys_code_query_depth {
                            for sys_code in sys_codes {
                                cond = cond.add(all![
                                    Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).like(format!("{sys_code}%").as_str()),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).gt(sys_code.len() as i32),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode)))
                                        .lte((sys_code.len() + funs.rbum_conf_set_cate_sys_code_node_len() * depth as usize) as i32),
                                ]);
                            }
                        } else {
                            for sys_code in sys_codes {
                                cond = cond.add(all![
                                    Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).like(format!("{sys_code}%").as_str()),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).gt(sys_code.len() as i32)
                                ]);
                            }
                        }
                    }
                    RbumSetCateLevelQueryKind::CurrentAndParent => {
                        for sys_code in sys_codes {
                            let mut sys_codes = Self::get_parent_sys_codes(sys_code, funs)?;
                            sys_codes.insert(0, sys_code.to_string());
                            cond = cond.add(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).is_in(sys_codes));
                        }
                    }
                    RbumSetCateLevelQueryKind::Parent => {
                        for sys_code in sys_codes {
                            let parent_sys_codes = Self::get_parent_sys_codes(sys_code, funs)?;
                            cond = cond.add(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).is_in(parent_sys_codes));
                        }
                    }
                    RbumSetCateLevelQueryKind::Current => {
                        for sys_code in sys_codes {
                            cond = cond.add(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).eq(sys_code.as_str()));
                        }
                    }
                }
                query.cond_where(all![cond]);
            }
        }
        if let Some(cate_exts) = &filter.cate_exts {
            query.and_where(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::Ext)).is_in(cate_exts.clone()));
        }
        if let Some(rbum_item_rel_filter_req) = &filter.rel {
            if rbum_item_rel_filter_req.rel_by_from {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).equals((rbum_set_cate::Entity, rbum_set_cate::Column::Id)),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).eq(rel_item_id.to_string()));
                }
            } else {
                query.inner_join(
                    rbum_rel::Entity,
                    Expr::col((rbum_rel::Entity, rbum_rel::Column::ToRbumItemId)).equals((rbum_set_cate::Entity, rbum_set_cate::Column::Id)),
                );
                if let Some(rel_item_id) = &rbum_item_rel_filter_req.rel_item_id {
                    query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumId)).eq(rel_item_id.to_string()));
                }
            }
            if let Some(tag) = &rbum_item_rel_filter_req.tag {
                query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::Tag)).eq(tag.to_string()));
            }
            if let Some(from_rbum_kind) = &rbum_item_rel_filter_req.from_rbum_kind {
                query.and_where(Expr::col((rbum_rel::Entity, rbum_rel::Column::FromRbumKind)).eq(from_rbum_kind.to_int()));
            }
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, true, ctx);
        query.order_by(rbum_set_cate::Column::Sort, Order::Asc);
        Ok(query)
    }
}

impl RbumSetCateServ {
    /// Fetch the sys_code of the parent node
    ///
    /// 获取父级节点的sys_code集合
    fn get_parent_sys_codes(sys_code: &str, funs: &TardisFunsInst) -> TardisResult<Vec<String>> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let mut level = sys_code.len() / set_cate_sys_code_node_len - 1;
        if level == 0 {
            return Ok(vec![]);
        }
        let mut sys_code_item = Vec::with_capacity(level);
        while level != 0 {
            sys_code_item.push(sys_code[..set_cate_sys_code_node_len * level].to_string());
            level -= 1;
        }
        Ok(sys_code_item)
    }

    /// Fetch the sys_code of the new node
    ///
    /// 获取新节点的sys_code
    async fn package_sys_code(rbum_set_id: &str, rbum_set_parent_cate_id: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let lock_key = format!("rbum_set_cate_sys_code_{rbum_set_id}");
        while !funs.cache().set_nx(&lock_key, "waiting").await? {
            sleep(Duration::from_millis(100)).await;
        }
        funs.cache().expire(&lock_key, 10).await?;
        let sys_code = if let Some(rbum_set_parent_cate_id) = rbum_set_parent_cate_id {
            let rel_parent_sys_code = Self::get_sys_code(rbum_set_parent_cate_id, funs, ctx).await?;
            Self::get_max_sys_code_by_level(rbum_set_id, Some(&rel_parent_sys_code), funs, ctx).await
        } else {
            Self::get_max_sys_code_by_level(rbum_set_id, None, funs, ctx).await
        };
        funs.cache().del(&lock_key).await?;
        sys_code
    }

    /// Fetch the max sys_code of the current node
    ///
    /// 获取当前节点的最大sys_code
    async fn get_max_sys_code_by_level(rbum_set_id: &str, parent_sys_code: Option<&str>, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<String> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let mut query = Query::select();
        query.columns(vec![(rbum_set_cate::Column::SysCode)]).from(rbum_set_cate::Entity).and_where(Expr::col(rbum_set_cate::Column::RelRbumSetId).eq(rbum_set_id));

        if let Some(parent_sys_code) = parent_sys_code {
            query.and_where(Expr::col(rbum_set_cate::Column::SysCode).like(format!("{parent_sys_code}%").as_str()));
            query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq((parent_sys_code.len() + set_cate_sys_code_node_len) as i32));
        } else {
            // fetch max code in level 1
            query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_cate::Column::SysCode))).eq(set_cate_sys_code_node_len as i32));
        }
        query.order_by(rbum_set_cate::Column::SysCode, Order::Desc);
        let current_max_sys_code = funs.db().get_dto::<SysCodeResp>(&query).await?.map(|r| r.sys_code);
        if let Some(current_max_sys_code) = current_max_sys_code {
            if current_max_sys_code.len() != set_cate_sys_code_node_len {
                // if level N (N!=1) not empty
                let current_level_sys_code = current_max_sys_code[current_max_sys_code.len() - set_cate_sys_code_node_len..].to_string();
                let parent_sys_code = current_max_sys_code[..current_max_sys_code.len() - set_cate_sys_code_node_len].to_string();
                let current_level_sys_code = TardisFuns::field.incr_by_base36(&current_level_sys_code).ok_or_else(|| {
                    funs.err().bad_request(
                        &Self::get_obj_name(),
                        "get_sys_code",
                        "current number of nodes is saturated",
                        "400-rbum-set-sys-code-saturated",
                    )
                })?;
                Ok(format!("{parent_sys_code}{current_level_sys_code}"))
            } else {
                // if level 1 not empty
                Ok(TardisFuns::field.incr_by_base36(&current_max_sys_code).ok_or_else(|| {
                    funs.err().bad_request(
                        &Self::get_obj_name(),
                        "get_sys_code",
                        "current number of nodes is saturated",
                        "400-rbum-set-sys-code-saturated",
                    )
                })?)
            }
        } else if let Some(parent_sys_code) = parent_sys_code {
            // if level N (N!=1) is empty
            Ok(format!("{}{}", parent_sys_code, String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?))
        } else {
            // if level 1 is empty
            Ok(String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?)
        }
    }

    /// Fetch the sys_code of the specified node
    ///
    /// 获取指定节点的sys_code
    async fn get_sys_code(rbum_set_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Self::check_scope(rbum_set_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
        let sys_code = funs
            .db()
            .get_dto::<SysCodeResp>(
                Query::select().column(rbum_set_cate::Column::SysCode).from(rbum_set_cate::Entity).and_where(Expr::col(rbum_set_cate::Column::Id).eq(rbum_set_cate_id)),
            )
            .await?
            .ok_or_else(|| {
                funs.err().not_found(
                    &Self::get_obj_name(),
                    "get_sys_code",
                    &format!("not found set cate {rbum_set_cate_id}"),
                    "404-rbum-set-cate-not-exist",
                )
            })?
            .sys_code;
        Ok(sys_code)
    }

    #[async_recursion]
    pub async fn move_set_cate(set_cate_id: &str, parent_set_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let set_cate_detail = Self::get_rbum(
            set_cate_id,
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let set_item_ids = RbumSetItemServ::find_id_rbums(
            &RbumSetItemFilterReq {
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Current),
                rel_rbum_set_cate_sys_codes: Some(vec![set_cate_detail.sys_code.clone()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let result = Self::modify_rbum(
            set_cate_id,
            &mut RbumSetCateModifyReq {
                bus_code: None,
                name: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: Some(parent_set_cate_id.to_string()),
                scope_level: None,
            },
            funs,
            ctx,
        )
        .await;
        for set_item_id in set_item_ids {
            RbumSetItemServ::modify_rbum(
                &set_item_id,
                &mut RbumSetItemModifyReq {
                    rel_rbum_set_cate_id: Some(set_cate_id.to_string()),
                    sort: None,
                },
                funs,
                ctx,
            )
            .await?;
        }

        let child_set_cates = Self::find_rbums(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_set_id: Some(set_cate_detail.rel_rbum_set_id.clone()),
                sys_codes: Some(vec![set_cate_detail.sys_code.clone()]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                sys_code_query_depth: Some(1),
                cate_exts: None,
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for child_set_cate in child_set_cates {
            Self::move_set_cate(&child_set_cate.id, &set_cate_detail.id, funs, ctx).await?;
        }

        result
    }
}

#[async_trait]
impl RbumCrudOperation<rbum_set_item::ActiveModel, RbumSetItemAddReq, RbumSetItemModifyReq, RbumSetItemSummaryResp, RbumSetItemDetailResp, RbumSetItemFilterReq>
    for RbumSetItemServ
{
    fn get_table_name() -> &'static str {
        rbum_set_item::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut RbumSetItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_set_id, RbumSetServ::get_table_name(), funs, ctx).await?;
        Self::check_scope(&add_req.rel_rbum_item_id, RbumItemServ::get_table_name(), funs, ctx).await?;
        let rel_sys_code = if add_req.rel_rbum_set_cate_id.is_empty() {
            "".to_string()
        } else {
            Self::check_scope(&add_req.rel_rbum_set_cate_id, RbumSetCateServ::get_table_name(), funs, ctx).await?;
            RbumSetCateServ::get_sys_code(add_req.rel_rbum_set_cate_id.as_str(), funs, ctx).await?
        };
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_set_item::Column::Id)
                    .from(rbum_set_item::Entity)
                    .and_where(Expr::col(rbum_set_item::Column::RelRbumSetId).eq(add_req.rel_rbum_set_id.as_str()))
                    .and_where(Expr::col(rbum_set_item::Column::RelRbumItemId).eq(add_req.rel_rbum_item_id.as_str()))
                    .and_where(Expr::col(rbum_set_item::Column::RelRbumSetCateCode).eq(rel_sys_code.as_str())),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add", "item already exists", "409-rbum-set-item-exist"));
        }
        Ok(())
    }

    async fn package_add(add_req: &RbumSetItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        let rel_sys_code = if add_req.rel_rbum_set_cate_id.is_empty() {
            "".to_string()
        } else {
            RbumSetCateServ::get_sys_code(add_req.rel_rbum_set_cate_id.as_str(), funs, ctx).await?
        };
        Ok(rbum_set_item::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            rel_rbum_set_id: Set(add_req.rel_rbum_set_id.to_string()),
            rel_rbum_set_cate_code: Set(rel_sys_code),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.to_string()),
            sort: Set(add_req.sort),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &RbumSetItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<rbum_set_item::ActiveModel> {
        let mut rbum_set_item = rbum_set_item::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(sort) = modify_req.sort {
            rbum_set_item.sort = Set(sort);
        }
        if let Some(rel_rbum_set_cate_id) = &modify_req.rel_rbum_set_cate_id {
            rbum_set_item.rel_rbum_set_cate_code = Set(RbumSetCateServ::get_sys_code(rel_rbum_set_cate_id.as_str(), funs, ctx).await?);
        }

        Ok(rbum_set_item)
    }

    async fn package_query(is_detail: bool, filter: &RbumSetItemFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let rel_item_table = Alias::new("relItem");
        let rbum_set_cate_join_type = if let Some(true) = filter.rel_rbum_item_can_not_exist {
            JoinType::LeftJoin
        } else {
            JoinType::InnerJoin
        };
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_item::Entity, rbum_set_item::Column::Id),
                (rbum_set_item::Entity, rbum_set_item::Column::Sort),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
                (rbum_set_item::Entity, rbum_set_item::Column::OwnPaths),
                (rbum_set_item::Entity, rbum_set_item::Column::Owner),
                (rbum_set_item::Entity, rbum_set_item::Column::CreateTime),
                (rbum_set_item::Entity, rbum_set_item::Column::UpdateTime),
            ])
            .expr_as(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::Id)), Alias::new("rel_rbum_set_cate_id"))
            .expr_as(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)), Alias::new("rel_rbum_set_cate_sys_code"))
            .expr_as(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::Name)), Alias::new("rel_rbum_set_cate_name"))
            .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::Name)), Alias::new("rel_rbum_item_name"))
            .from(rbum_set_item::Entity)
            .join(
                rbum_set_cate_join_type,
                rbum_set_cate::Entity,
                all![
                    Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::SysCode)).equals((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)),
                    Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::RelRbumSetId)).equals((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId))
                ],
            )
            .join_as(
                JoinType::InnerJoin,
                rbum_item::Entity,
                rel_item_table.clone(),
                Expr::col((rel_item_table.clone(), rbum_item::Column::Id)).equals((rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId)),
            );
        if is_detail {
            query
                .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::Code)), Alias::new("rel_rbum_item_code"))
                .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::RelRbumKindId)), Alias::new("rel_rbum_item_kind_id"))
                .expr_as(
                    Expr::col((rel_item_table.clone(), rbum_item::Column::RelRbumDomainId)),
                    Alias::new("rel_rbum_item_domain_id"),
                )
                .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::Owner)), Alias::new("rel_rbum_item_owner"))
                .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::CreateTime)), Alias::new("rel_rbum_item_create_time"))
                .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::UpdateTime)), Alias::new("rel_rbum_item_update_time"))
                .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::Disabled)), Alias::new("rel_rbum_item_disabled"))
                .expr_as(Expr::col((rel_item_table.clone(), rbum_item::Column::ScopeLevel)), Alias::new("rel_rbum_item_scope_level"));
        }
        if let Some(rel_rbum_set_id) = &filter.rel_rbum_set_id {
            query.and_where(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetId)).eq(rel_rbum_set_id.to_string()));
        }
        if let Some(rel_rbum_set_cate_ids) = &filter.rel_rbum_set_cate_ids {
            query.and_where(Expr::col((rbum_set_cate::Entity, rbum_set_cate::Column::Id)).is_in(rel_rbum_set_cate_ids.clone()));
        }
        if let Some(rel_rbum_item_ids) = &filter.rel_rbum_item_ids {
            query.and_where(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId)).is_in(rel_rbum_item_ids.clone()));
        }
        if let Some(rel_rbum_item_scope_level) = &filter.rel_rbum_item_scope_level {
            query.and_where(Expr::col((rel_item_table.clone(), rbum_item::Column::ScopeLevel)).eq(rel_rbum_item_scope_level.to_int()));
        }
        if let Some(rel_rbum_item_disabled) = &filter.rel_rbum_item_disabled {
            query.and_where(Expr::col((rel_item_table.clone(), rbum_item::Column::Disabled)).eq(*rel_rbum_item_disabled));
        }
        if let Some(rel_rbum_item_domain_ids) = &filter.rel_rbum_item_domain_ids {
            query.and_where(Expr::col((rel_item_table.clone(), rbum_item::Column::RelRbumDomainId)).is_in(rel_rbum_item_domain_ids.clone()));
        }
        if let Some(rel_rbum_item_kind_ids) = &filter.rel_rbum_item_kind_ids {
            query.and_where(Expr::col((rel_item_table, rbum_item::Column::RelRbumKindId)).is_in(rel_rbum_item_kind_ids.clone()));
        }
        if let Some(sys_codes) = &filter.rel_rbum_set_cate_sys_codes {
            let query_kind = filter.sys_code_query_kind.clone().unwrap_or(RbumSetCateLevelQueryKind::Current);
            if sys_codes.is_empty() {
                if query_kind == RbumSetCateLevelQueryKind::Current {
                    // query the first level
                    query.and_where(Expr::expr(Func::char_length(Expr::col(rbum_set_item::Column::RelRbumSetCateCode))).eq(funs.rbum_conf_set_cate_sys_code_node_len() as i32));
                }
            } else {
                let mut cond = Cond::any();
                match query_kind {
                    RbumSetCateLevelQueryKind::CurrentAndSub => {
                        if let Some(depth) = filter.sys_code_query_depth {
                            for sys_code in sys_codes {
                                cond = cond.add(all![
                                    Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).like(format!("{sys_code}%").as_str()),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_item::Column::RelRbumSetCateCode)))
                                        .lte((sys_code.len() + funs.rbum_conf_set_cate_sys_code_node_len() * depth as usize) as i32),
                                ]);
                            }
                        } else {
                            for sys_code in sys_codes {
                                cond = cond.add(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).like(format!("{sys_code}%").as_str()));
                            }
                        }
                    }
                    RbumSetCateLevelQueryKind::Sub => {
                        if let Some(depth) = filter.sys_code_query_depth {
                            for sys_code in sys_codes {
                                cond = cond.add(all![
                                    Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).like(format!("{sys_code}%").as_str()),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_item::Column::RelRbumSetCateCode))).gt(sys_code.len() as i32),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_item::Column::RelRbumSetCateCode)))
                                        .lte((sys_code.len() + funs.rbum_conf_set_cate_sys_code_node_len() * depth as usize) as i32),
                                ]);
                            }
                        } else {
                            for sys_code in sys_codes {
                                cond = cond.add(all![
                                    Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).like(format!("{sys_code}%").as_str()),
                                    Expr::expr(Func::char_length(Expr::col(rbum_set_item::Column::RelRbumSetCateCode))).gt(sys_code.len() as i32)
                                ]);
                            }
                        }
                    }
                    RbumSetCateLevelQueryKind::CurrentAndParent => {
                        for sys_code in sys_codes {
                            let mut sys_codes = RbumSetCateServ::get_parent_sys_codes(sys_code, funs)?;
                            sys_codes.insert(0, sys_code.to_string());
                            cond = cond.add(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).is_in(sys_codes));
                        }
                    }
                    RbumSetCateLevelQueryKind::Parent => {
                        for sys_code in sys_codes {
                            let parent_sys_codes = RbumSetCateServ::get_parent_sys_codes(sys_code, funs)?;
                            cond = cond.add(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).is_in(parent_sys_codes));
                        }
                    }
                    RbumSetCateLevelQueryKind::Current => {
                        for sys_code in sys_codes {
                            cond = cond.add(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).eq(sys_code.as_str()));
                        }
                    }
                }
                query.cond_where(all![cond]);
            }
        }
        if let Some(rbum_set_item_cate_code) = &filter.rel_rbum_set_item_cate_code {
            query.and_where(Expr::col((rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode)).eq(rbum_set_item_cate_code.as_str()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl RbumSetItemServ {
    /// Fetch all the paths of the resource item in the resource set
    ///
    /// 获取资源项在某一资源集上的所有路径
    ///
    /// Return format:
    ///
    /// * The first-level array is all the paths corresponding to the resource item (a resource item can hang multiple paths)
    /// * The second-level data is each node (such as the path is l1/l2/l3, then the corresponding node is l1, l2, l3)
    ///
    /// * 第一层数组为资源项对应的所有路径（一个资源项可以挂多个路径）
    /// * 第二层数据为每个节点（比如路径为 l1/l2/l3， 那么对应的节点为 l1, l2, l3）
    ///
    /// ```json
    /// [
    ///     [
    ///         {
    ///             "id": "Node Id",
    ///             "name": "Node Name"
    ///         }
    ///     ]
    /// ]
    /// ```
    pub async fn find_set_paths(rbum_item_id: &str, rbum_set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<Vec<RbumSetPathResp>>> {
        let rbum_set_cate_sys_codes: Vec<String> = Self::find_rbums(
            &RbumSetItemFilterReq {
                rel_rbum_item_can_not_exist: Some(true),
                rel_rbum_set_id: Some(rbum_set_id.to_string()),
                rel_rbum_item_ids: Some(vec![rbum_item_id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|item| item.rel_rbum_set_cate_sys_code.unwrap_or_default())
        .collect();
        let mut result: Vec<Vec<RbumSetPathResp>> = Vec::with_capacity(rbum_set_cate_sys_codes.len());
        for rbum_set_cate_sys_code in rbum_set_cate_sys_codes {
            if rbum_set_cate_sys_code.is_empty() {
                // Mount on the root node and directly give an empty array
                result.push(vec![]);
                continue;
            }
            let rbum_set_paths = RbumSetCateServ::find_rbums(
                &RbumSetCateFilterReq {
                    rel_rbum_set_id: Some(rbum_set_id.to_string()),
                    sys_codes: Some(vec![rbum_set_cate_sys_code]),
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndParent),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.sys_code, &b.sys_code))
            .map(|item| RbumSetPathResp {
                id: item.id,
                name: item.name,
                own_paths: item.own_paths,
            })
            .collect();
            result.push(rbum_set_paths);
        }
        Ok(result)
    }

    /// Check whether resource item a is the parent of resource item b in the specified resource set
    ///
    /// 检查在指定资源集中资源项a是否是资源项b的父级
    pub async fn check_a_is_parent_of_b(rbum_item_a_id: &str, rbum_item_b_id: &str, rbum_set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        Self::check_a_and_b(rbum_item_a_id, rbum_item_b_id, true, false, rbum_set_id, funs, ctx).await
    }

    /// Check whether resource item a is the sibling of resource item b in the specified resource set
    ///
    /// 检查在指定资源集中资源项a是否是资源项b的兄弟级
    pub async fn check_a_is_sibling_of_b(rbum_item_a_id: &str, rbum_item_b_id: &str, rbum_set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        Self::check_a_and_b(rbum_item_a_id, rbum_item_b_id, false, true, rbum_set_id, funs, ctx).await
    }

    /// Check whether resource item a is the parent or sibling of resource item b in the specified resource set
    ///
    /// 检查在指定资源集中资源项a是否是资源项b的父级或兄弟级
    pub async fn check_a_is_parent_or_sibling_of_b(
        rbum_item_a_id: &str,
        rbum_item_b_id: &str,
        rbum_set_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<bool> {
        Self::check_a_and_b(rbum_item_a_id, rbum_item_b_id, true, true, rbum_set_id, funs, ctx).await
    }

    async fn check_a_and_b(
        rbum_item_a_id: &str,
        rbum_item_b_id: &str,
        is_parent: bool,
        is_sibling: bool,
        rbum_set_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<bool> {
        let set_items: Vec<RbumSetItemSummaryResp> = Self::find_rbums(
            &RbumSetItemFilterReq {
                rel_rbum_set_id: Some(rbum_set_id.to_string()),
                rel_rbum_item_ids: Some(vec![rbum_item_a_id.to_string(), rbum_item_b_id.to_string()]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let set_item_a_sys_codes = set_items
            .iter()
            .filter(|item| item.rel_rbum_item_id == rbum_item_a_id)
            .map(|item| item.rel_rbum_set_cate_sys_code.clone().unwrap_or_default())
            .collect::<Vec<String>>();
        let set_item_b_sys_codes = set_items
            .iter()
            .filter(|item| item.rel_rbum_item_id == rbum_item_b_id)
            .map(|item| item.rel_rbum_set_cate_sys_code.clone().unwrap_or_default())
            .collect::<Vec<String>>();

        Ok(set_item_a_sys_codes.iter().any(|sys_code_a| {
            set_item_b_sys_codes.iter().any(|sys_code_b| {
                if is_parent && is_sibling {
                    sys_code_b.starts_with(sys_code_a)
                } else if is_parent {
                    sys_code_b.starts_with(sys_code_a) && sys_code_a != sys_code_b
                } else if is_sibling {
                    sys_code_a == sys_code_b
                } else {
                    // !is_parent && !is_sibling
                    sys_code_a != sys_code_b && !sys_code_b.starts_with(sys_code_a)
                }
            })
        }))
    }
}

#[derive(Debug, sea_orm::FromQueryResult)]
struct SysCodeResp {
    pub sys_code: String,
}
