use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::object_dto::{ObjectBatchBuildCreatePresignUrlReq, ObjectCompleteMultipartUploadReq, ObjectInitiateMultipartUploadReq, ObjectObjPresignKind};
use crate::{object_constants, object_initializer};

use super::s3;

pub async fn presign_obj_url(
    presign_kind: ObjectObjPresignKind,
    object_path: &str,
    max_width: Option<String>,
    max_height: Option<String>,
    exp_secs: u32,
    private: Option<bool>,
    special: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    let inst = funs.init(ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::presign_obj_url(
                presign_kind,
                object_path,
                max_width,
                max_height,
                exp_secs,
                private,
                special,
                funs,
                ctx,
                &inst,
            )
            .await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn initiate_multipart_upload(req: ObjectInitiateMultipartUploadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    let inst = funs.init(ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::initiate_multipart_upload(&req.object_path, req.content_type, req.private, req.special, funs, ctx, &inst).await
        }
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn batch_build_create_presign_url(req: ObjectBatchBuildCreatePresignUrlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    let inst = funs.init(ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::batch_build_create_presign_url(
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
    let inst = funs.init(ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => {
            s3::object_s3_obj_serv::complete_multipart_upload(
                &req.object_path,
                &req.upload_id,
                req.parts,
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

pub async fn object_delete(
    object_path: String,
    private: Option<bool>,
    special: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    let inst = funs.init(ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::object_delete(&object_path, private, special, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn object_copy(
    from: String,
    to: String,
    private: Option<bool>,
    special: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    let inst = funs.init(ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::object_copy(&from, &to, private, special, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
