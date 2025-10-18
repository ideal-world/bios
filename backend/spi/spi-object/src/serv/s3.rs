pub mod object_s3_initializer;
pub mod object_s3_obj_serv;

use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, serv::spi_bs_serv::SpiBsServ, spi_funs::SpiBsInst, spi_initializer::common};
use itertools::Itertools;
use tardis::{
    TardisFunsInst, basic::{dto::TardisContext, error::TardisError, result::TardisResult}, futures::future::join_all, os::os_client::TardisOSClient, web::poem::http::{HeaderMap, HeaderValue}
};

use crate::dto::object_dto::ObjectObjPresignKind;

pub trait S3 {
    ///
    /// obj_exp: 设置obj的过期时间 单位为天
    #[allow(clippy::too_many_arguments)]
    async fn presign_obj_url(
        presign_kind: ObjectObjPresignKind,
        object_path: &str,
        _max_width: Option<String>,
        _max_height: Option<String>,
        exp_secs: u32,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<String> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, obj_exp.map(|_| true), bucket, bs_id, inst);
        let path = Self::rebuild_path(bucket_name.as_deref(), object_path, obj_exp, client).await?;
        match presign_kind {
            ObjectObjPresignKind::Upload => {
                let headers = obj_exp.map(|o| -> TardisResult<HeaderMap> {
                    let mut headers = HeaderMap::new();
                    headers.insert("Expires", HeaderValue::from_str(&o.to_string())
                        .map_err(|_| TardisError::internal_error("Cannot convert expires to header value", "500-spi-object-invalid-header-value"))?);
                    Ok(headers)
                }).transpose()?;
                client.object_create_url(&path, exp_secs, bucket_name.as_deref(), headers, None).await
            },
            ObjectObjPresignKind::Delete => client.object_delete_url(&path, exp_secs, bucket_name.as_deref()).await,
            ObjectObjPresignKind::View => {
                if private.unwrap_or(true) || special.unwrap_or(false) {
                    client.object_get_url(&path, exp_secs, bucket_name.as_deref(), None).await
                } else {
                    let spi_bs = if let Some(bs_id) = bs_id {
                        SpiBsServ::get_bs(bs_id, funs, ctx).await.map(|spi| SpiBsCertResp {
                            kind_code: spi.kind_code.clone(),
                            conn_uri: spi.conn_uri.clone(),
                            ak: spi.ak.clone(),
                            sk: spi.sk.clone().unwrap_or_default(),
                            ext: spi.ext.clone(),
                            private: spi.private,
                        })?
                    } else {
                        SpiBsServ::get_bs_by_rel(&ctx.ak, None, funs, ctx).await?
                    };
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

    async fn object_delete(
        object_path: &str,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<()> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, obj_exp.map(|_| true), bucket, bs_id, inst);
        let path = Self::rebuild_path(bucket_name.as_deref(), object_path, obj_exp, client).await?;
        client.object_delete(&path, bucket_name.as_deref()).await
    }

    async fn batch_object_delete(
        object_paths: Vec<String>,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<Vec<String>> {
        let failed_object_paths = join_all(
            object_paths
                .into_iter()
                .map(|object_path| async move {
                    let result = Self::object_delete(&object_path, private, special, obj_exp, bs_id, bucket, _funs, _ctx, inst).await;
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

    async fn object_copy(
        from: &str,
        to: &str,
        private: Option<bool>,
        special: Option<bool>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<()> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, None, bucket, bs_id, inst);
        client.object_copy(from, to, bucket_name.as_deref()).await
    }

    async fn object_exist(
        object_path: &str,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<bool> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, obj_exp.map(|_| true), bucket, bs_id, inst);
        let path = Self::rebuild_path(bucket_name.as_deref(), object_path, obj_exp, client).await?;
        client.object_exist(&path, bucket_name.as_deref()).await
    }

    async fn batch_get_presign_obj_url(
        object_paths: Vec<String>,
        exp_secs: u32,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<HashMap<String, String>> {
        let result = join_all(
            object_paths
                .into_iter()
                .map(|object_path| async move {
                    let result = Self::presign_obj_url(
                        ObjectObjPresignKind::View,
                        &object_path,
                        None,
                        None,
                        exp_secs,
                        private,
                        special,
                        obj_exp,
                        bs_id,
                        bucket,
                        _funs,
                        _ctx,
                        inst,
                    )
                    .await;
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

    async fn initiate_multipart_upload(
        object_path: &str,
        content_type: Option<String>,
        private: Option<bool>,
        special: Option<bool>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<String> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, None, bucket, bs_id, inst);
        client.initiate_multipart_upload(object_path, content_type.as_deref(), bucket_name.as_deref()).await
    }

    async fn batch_build_create_presign_url(
        object_path: &str,
        upload_id: &str,
        part_number: u32,
        expire_sec: u32,
        private: Option<bool>,
        special: Option<bool>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<Vec<String>> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, None, bucket, bs_id, inst);
        client.batch_build_create_presign_url(object_path, upload_id, part_number, expire_sec, bucket_name.as_deref()).await
    }

    async fn complete_multipart_upload(
        object_path: &str,
        upload_id: &str,
        parts: Vec<String>,
        private: Option<bool>,
        special: Option<bool>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<()> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, None, bucket, bs_id, inst);
        client.complete_multipart_upload(object_path, upload_id, parts, bucket_name.as_deref()).await
    }

    fn get_bucket_name(private: Option<bool>, special: Option<bool>, tamp: Option<bool>, bucket_name: Option<&str>, bs_id: Option<&str>, inst: &SpiBsInst) -> Option<String> {
        // 使用自定义客户端时，不需要遵循内置桶的规则，直接返回传入的桶名
        if bs_id.is_some() {
            return bucket_name.map(|s| s.to_string());
        }
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

    async fn rebuild_path(bucket_name: Option<&str>, origin_path: &str, obj_exp: Option<u32>, client: &TardisOSClient) -> TardisResult<String>;
}
