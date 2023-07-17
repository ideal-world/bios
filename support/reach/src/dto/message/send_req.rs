/* /**
 * 用户触达消息发送请求
 *
 * @author gudaoxuri
 */
@Data
@SuperBuilder
@NoArgsConstructor
@AllArgsConstructor
public class ReachMsgSendReq {

    private String sceneCode;
     private List<ReachMsgReceive> receives;
    private String relItemId;
    private Map<String, String> replace;
    private String ownPaths;

    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    public static class ReachMsgReceive {
        private String receiveGroupCode;
        private ReachReceiveKind receiveKind;
        private Set<String> receiveIds;
    }
} */

use super::*;
use std::collections::HashMap;
use tardis::web::poem_openapi;

/// 用户触达消息发送请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSendReq {
    scene_code: String,
    receives: Vec<ReachMsgReceive>,
    rel_item_id: String,
    replace: HashMap<String, String>,
    own_paths: String,
}

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgReceive {
    receive_group_code: String,
    receive_kind: ReachReceiveKind,
    receive_ids: Vec<String>,
}
