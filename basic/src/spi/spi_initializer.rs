use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::TardisFuns;

use crate::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use crate::rbum::rbum_enumeration::RbumScopeLevelKind;
use crate::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;

use super::domain::spi_bs;

pub async fn init(code: &str) -> TardisResult<String> {
    let mut funs = TardisFuns::inst_with_db_conn(code.to_string(), None);
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    if RbumDomainServ::get_rbum_domain_id_by_code(code, &funs).await?.is_some() {
        let domain_id = RbumDomainServ::get_rbum_domain_id_by_code(code, &funs).await?.ok_or_else(|| funs.err().not_found("spi", "init", "not found spi domain", ""))?;
        return Ok(domain_id);
    }
    
    funs.begin().await?;
    funs.db().init(spi_bs::ActiveModel::init(TardisFuns::reldb().backend(), None)).await?;
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(code.to_string()),
            name: TrimString(code.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;
    funs.commit().await?;
    Ok(domain_id)
}


