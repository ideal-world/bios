use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::serv::iam_sub_deploy_serv::IamSubDeployServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;

#[derive(Clone, Default)]
pub struct IamCcSubDeployApi;

/// Common Console Sub Deploy API
/// 通用控制台二级部署API
#[poem_openapi::OpenApi(prefix_path = "/cc/sub_deploy", tag = "bios_basic::ApiTag::Common")]
impl IamCcSubDeployApi {
    /// Exist Sub Deploy Rel Account By Account Ids
    ///
    /// 二级部署关联账号列表是否存在
    #[oai(path = "/account/exist", method = "get")]
    async fn exist_rel_account_by_account_ids(&self, account_ids: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, bool>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        let mut result = HashMap::new();
        for s in split {
            let exist = IamSubDeployServ::exist_to_rel(&IamRelKind::IamSubDeployAccount, s, &funs, &ctx.0).await?;
            result.insert(s.to_string(), exist);
        }
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/auth/account/exist", method = "get")]
    async fn exist_rel_auth_account_by_account_ids(&self, account_ids: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, bool>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        let mut result = HashMap::new();
        for s in split {
            let exist = IamSubDeployServ::exist_to_rel(&IamRelKind::IamSubDeployAuthAccount, s, &funs, &ctx.0).await?;
            result.insert(s.to_string(), exist);
        }
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Exist Sub Deploy Rel Apps By Org Ids
    ///
    /// 二级部署关联组织列表是否存在
    #[oai(path = "/org/exist", method = "get")]
    async fn exist_rel_org_by_org_ids(&self, org_ids: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, bool>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let split = org_ids.0.split(',').collect::<Vec<_>>();
        let mut result = HashMap::new();
        for s in split {
            let exist = IamSubDeployServ::exist_to_rel(&IamRelKind::IamSubDeployOrg, s, &funs, &ctx.0).await?;
            result.insert(s.to_string(), exist);
        }
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Exist Sub Deploy Rel Apps By Apps Ids
    ///
    /// 二级部署关联项目组列表是否存在
    #[oai(path = "/apps/exist", method = "get")]
    async fn exist_rel_apps_by_apps_ids(&self, apps_ids: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, bool>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let split = apps_ids.0.split(',').collect::<Vec<_>>();
        let mut result = HashMap::new();
        for s in split {
            let exist = IamSubDeployServ::exist_to_rel(&IamRelKind::IamSubDeployApps, s, &funs, &ctx.0).await?;
            result.insert(s.to_string(), exist);
        }
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
