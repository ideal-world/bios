use std::collections::HashSet;

use bios_basic::rbum::{
    domain::{rbum_kind_attr, rbum_rel, rbum_rel_attr},
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumRelExtFilterReq, RbumRelFilterReq},
        rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAggResp, RbumRelAttrAggAddReq},
        rbum_rel_attr_dto::RbumRelAttrDetailResp,
        rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp, RbumRelDetailResp, RbumRelSimpleFindReq},
    },
    helper::secret_helper,
    rbum_enumeration::RbumRelFromKind,
    serv::{
        rbum_crud_serv::RbumCrudOperation,
        rbum_rel_serv::{RbumRelAttrServ, RbumRelEnvServ, RbumRelServ},
    },
};
use bios_basic::spi::spi_constants;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::sea_query::{Expr, Query, SelectStatement},
    db::sea_orm::{self, Set},
    log::error,
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{dto::plugin_bs_dto::PluginRelSecretMigrateResp, plugin_enumeration::PluginAppBindRelKind};
pub struct PluginRelServ;

impl PluginRelServ {
    pub async fn add_simple_rel(
        tag: &PluginAppBindRelKind,
        from_rbum_id: &str,
        to_rbum_item_id: &str,
        note: Option<String>,
        ext: Option<String>,
        ignore_exist: bool,
        to_is_outside: bool,
        attrs: Option<Vec<RbumRelAttrAggAddReq>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if !ignore_exist {
            if Self::exist_rels(tag, from_rbum_id, to_rbum_item_id, funs, ctx).await? {
                return Ok(());
            }
        }
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: tag.to_string(),
                note,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_rbum_id.to_string(),
                to_rbum_item_id: to_rbum_item_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside,
                ext,
                disabled: None,
            },
            attrs: attrs.unwrap_or(vec![]),
            envs: vec![],
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        Ok(())
    }

    pub async fn delete_simple_rel(tag: &PluginAppBindRelKind, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_rbum_id.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if rel_ids.is_empty() {
            return Ok(());
        }
        for rel_id in rel_ids {
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, ctx).await?;
        }

        Ok(())
    }

    pub async fn delete_simple_rel_by_id(
        id: &str,
        tag: &PluginAppBindRelKind,
        from_rbum_id: &str,
        to_rbum_item_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ids: Some(vec![id.to_string()]),
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_rbum_id.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if rel_ids.is_empty() {
            return Ok(());
        }
        for rel_id in rel_ids {
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, ctx).await?;
        }

        Ok(())
    }

    pub async fn exist_to_simple_rels(tag: &PluginAppBindRelKind, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        RbumRelServ::check_simple_rel(
            &RbumRelSimpleFindReq {
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn exist_rels(tag: &PluginAppBindRelKind, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        RbumRelServ::check_simple_rel(
            &RbumRelSimpleFindReq {
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_rbum_id.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
                from_own_paths: Some(ctx.own_paths.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_to_simple_rels(
        tag: &PluginAppBindRelKind,
        to_rbum_item_id: &str,
        ext: Option<String>,
        with: bool,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_simple_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_owned()),
                ext_eq: ext,
                disabled: Some(false),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            false,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_from_simple_rels(
        tag: &PluginAppBindRelKind,
        from_rbum_kind: &RbumRelFromKind,
        from_rbum_id: &str,
        ext: Option<String>,
        with: bool,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_simple_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_owned()),
                ext_eq: ext,
                disabled: Some(false),
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            true,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_from_rels(
        tag: &PluginAppBindRelKind,
        from_rbum_kind: &RbumRelFromKind,
        from_rbum_id: &str,
        ext: Option<String>,
        with: bool,
        is_hide_secret: bool,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        if is_hide_secret {
            RbumRelServ::find_hide_secret_rels(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: with,
                        ..Default::default()
                    },
                    tag: Some(tag.to_string()),
                    from_rbum_kind: Some(from_rbum_kind.clone()),
                    from_rbum_id: Some(from_rbum_id.to_owned()),
                    ext_eq: ext,
                    disabled: Some(false),
                    ..Default::default()
                },
                desc_sort_by_create,
                desc_sort_by_update,
                funs,
                ctx,
            )
            .await
        } else {
            RbumRelServ::find_rels(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: with,
                        ..Default::default()
                    },
                    tag: Some(tag.to_string()),
                    from_rbum_kind: Some(from_rbum_kind.clone()),
                    from_rbum_id: Some(from_rbum_id.to_owned()),
                    ext_eq: ext,
                    disabled: Some(false),
                    ..Default::default()
                },
                desc_sort_by_create,
                desc_sort_by_update,
                funs,
                ctx,
            )
            .await
        }
    }

    pub async fn find_rels(
        tag: &PluginAppBindRelKind,
        from_rbum_kind: &RbumRelFromKind,
        from_rbum_id: &str,
        to_rbum_item_id: &str,
        ext: Option<String>,
        is_hide_secret: bool,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        if is_hide_secret {
            RbumRelServ::find_hide_secret_rels(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tag: Some(tag.to_string()),
                    from_rbum_kind: Some(from_rbum_kind.clone()),
                    from_rbum_id: Some(from_rbum_id.to_owned()),
                    to_rbum_item_id: Some(to_rbum_item_id.to_owned()),
                    ext_eq: ext,
                    disabled: Some(false),
                    ..Default::default()
                },
                desc_sort_by_create,
                desc_sort_by_update,
                funs,
                ctx,
            )
            .await
        } else {
            RbumRelServ::find_rels(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    tag: Some(tag.to_string()),
                    from_rbum_kind: Some(from_rbum_kind.clone()),
                    from_rbum_id: Some(from_rbum_id.to_owned()),
                    to_rbum_item_id: Some(to_rbum_item_id.to_owned()),
                    ext_eq: ext,
                    disabled: Some(false),
                    ..Default::default()
                },
                desc_sort_by_create,
                desc_sort_by_update,
                funs,
                ctx,
            )
            .await
        }
    }

    pub async fn paginate_rels(
        filter: &RbumRelFilterReq,
        is_hide_secret: bool,
        page_number: u32,
        page_size: u32,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        if is_hide_secret {
            RbumRelServ::paginate_hide_secret_rels(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
        } else {
            RbumRelServ::paginate_rels(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
        }
    }

    pub async fn get_rel(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumRelDetailResp> {
        let filter = RbumRelFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                own_paths: Some("".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let rel = RbumRelServ::get_rbum(id, &filter, funs, ctx).await?;
        Ok(rel)
    }

    pub async fn get_rel_agg(id: &str, is_hide_secret: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumRelAggResp> {
        let filter = RbumRelFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                own_paths: Some("".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let rbum_rel_id = id.to_string();
        let rel = Self::get_rel(id, funs, ctx).await?;
        let resp = RbumRelAggResp {
            rel,
            attrs: if is_hide_secret {
                RbumRelAttrServ::find_hide_secret(
                    &RbumRelExtFilterReq {
                        basic: filter.basic.clone(),
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?
            } else {
                RbumRelAttrServ::find_rbums(
                    &RbumRelExtFilterReq {
                        basic: filter.basic.clone(),
                        rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?
            },
            envs: RbumRelEnvServ::find_rbums(
                &RbumRelExtFilterReq {
                    basic: filter.basic.clone(),
                    rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?,
        };
        Ok(resp)
    }

    pub async fn show_rel_attr(rel_id: &str, attr_name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let attrs = RbumRelAttrServ::find_rbums(
            &RbumRelExtFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                rel_rbum_rel_id: Some(rel_id.to_string()),
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if let Some(attr) = attrs.iter().find(|attr| attr.name == attr_name) {
            return Ok(attr.value.to_string());
        }
        Ok("".to_string())
    }

    /// The query of the sensitive attributes of the plugin relationship which are stored in plaintext
    ///
    /// 以明文存储的插件关联关系敏感属性的查询
    ///
    /// NOTE: Only the attributes of the relationships of the plugin backend service (``spi_ident``)
    /// and the plugin kind binding (``PluginAppBindKind``) are included.
    ///
    /// NOTE： 仅包含插件后端服务关联（``spi_ident``）与插件类型绑定（``PluginAppBindKind``）关系上的属性。
    fn package_plaintext_secret_attr_query() -> SelectStatement {
        let mut query = Query::select();
        query
            .columns([(rbum_rel_attr::Entity, rbum_rel_attr::Column::Id), (rbum_rel_attr::Entity, rbum_rel_attr::Column::Value)])
            .from(rbum_rel_attr::Entity)
            .left_join(
                rbum_rel::Entity,
                Expr::col((rbum_rel::Entity, rbum_rel::Column::Id)).equals((rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumRelId)),
            )
            .left_join(
                rbum_kind_attr::Entity,
                Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Id)).equals((rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumKindAttrId)),
            )
            .and_where(
                Expr::col((rbum_rel::Entity, rbum_rel::Column::Tag)).is_in(vec![spi_constants::SPI_IDENT_REL_TAG.to_string(), PluginAppBindRelKind::PluginAppBindKind.to_string()]),
            )
            .and_where(Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Secret)).eq(true))
            .and_where(Expr::col((rbum_rel_attr::Entity, rbum_rel_attr::Column::Value)).ne(""))
            .and_where(Expr::col((rbum_rel_attr::Entity, rbum_rel_attr::Column::Value)).not_like(format!("{}%", secret_helper::SECRET_ENCRYPTED_PREFIX)));
        query
    }

    /// Encrypt the sensitive attributes of the plugin relationship which are stored in plaintext
    ///
    /// 把以明文存储的插件关联关系敏感属性加密
    ///
    /// NOTE: It is used for the data migration of the existing plugin records.
    ///
    /// NOTE： 用于存量插件数据的迁移。
    ///
    /// NOTE: The encrypted records are ignored, so it is idempotent and can be executed repeatedly.
    ///
    /// NOTE： 已加密的记录会被忽略，幂等，可重复执行。
    ///
    /// # Arguments
    ///
    /// * `page_size` - The number of the records processed each time, 100 by default
    /// * `dry_run` - Only count and return the id samples without writing
    pub async fn migrate_plaintext_secret_attrs(page_size: u32, dry_run: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginRelSecretMigrateResp> {
        if !dry_run && !secret_helper::is_encrypt_enabled(funs) {
            return Err(funs.err().conflict(
                "plugin_rel",
                "migrate_secret_attr",
                "The storage encryption of the sensitive attributes is not enabled, please configure [rbum.secret_attr_key] first",
                "409-spi-plugin-secret-attr-encrypt-not-enabled",
            ));
        }
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct PlaintextSecretAttrResp {
            pub id: String,
            pub value: String,
        }
        let page_size = if page_size == 0 { 100 } else { page_size };
        let total_size = funs.db().count(&Self::package_plaintext_secret_attr_query()).await?;
        let mut processed_size = 0_u64;
        let mut sample_ids = Vec::new();
        if dry_run {
            sample_ids = funs
                .db()
                .find_dtos::<PlaintextSecretAttrResp>(&Self::package_plaintext_secret_attr_query().limit(page_size as u64))
                .await?
                .into_iter()
                .map(|resp| resp.id)
                .collect();
        } else {
            loop {
                let records = funs.db().find_dtos::<PlaintextSecretAttrResp>(&Self::package_plaintext_secret_attr_query().limit(page_size as u64)).await?;
                if records.is_empty() {
                    break;
                }
                for record in records {
                    let value = secret_helper::encrypt(&record.value, funs)?;
                    if value == record.value {
                        // The encryption does not take effect, break the loop to avoid an infinite loop
                        //
                        // 加密未生效，中止以免死循环
                        return Err(funs.err().internal_error(
                            "plugin_rel",
                            "migrate_secret_attr",
                            "Fail to encrypt the sensitive attribute of the plugin relationship",
                            "500-spi-plugin-secret-attr-encrypt-failed",
                        ));
                    }
                    funs.db()
                        .update_one(
                            rbum_rel_attr::ActiveModel {
                                id: Set(record.id),
                                value: Set(value),
                                ..Default::default()
                            },
                            ctx,
                        )
                        .await?;
                    processed_size += 1;
                }
                if processed_size >= total_size {
                    break;
                }
            }
        }
        Ok(PluginRelSecretMigrateResp {
            dry_run,
            total_size,
            processed_size,
            failed_size: 0,
            sample_ids,
        })
    }

    /// The query of the sensitive attributes of the plugin relationship which are stored as ciphertext
    ///
    /// 以密文存储的插件关联关系敏感属性的查询
    fn package_encrypted_secret_attr_query() -> SelectStatement {
        let mut query = Query::select();
        query
            .columns([(rbum_rel_attr::Entity, rbum_rel_attr::Column::Id), (rbum_rel_attr::Entity, rbum_rel_attr::Column::Value)])
            .from(rbum_rel_attr::Entity)
            .left_join(
                rbum_rel::Entity,
                Expr::col((rbum_rel::Entity, rbum_rel::Column::Id)).equals((rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumRelId)),
            )
            .left_join(
                rbum_kind_attr::Entity,
                Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Id)).equals((rbum_rel_attr::Entity, rbum_rel_attr::Column::RelRbumKindAttrId)),
            )
            .and_where(
                Expr::col((rbum_rel::Entity, rbum_rel::Column::Tag)).is_in(vec![spi_constants::SPI_IDENT_REL_TAG.to_string(), PluginAppBindRelKind::PluginAppBindKind.to_string()]),
            )
            .and_where(Expr::col((rbum_kind_attr::Entity, rbum_kind_attr::Column::Secret)).eq(true))
            .and_where(Expr::col((rbum_rel_attr::Entity, rbum_rel_attr::Column::Value)).like(format!("{}%", secret_helper::SECRET_ENCRYPTED_PREFIX)));
        query
    }

    /// Decrypt the sensitive attributes of the plugin relationship which are stored as ciphertext
    ///
    /// 把以密文存储的插件关联关系敏感属性解密回明文
    ///
    /// NOTE: It is used for the data rollback, such as rolling back to a version that does not support the encryption,
    /// or rotating the encryption key (decrypt first, then encrypt with the new key).
    ///
    /// NOTE： 用于数据回退，例如回退到不支持加密的版本，或轮换加密密钥（先解密再换新密钥加密）。
    ///
    /// NOTE: The decrypted records are ignored, so it is idempotent and can be executed repeatedly.
    ///
    /// NOTE： 已解密的记录会被忽略，幂等，可重复执行。
    ///
    /// # Arguments
    ///
    /// * `page_size` - The number of the records processed each time, 100 by default
    /// * `dry_run` - Only count and return the id samples without writing
    pub async fn decrypt_secret_attrs(page_size: u32, dry_run: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginRelSecretMigrateResp> {
        if !secret_helper::is_key_configured(funs) {
            return Err(funs.err().conflict(
                "plugin_rel",
                "decrypt_secret_attr",
                "The key of the sensitive attributes is not configured, please configure [rbum.secret_attr_key] first",
                "409-spi-plugin-secret-attr-key-not-configured",
            ));
        }
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct SecretAttrResp {
            pub id: String,
            pub value: String,
        }
        let page_size = if page_size == 0 { 100 } else { page_size };
        let total_size = funs.db().count(&Self::package_encrypted_secret_attr_query()).await?;
        let mut processed_size = 0_u64;
        let mut failed_size = 0_u64;
        let mut sample_ids = Vec::new();
        if dry_run {
            sample_ids =
                funs.db().find_dtos::<SecretAttrResp>(&Self::package_encrypted_secret_attr_query().limit(page_size as u64)).await?.into_iter().map(|resp| resp.id).collect();
        } else {
            // The failed records are skipped and counted, so that a single invalid record does not block the whole migration
            //
            // 失败的记录跳过并计数，避免单条异常数据阻塞整个迁移
            let mut failed_ids = HashSet::new();
            loop {
                let records = funs.db().find_dtos::<SecretAttrResp>(&Self::package_encrypted_secret_attr_query().limit(page_size as u64)).await?;
                if records.is_empty() {
                    break;
                }
                let mut has_progress = false;
                for record in records {
                    if failed_ids.contains(&record.id) {
                        continue;
                    }
                    match secret_helper::decrypt(&record.value, funs) {
                        Ok(plaintext) if !secret_helper::is_encrypted(&plaintext) => {
                            funs.db()
                                .update_one(
                                    rbum_rel_attr::ActiveModel {
                                        id: Set(record.id),
                                        value: Set(plaintext),
                                        ..Default::default()
                                    },
                                    ctx,
                                )
                                .await?;
                            processed_size += 1;
                            has_progress = true;
                        }
                        Ok(_) => {
                            error!(
                                "[BIOS.Plugin] Fail to decrypt the sensitive attribute [{}] because the plaintext looks like a ciphertext",
                                record.id
                            );
                            failed_ids.insert(record.id);
                            failed_size += 1;
                        }
                        Err(err) => {
                            error!("[BIOS.Plugin] Fail to decrypt the sensitive attribute [{}]: {err}", record.id);
                            failed_ids.insert(record.id);
                            failed_size += 1;
                        }
                    }
                }
                if !has_progress {
                    break;
                }
            }
        }
        Ok(PluginRelSecretMigrateResp {
            dry_run,
            total_size,
            processed_size,
            failed_size,
            sample_ids,
        })
    }
}
