use asteroid_mq::prelude::{Node, NodeId};
use asteroid_mq::protocol::node::edge::codec::CodecKind;
use asteroid_mq::protocol::node::edge::packet::Auth;
use asteroid_mq::protocol::node::edge::EdgeConfig;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::reqwest::StatusCode;
use tardis::{log as tracing, TardisFuns};

use crate::serv::event_connect_serv::PoemWs;
use crate::serv::event_register_serv::EventRegisterServ;

#[derive(Clone, Default, Debug)]
pub struct EventConnectApi {
    register_serv: EventRegisterServ,
}

/// Event Connect API
///
/// 事件处理API
#[poem_openapi::OpenApi(prefix_path = "/ca/connect")]
impl EventConnectApi {
    /// Connect client nodes
    ///
    /// 连接客户端节点
    #[oai(path = "/", method = "get")]
    async fn ws_process(&self, node_id: Query<String>, websocket: WebSocket) -> Result<BoxWebSocketUpgraded, tardis::web::poem::Error> {
        let peer_id = NodeId::from_base64(&node_id).map_err(|e| tardis::web::poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        let _ctx = self.register_serv.get_ctx(peer_id).await.map_err(|e| tardis::web::poem::Error::from_string(e.to_string(), StatusCode::UNAUTHORIZED))?;
        let config = EdgeConfig {
            peer_id,
            supported_codec_kinds: vec![CodecKind::JSON].into_iter().collect(),
            peer_auth: Auth {},
        };
        let Some(node) = TardisFuns::store().get_singleton::<Node>() else {
            return Err(tardis::web::poem::Error::from_string(
                "mq server node have not initialized",
                tardis::web::poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        };
        let register_serv = self.register_serv.clone();
        let upgraded: BoxWebSocketUpgraded = websocket.on_upgrade(Box::new(|stream| {
            Box::pin(async move {
                let ws = PoemWs::new(stream);
                let Ok(node_id) = node.create_edge_connection(ws, config).await.inspect_err(|e| {
                    tracing::error!(?e, "failed to create edge connection");
                }) else {
                    return;
                };
                tracing::info!(?node_id, "edge connected");
                let Some(connection) = node.get_edge_connection(node_id) else {
                    return;
                };
                let _ = connection.finish_signal.recv_async().await;
                let _ = register_serv.unregister_ctx(node_id).await;
                tracing::info!(?node_id, "edge disconnected");
            })
        }));
        Ok(upgraded)
    }
}
