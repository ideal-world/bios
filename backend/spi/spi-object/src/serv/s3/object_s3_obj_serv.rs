use std::collections::HashMap;

use bios_basic::spi::{serv::spi_bs_serv::SpiBsServ, spi_funs::SpiBsInst, spi_initializer::common};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    futures::future::join_all,
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
    private: Option<bool>,
    special: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<String> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(private, special, inst);
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

pub async fn object_delete(object_path: &str, private: Option<bool>, special: Option<bool>, _funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(private, special, inst);
    client.object_delete(object_path, bucket_name.as_deref()).await
}

pub async fn batch_object_delete(
    object_paths: Vec<&str>,
    private: Option<bool>,
    special: Option<bool>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<String>> {
    let failed_object_paths = join_all(
        object_paths
            .into_iter()
            .map(|object_path| async {
                let result = object_delete(object_path, private, special, _funs, _ctx, inst).await;
                if result.is_err() {
                    object_path.to_string()
                } else {
                    "".to_string()
                }
            })
            .collect_vec(),
    )
    .await;
    Ok(failed_object_paths.into_iter().filter(|object_path| !object_path.is_empty()).collect_vec())
}

pub async fn object_copy(from: &str, to: &str, private: Option<bool>, special: Option<bool>, _funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(private, special, inst);
    client.object_copy(from, to, bucket_name.as_deref()).await
}

pub async fn object_exist(object_path: &str, private: Option<bool>, special: Option<bool>, _funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<bool> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(private, special, inst);
    let result = client.object_get(object_path, bucket_name.as_deref()).await;
    if result.is_err() && result.clone().expect_err("unreachable").code == "404" {
        Ok(false)
    } else if result.is_ok() {
        Ok(true)
    } else {
        result.map(|_| true)
    }
}

pub async fn batch_get_presign_obj_url(
    object_paths: Vec<&str>,
    exp_secs: u32,
    private: Option<bool>,
    special: Option<bool>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<HashMap<String, String>> {
    let result = join_all(
        object_paths
            .into_iter()
            .map(|object_path| async move {
                let result = presign_obj_url(ObjectObjPresignKind::View, object_path, None, None, exp_secs, private, special, _funs, _ctx, inst).await;
                if let Ok(presign_obj_url) = result {
                    (object_path.to_string(), presign_obj_url)
                } else {
                    ("".to_string(), "".to_string())
                }
            })
            .collect_vec(),
    )
    .await;
    Ok(result.into_iter().filter(|(object_path, _)| !object_path.is_empty()).collect::<HashMap<_, _>>())
}

pub async fn initiate_multipart_upload(
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
    let bucket_name = get_bucket_name(private, special, inst);
    client.initiate_multipart_upload(object_path, content_type.as_deref(), bucket_name.as_deref()).await
}

pub async fn batch_build_create_presign_url(
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
    let bucket_name = get_bucket_name(private, special, inst);
    client.batch_build_create_presign_url(object_path, upload_id, part_number, expire_sec, bucket_name.as_deref()).await
}

pub async fn complete_multipart_upload(
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
    let bucket_name = get_bucket_name(private, special, inst);
    client.complete_multipart_upload(object_path, upload_id, parts, bucket_name.as_deref()).await
}

fn get_bucket_name(private: Option<bool>, special: Option<bool>, inst: &SpiBsInst) -> Option<String> {
    let bs_inst = inst.inst::<TardisOSClient>();
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
