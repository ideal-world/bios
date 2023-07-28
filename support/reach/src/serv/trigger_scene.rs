use crate::domain::trigger_scene;
use crate::dto::*;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFunsInst, TardisFuns};



pub struct ReachTriggerSceneService;

#[async_trait]
impl
    RbumCrudOperation<
        trigger_scene::ActiveModel,
        ReachTriggerSceneAddReq,
        ReachTriggerSceneModifyReq,
        ReachTriggerSceneSummaryResp,
        ReachTriggerSceneDetailResp,
        ReachTriggerSceneFilterReq,
    > for ReachTriggerSceneService
{
    fn get_table_name() -> &'static str {
        trigger_scene::Entity.table_name()
    }
    async fn package_add(add_req: &ReachTriggerSceneAddReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<trigger_scene::ActiveModel> {
        let mut model = trigger_scene::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn before_add_rbum(add_req: &mut ReachTriggerSceneAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(pid) = &add_req.pid {
            if !pid.trim().is_empty() {
                return Self::check_ownership(pid, funs, ctx).await;
            }
        }
        add_req.pid = Some(String::default());
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachTriggerSceneModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<trigger_scene::ActiveModel> {
        let mut model = trigger_scene::ActiveModel::from(modify_req);
        model.fill_ctx(ctx, true);
        model.id = Set(id.into());
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachTriggerSceneFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        if let Some(code) = &filter.code {
            query.and_where(trigger_scene::Column::Code.starts_with(code));
        }
        if let Some(name) = &filter.name {
            query.and_where(trigger_scene::Column::Name.eq(name));
        }
        query.with_filter(Self::get_table_name(), &filter.base_filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

macro_rules! add_scene {
    // empty
    () => {};
    // for tree
    (
        $funs: expr;
        $ctx: expr;
        @tree_or_item $($p: expr),* $(=> {$(
            $($args: expr),*$( => $subs: tt)?;
        )*})?
    ) => {
        {
            let parent_id = add_scene!(
                $funs;
                $ctx;
                @item $($p),*
            );
            $(
                $(
                    add_scene!(
                        $funs;
                        $ctx;
                        @tree_or_item $($args),*, &parent_id $( => $subs)?
                    );
                )*
            )?
            parent_id
        }
    };
    // for single item
    (
        $funs: expr;
        $ctx: expr;
        @item $code: expr, $name: expr $(, $pid: expr)?
    ) => {
        Self::add_rbum(&mut ReachTriggerSceneAddReq::new_with_name_code($name, $code)$(.pid($pid))?, $funs, $ctx).await?
    };
    // enter
    (
        $funs: expr;
        $ctx: expr;
        $($tt: tt)*
    ) => {
        add_scene!(
            $funs;
            $ctx;
            @tree_or_item $($tt)*
        )
    };
}

impl ReachTriggerSceneService {
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let count = funs.db().count(Query::select().from(trigger_scene::Entity)).await?;
        if count > 0 {
            return Ok(());
        }
        Self::init_doc(funs, ctx).await?;
        Self::init_app(funs, ctx).await?;
        Ok(())
    }
    async fn init_doc(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        add_scene! {
            funs;
            ctx;
            "doc", "知识库" => {
                "doc_set_create", "创建知识库";
                "doc_set_delete", "删除知识库";
                "doc_content_create", "创建知识库内容";
                "doc_content_delete", "删除知识库内容";
            }
        };
        Ok(())
    }
    async fn init_app(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        add_scene! {
            funs;
            ctx;
            "app", "项目" => {
                "app_milestone", "里程碑" => {
                    "app_milestone_create", "创建里程碑";
                    "app_milestone_delete", "删除里程碑";
                };
                "app_need", "需求" => {
                    "app_need_create", "创建需求";
                    "app_need_change", "变更需求";
                    "app_need_change_status", "变更需求状态";
                    "app_need_delete", "删除需求";
                };
                "app_task", "任务" => {
                    "app_task_iterate_delete", "删除迭代";
                    "app_task_create", "创建任务";
                    "app_task_delete", "删除任务";
                };
                "app_iterate", "迭代" => {
                    "app_iterate_create", "创建迭代";
                    "app_iterate_change_status", "变更迭代状态";
                    "app_iterate_delete", "删除迭代";
                };
                "app_develop", "开发" => {
                    "app_develop_project_create", "创建项目";
                    "app_develop_project_delete", "删除项目";
                    "app_develop_project_branch_create", "新增工程分支";
                    "app_develop_project_branch_delete", "删除工程分支";
                    "app_develop_env_create", "新建环境";
                };
            }
        };
        Ok(())
    }
}
