use bios_basic::spi::{serv::spi_bs_serv::SpiBsServ, spi_funs::SpiBsInst, spi_initializer::common};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    os::os_client::TardisOSClient,
    TardisFunsInst,
};

use crate::dto::object_dto::ObjectObjPresignKind;

pub async fn presign_obj_url(
    presign_kind: ObjectObjPresignKind,
    specified_bucket_name: Option<String>,
    object_path: &str,
    _max_width: Option<String>,
    _max_height: Option<String>,
    exp_secs: u32,
    private: Option<bool>,
    special: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<String> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(specified_bucket_name, private, special, inst);
    match presign_kind {
        ObjectObjPresignKind::Upload => client.object_create_url(object_path, exp_secs, bucket_name.as_deref()).await,
        ObjectObjPresignKind::Delete => client.object_delete_url(object_path, exp_secs, bucket_name.as_deref()).await,
        ObjectObjPresignKind::View => {
            if private.unwrap_or(true) {
                client.object_get_url(object_path, exp_secs, bucket_name.as_deref()).await
            } else {
                let spi_bs = SpiBsServ::get_bs_by_rel(&ctx.ak, None, funs, ctx).await?;
                let Some(bucket_name) = bucket_name else {
                    return Err(TardisError::internal_error(
                        "Cannot get public bucket name while presign object url, it may due to the lack of isolation_flag",
                        "500-spi-object-s3-cannot-get-bucket-name",
                    ));
                };
                Ok(format!("{}/{}/{}", spi_bs.conn_uri, bucket_name, object_path))
            }
        }
    }
}

pub async fn object_delete(
    specified_bucket_name: Option<String>,
    object_path: &str,
    private: Option<bool>,
    special: Option<bool>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(specified_bucket_name, private, special, inst);
    client.object_delete(object_path, bucket_name.as_deref()).await
}

pub async fn object_copy(
    specified_bucket_name: Option<String>,
    from: &str,
    to: &str,
    private: Option<bool>,
    special: Option<bool>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(specified_bucket_name, private, special, inst);
    client.object_copy(from, to, bucket_name.as_deref()).await
}

pub async fn initiate_multipart_upload(
    specified_bucket_name: Option<String>,
    object_path: &str,
    content_type: Option<String>,
    private: Option<bool>,
    special: Option<bool>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<String> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(specified_bucket_name, private, special, inst);
    client.initiate_multipart_upload(object_path, content_type.as_deref(), bucket_name.as_deref()).await
}

pub async fn batch_build_create_presign_url(
    specified_bucket_name: Option<String>,
    object_path: &str,
    upload_id: &str,
    part_number: u32,
    expire_sec: u32,
    private: Option<bool>,
    special: Option<bool>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<String>> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(specified_bucket_name, private, special, inst);
    client.batch_build_create_presign_url(object_path, upload_id, part_number, expire_sec, bucket_name.as_deref()).await
}

// pub async fn complete_multipart_upload(&self, path: &str, upload_id: &str, parts: Vec<String>, bucket_name: Option<&str>) -> TardisResult<()> {
//     trace!("[Tardis.OSClient] Complete multipart upload {}", path);
//     self.get_client().complete_multipart_upload(path, upload_id, parts, bucket_name).await
// }
pub async fn complete_multipart_upload(
    specified_bucket_name: Option<String>,
    object_path: &str,
    upload_id: &str,
    parts: Vec<String>,
    private: Option<bool>,
    special: Option<bool>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(specified_bucket_name, private, special, inst);
    client.complete_multipart_upload(object_path, upload_id, parts, bucket_name.as_deref()).await
}

fn get_bucket_name(specified_bucket_name: Option<String>, private: Option<bool>, special: Option<bool>, inst: &SpiBsInst) -> Option<String> {
    let bs_inst = inst.inst::<TardisOSClient>();
    if let Some(specified_bucket_name) = specified_bucket_name {
        Some(specified_bucket_name)
    } else {
        common::get_isolation_flag_from_ext(bs_inst.1).map(|bucket_name_prefix| {
            format!(
                "{}-{}",
                bucket_name_prefix,
                if special.unwrap_or(false) {
                    "spe"
                } else if private.unwrap_or(true) {
                    "pri"
                } else {
                    "pub"
                }
            )
        })
    }
}
