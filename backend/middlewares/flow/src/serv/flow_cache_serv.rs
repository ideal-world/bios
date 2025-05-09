use tardis::{basic::result::TardisResult, TardisFunsInst};

use crate::flow_config::FlowConfig;

pub struct FlowCacheServ;

impl FlowCacheServ {
    pub async fn add_sync_modify_inst(own_paths: &str, tag: &str, inst_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        tardis::log::trace!(
            "add_sync_modify_inst: own_paths={},tag={},inst_id={}",
            own_paths,
            tag,
            inst_id,
        );
        funs.cache()
            .hset(
                format!(
                    "{}:{}:{}",
                    funs.conf::<FlowConfig>().cache_key_sync_modify_state,
                    own_paths,
                    tag,
                )
                .as_str(),
                inst_id,
                "",
            )
            .await?;
        Ok(())
    }

    pub async fn exist_sync_modify_inst(own_paths: &str, tag: &str, inst_id: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        tardis::log::trace!(
            " exist_sync_modify_inst: own_paths={},tag={},inst_id={}",
            own_paths,
            tag,
            inst_id,
        );
        Ok(funs.cache()
        .hexists(
            format!(
                "{}:{}:{}",
                funs.conf::<FlowConfig>().cache_key_sync_modify_state,
                own_paths,
                tag,
            )
            .as_str(),
            inst_id,
        )
        .await?)
    }

    pub async fn del_sync_modify_inst(own_paths: &str, tag: &str, inst_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        tardis::log::trace!(
            "add del_sync_modify_inst: own_paths={},tag={},inst_id={}",
            own_paths,
            tag,
            inst_id,
        );
        funs.cache()
        .hdel(
            format!(
                "{}:{}:{}",
                funs.conf::<FlowConfig>().cache_key_sync_modify_state,
                own_paths,
                tag,
            )
            .as_str(),
            inst_id,
        )
        .await?;
        Ok(())
    }
}