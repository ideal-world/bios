use asteroid_mq::model::codec::{self, DynCodec};
use asteroid_mq::model::EdgeAuth;
use asteroid_mq::prelude::{Node, NodeId};

use asteroid_mq::protocol::node::edge::EdgeConfig;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::reqwest::StatusCode;
use tardis::{log as tracing, TardisFuns};

use crate::serv::event_connect_serv::PoemWs;
use crate::serv::event_register_serv::EventRegisterServ;

#[derive(Clone)]
pub struct EventConnectApi {
    pub(crate) register_serv: EventRegisterServ,
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
    async fn ws_process(&self, node_id: Query<String>, codec: Query<Option<String>>, websocket: WebSocket) -> Result<BoxWebSocketUpgraded, tardis::web::poem::Error> {
        let peer_id = NodeId::from_base64(&node_id).map_err(|e| tardis::web::poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        let config = EdgeConfig {
            peer_id,
            peer_auth: EdgeAuth::default(),
        };
        let _ctx = self.register_serv.get_ctx(peer_id).await.map_err(|e| tardis::web::poem::Error::from_string(e.to_string(), StatusCode::UNAUTHORIZED))?;
        let codec = match codec.0.as_deref().unwrap_or("json").to_lowercase().as_str() {
            "json" => DynCodec::new(codec::Json),
            "bincode" => DynCodec::new(codec::Bincode),
            _ => return Err(tardis::web::poem::Error::from_string("unsupported codec", StatusCode::BAD_REQUEST)),
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
                let ws = PoemWs::new(stream, codec);
                let Ok(node_id) = node.create_edge_connection(ws, config).await.inspect_err(|e| {
                    tracing::error!(?e, "failed to create edge connection");
                }) else {
                    return;
                };
                tracing::info!(?node_id, "edge connected");
                let Some(connection) = node.get_edge_connection(node_id) else {
                    return;
                };
                let _ = connection.finish_signal.notified().await;
                let _ = register_serv.unregister_ctx(node_id).await;
                tracing::info!(?node_id, "edge disconnected");
            })
        }));
        Ok(upgraded)
    }
}
