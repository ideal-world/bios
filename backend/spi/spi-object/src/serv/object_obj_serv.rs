use std::collections::HashMap;
use std::sync::Arc;

use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::tokio::sync::RwLock;
use tardis::TardisFunsInst;

use crate::dto::object_dto::{ObjectBatchBuildCreatePresignUrlReq, ObjectCompleteMultipartUploadReq, ObjectInitiateMultipartUploadReq, ObjectObjPresignKind};
use crate::object_constants::USE_REGION_ENDPOINT;
use crate::{object_constants, object_initializer};

use super::s3::S3 as _;
use super::{obs, s3};

pub async fn presign_obj_url(
    presign_kind: ObjectObjPresignKind,
    object_path: &str,
    max_width: Option<String>,
    max_height: Option<String>,
    exp_secs: u32,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::S3Service::presign_obj_url(presign_kind, object_path, max_width, max_height, exp_secs, private, special, obj_exp, funs, ctx, &inst).await
        }
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => {
            obs::object_obs_obj_serv::OBSService::presign_obj_url(presign_kind, object_path, max_width, max_height, exp_secs, private, special, obj_exp, funs, ctx, &inst).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn batch_get_presign_obj_url(
    object_paths: Vec<String>,
    exp_secs: u32,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<HashMap<String, String>> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::S3Service::batch_get_presign_obj_url(object_paths, exp_secs, private, special, obj_exp, funs, ctx, &inst).await
        }
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => {
            obs::object_obs_obj_serv::OBSService::batch_get_presign_obj_url(object_paths, exp_secs, private, special, obj_exp, funs, ctx, &inst).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn initiate_multipart_upload(req: ObjectInitiateMultipartUploadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::S3Service::initiate_multipart_upload(&req.object_path, req.content_type, req.private, req.special, funs, ctx, &inst).await
        }
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => {
            obs::object_obs_obj_serv::OBSService::initiate_multipart_upload(&req.object_path, req.content_type, req.private, req.special, funs, ctx, &inst).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn batch_build_create_presign_url(req: ObjectBatchBuildCreatePresignUrlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::S3Service::batch_build_create_presign_url(
                &req.object_path,
                &req.upload_id,
                req.part_number,
                req.expire_sec,
                req.private,
                req.special,
                funs,
                ctx,
                &inst,
            )
            .await
        }
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => {
            obs::object_obs_obj_serv::OBSService::batch_build_create_presign_url(
                &req.object_path,
                &req.upload_id,
                req.part_number,
                req.expire_sec,
                req.private,
                req.special,
                funs,
                ctx,
                &inst,
            )
            .await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn complete_multipart_upload(req: ObjectCompleteMultipartUploadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::S3Service::complete_multipart_upload(&req.object_path, &req.upload_id, req.parts, req.private, req.special, funs, ctx, &inst).await
        }
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => {
            obs::object_obs_obj_serv::OBSService::complete_multipart_upload(&req.object_path, &req.upload_id, req.parts, req.private, req.special, funs, ctx, &inst).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn object_delete(
    object_path: String,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::S3Service::object_delete(&object_path, private, special, obj_exp, funs, ctx, &inst).await,
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => obs::object_obs_obj_serv::OBSService::object_delete(&object_path, private, special, obj_exp, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn batch_object_delete(
    object_paths: Vec<String>,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Vec<String>> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::S3Service::batch_object_delete(object_paths, private, special, obj_exp, funs, ctx, &inst).await,
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => obs::object_obs_obj_serv::OBSService::batch_object_delete(object_paths, private, special, obj_exp, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn object_copy(from: String, to: String, private: Option<bool>, special: Option<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let mock_ctx = TardisContext {
        ext: Arc::new(RwLock::new(HashMap::from([(USE_REGION_ENDPOINT.to_string(), "true".to_string())]))),
        ..ctx.clone()
    };
    let inst = funs.init(Some(USE_REGION_ENDPOINT.to_string()), ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::S3Service::object_copy(&from, &to, private, special, funs, ctx, &inst).await,
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => {
            obs::object_obs_obj_serv::OBSService::object_copy(&from, &to, private, special, funs, &mock_ctx, &inst).await
        },
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn object_exist(
    object_paths: String,
    private: Option<bool>,
    special: Option<bool>,
    obj_exp: Option<u32>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<bool> {
    let inst = funs.init(None, ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::S3Service::object_exist(&object_paths, private, special, obj_exp, funs, ctx, &inst).await,
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_OBS_KIND_CODE => obs::object_obs_obj_serv::OBSService::object_exist(&object_paths, private, special, obj_exp, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
