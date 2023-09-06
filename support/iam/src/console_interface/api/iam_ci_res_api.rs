use crate::basic::dto::iam_res_dto::IamResAggAddReq;
use crate::basic::dto::iam_set_dto::IamSetCateAddReq;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use bios_basic::rbum::rbum_config::RbumConfigApi;

use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::TardisFuns;
#[derive(Clone, Default)]
pub struct IamCiResApi;

/// # Interface Console Manage Cert API
///
/// Allow Management Of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/res", tag = "bios_basic::ApiTag::Interface")]
impl IamCiResApi {
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamResAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamResServ::add_res_agg(&mut add_req.0, &set_id, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Res Cate
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let api_sys_codes = TardisFuns::field.incr_by_base36(&String::from_utf8(vec![b'0'; set_cate_sys_code_node_len]).unwrap_or_default()).map(|api_sys_code| vec![api_sys_code]);
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let rbum_parent_cate_id = if add_req.0.rbum_parent_cate_id.is_none() {
            Some(IamSetServ::get_cate_id_with_sys_code(set_id.as_str(), api_sys_codes, &funs, &ctx.0).await?)
        } else {
            add_req.0.rbum_parent_cate_id
        };
        let result = IamSetServ::add_set_cate(
            &set_id,
            &IamSetCateAddReq {
                name: add_req.0.name,
                scope_level: add_req.0.scope_level,
                bus_code: add_req.0.bus_code,
                icon: add_req.0.icon,
                sort: add_req.0.sort,
                ext: add_req.0.ext,
                rbum_parent_cate_id,
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
