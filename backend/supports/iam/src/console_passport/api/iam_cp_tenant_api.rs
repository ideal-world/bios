use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantBoneResp, IamTenantKeyNameResp};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCpTenantApi;

/// Passport Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/cp/tenant", tag = "bios_basic::ApiTag::Passport")]
impl IamCpTenantApi {
    /// Get All Tenants
    /// 获取所有租户
    #[oai(path = "/all", method = "get")]
    async fn find(&self) -> TardisApiResult<Vec<IamTenantBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::find_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &IamCertServ::get_anonymous_context(),
        )
        .await?;
        let result = result
            .into_iter()
            .map(|i| IamTenantBoneResp {
                id: i.id,
                name: i.name,
                icon: i.icon,
            })
            .collect();
        TardisResp::ok(result)
    }

    /// Get All Tenants Return Kv
    /// 查找租户
    #[oai(path = "/all/key-name", method = "get")]
    async fn key_name(&self) -> TardisApiResult<Vec<IamTenantKeyNameResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::find_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &IamCertServ::get_anonymous_context(),
        )
        .await?;
        let result = result
            .into_iter()
            .map(|i| IamTenantKeyNameResp {
                key: i.id,
                name: i.name,
            })
            .collect();
        TardisResp::ok(result)
    }
}
