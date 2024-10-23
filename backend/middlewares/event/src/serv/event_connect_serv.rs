use std::task::ready;

use asteroid_mq::protocol::node::edge::{
    codec::CodecKind,
    connection::{NodeConnection, NodeConnectionError, NodeConnectionErrorKind},
    packet::EdgePacket,
};
use tardis::{
    futures::{Sink, Stream},
    log as tracing,
    web::poem::web::websocket::{Message, WebSocketStream},
};
pin_project_lite::pin_project! {
    pub struct PoemWs {
        #[pin]
        inner: WebSocketStream,
    }
}
impl PoemWs {
    pub fn new(inner: WebSocketStream) -> Self {
        Self { inner }
    }
}
impl Sink<EdgePacket> for PoemWs {
    type Error = NodeConnectionError;

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx).map_err(|e| NodeConnectionError::new(NodeConnectionErrorKind::Underlying(Box::new(e)), "web socket poll ready failed"))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: EdgePacket) -> Result<(), Self::Error> {
        self.project()
            .inner
            .start_send(Message::Binary(item.payload.to_vec()))
            .map_err(|e| NodeConnectionError::new(NodeConnectionErrorKind::Underlying(Box::new(e)), "web socket start send failed"))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx).map_err(|e| NodeConnectionError::new(NodeConnectionErrorKind::Underlying(Box::new(e)), "web socket poll flush failed"))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx).map_err(|e| NodeConnectionError::new(NodeConnectionErrorKind::Underlying(Box::new(e)), "web socket poll close failed"))
    }
}

impl Stream for PoemWs {
    type Item = Result<EdgePacket, NodeConnectionError>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let next = ready!(self.project().inner.poll_next(cx));
        match next {
            Some(Ok(Message::Binary(data))) => {
                let packet = EdgePacket::new(CodecKind::JSON, data);
                std::task::Poll::Ready(Some(Ok(packet)))
            }
            Some(Ok(Message::Text(data))) => {
                let packet = EdgePacket::new(CodecKind::JSON, data);
                std::task::Poll::Ready(Some(Ok(packet)))
            }
            Some(Ok(Message::Close(_))) => {
                tracing::debug!("received close message");
                std::task::Poll::Ready(None)
            }
            Some(Ok(p)) => {
                tracing::debug!(?p, "unexpected message type");
                // immediately wake up the task to poll next
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
            Some(Err(e)) => std::task::Poll::Ready(Some(Err(NodeConnectionError::new(
                NodeConnectionErrorKind::Underlying(Box::new(e)),
                "web socket poll next failed",
            )))),
            None => std::task::Poll::Ready(None),
        }
    }
}

impl NodeConnection for PoemWs {}
