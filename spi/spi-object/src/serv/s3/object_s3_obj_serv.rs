use bios_basic::spi::{serv::spi_bs_serv::SpiBsServ, spi_funs::SpiBsInstExtractor, spi_initializer::common};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    os::os_client::TardisOSClient,
    TardisFunsInst,
};

use crate::dto::object_dto::ObjectObjPresignKind;

pub async fn presign_obj_url(
    presign_kind: ObjectObjPresignKind,
    object_path: &str,
    _max_width: Option<String>,
    _max_height: Option<String>,
    exp_secs: u32,
    private: bool,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisOSClient>();
    let spi_bs = SpiBsServ::get_bs_by_rel(&ctx.owner, None, funs, ctx).await?;
    let client = bs_inst.0;
    let bucket_name = common::get_isolation_flag_from_ext(bs_inst.1).map(|bucket_name_prefix| format!("{}-{}", bucket_name_prefix, if private { "pri" } else { "pub" }));
    match presign_kind {
        ObjectObjPresignKind::Upload => client.object_create_url(object_path, exp_secs, bucket_name),
        ObjectObjPresignKind::Delete => client.object_delete_url(object_path, exp_secs, bucket_name),
        ObjectObjPresignKind::View => {
            if private {
                client.object_get_url(object_path, exp_secs, bucket_name)
            } else {
                Ok(format!("{}/{}/{}", spi_bs.conn_uri, bucket_name.unwrap(), object_path))
            }
        }
    }
}
