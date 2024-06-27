use std::collections::HashMap;

use bios_basic::spi::{serv::spi_bs_serv::SpiBsServ, spi_funs::SpiBsInst, spi_initializer::common};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    futures::future::join_all,
    os::{
        os_client::TardisOSClient,
        serde_types::{BucketLifecycleConfiguration, Expiration, LifecycleFilter, LifecycleRule},
    },
    TardisFunsInst,
};

use crate::dto::object_dto::ObjectObjPresignKind;
///
/// obj_exp: 设置obj的过期时间 单位为天
pub async fn presign_obj_url(
    presign_kind: ObjectObjPresignKind,
    object_path: &str,
    _max_width: Option<String>,
    _max_height: Option<String>,
    exp_secs: u32,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<String> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(private, special, obj_exp.map(|_| true), inst);
    match presign_kind {
        ObjectObjPresignKind::Upload => {
            client.object_create_url(&rebuild_path(bucket_name.as_deref(), object_path, obj_exp, client).await?, exp_secs, bucket_name.as_deref()).await
        }
        ObjectObjPresignKind::Delete => {
            client.object_delete_url(&rebuild_path(bucket_name.as_deref(), object_path, obj_exp, client).await?, exp_secs, bucket_name.as_deref()).await
        }
        ObjectObjPresignKind::View => {
            if private.unwrap_or(true) || special.unwrap_or(false) {
                client.object_get_url(&rebuild_path(bucket_name.as_deref(), object_path, obj_exp, client).await?, exp_secs, bucket_name.as_deref()).await
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
    object_path: &str,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(private, special, obj_exp.map(|_| true), inst);
    client.object_delete(object_path, bucket_name.as_deref()).await
}

pub async fn batch_object_delete(
    object_paths: Vec<String>,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<String>> {
    let failed_object_paths = join_all(
        object_paths
            .into_iter()
            .map(|object_path| async move {
                let result = object_delete(&object_path, private, special, obj_exp, _funs, _ctx, inst).await;
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
    let bucket_name = get_bucket_name(private, special, None, inst);
    client.object_copy(from, to, bucket_name.as_deref()).await
}

pub async fn object_exist(
    object_path: &str,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<bool> {
    let bs_inst = inst.inst::<TardisOSClient>();
    let client = bs_inst.0;
    let bucket_name = get_bucket_name(private, special, obj_exp.map(|_| true), inst);
    client.object_exist(object_path, bucket_name.as_deref()).await
}

pub async fn batch_get_presign_obj_url(
    object_paths: Vec<String>,
    exp_secs: u32,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    _funs: &TardisFunsInst,
    _ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<HashMap<String, String>> {
    let result = join_all(
        object_paths
            .into_iter()
            .map(|object_path| async move {
                let result = presign_obj_url(ObjectObjPresignKind::View, &object_path, None, None, exp_secs, private, special, obj_exp, _funs, _ctx, inst).await;
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
    let bucket_name = get_bucket_name(private, special, None, inst);
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
    let bucket_name = get_bucket_name(private, special, None, inst);
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
    let bucket_name = get_bucket_name(private, special, None, inst);
    client.complete_multipart_upload(object_path, upload_id, parts, bucket_name.as_deref()).await
}

fn get_bucket_name(private: Option<bool>, special: Option<bool>, tamp: Option<bool>, inst: &SpiBsInst) -> Option<String> {
    let bs_inst = inst.inst::<TardisOSClient>();
    common::get_isolation_flag_from_ext(bs_inst.1).map(|bucket_name_prefix| {
        format!(
            "{}-{}",
            bucket_name_prefix,
            if special.unwrap_or(false) {
                "spe"
            } else if tamp.unwrap_or(false) {
                "tamp"
            } else if private.unwrap_or(true) {
                "pri"
            } else {
                "pub"
            }
        )
    })
}
async fn rebuild_path(bucket_name: Option<&str>, origin_path: &str, obj_exp: Option<u32>, client: &TardisOSClient) -> TardisResult<String> {
    if let Some(obj_exp) = obj_exp {
        let resp = client.get_lifecycle(bucket_name).await;
        match resp {
            Ok(config) => {
                let mut rules = config.rules;
                let prefix = if let Some(is_have_prefix) = rules
                    .iter()
                    .filter(|r| r.status == *"Enabled" && r.expiration.clone().is_some_and(|exp| exp.days.is_some_and(|days| days == obj_exp)))
                    .filter_map(|r| r.filter.clone())
                    .find_map(|f| f.prefix)
                {
                    is_have_prefix
                } else {
                    let rand_id = tardis::rand::random::<usize>().to_string();
                    let prefix = format!("{}/", rand_id);
                    //add rule
                    let add_rule = LifecycleRule::builder("Enabled")
                        .id(&rand_id)
                        .expiration(Expiration::new(None, Some(obj_exp), None))
                        .filter(LifecycleFilter::new(None, None, None, Some(prefix.clone()), None))
                        .build();
                    rules.push(add_rule);
                    client.put_lifecycle(bucket_name, BucketLifecycleConfiguration::new(rules)).await?;
                    prefix
                };
                Ok(format!("{}{}", prefix, origin_path))
            }
            Err(e) => {
                if e.code != "404" {
                    return Err(TardisError::internal_error(&format!("Bucket {:?} get lifecycle failed", bucket_name), &format!("{:?}", e)));
                }
                let mut rules = vec![];
                let rand_id = tardis::rand::random::<usize>().to_string();
                let prefix = format!("{}/", rand_id);
                //add rule
                let add_rule = LifecycleRule::builder("Enabled")
                    .id(&rand_id)
                    .expiration(Expiration::new(None, Some(obj_exp), None))
                    .filter(LifecycleFilter::new(None, None, None, Some(prefix.clone()), None))
                    .build();
                rules.push(add_rule);
                client.put_lifecycle(bucket_name, BucketLifecycleConfiguration::new(rules)).await?;
                Ok(format!("{}{}", prefix, origin_path))
            }
        }
    } else {
        Ok(origin_path.to_string())
    }
}
