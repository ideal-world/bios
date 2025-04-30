use tardis::{basic::result::TardisResult, TardisFunsInst};

use crate::flow_config::FlowConfig;

pub struct FlowCacheServ;

impl FlowCacheServ {
    pub async fn add_or_modify_sync_modify_inst(own_paths: &str, tag: &str, inst_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        tardis::log::trace!(
            "add sync_modify_inst: own_paths={},tag={},inst_id={}",
            own_paths,
            tag,
            inst_id,
        );
        let match_path = &funs.conf::<FlowConfig>().cache_key_sync_modify_state;
        funs.cache()
            .set(
                format!(
                    "{}{}:{}:{}",
                    funs.conf::<FlowConfig>().cache_key_sync_modify_state,
                    own_paths,
                    tag,
                    inst_id,
                )
                .as_str(),
                value,
            )
            .await?;
        Ok(())
    }
}