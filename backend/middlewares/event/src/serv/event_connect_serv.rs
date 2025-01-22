use std::task::ready;

use asteroid_mq::model::{
    codec::{Codec, CodecKind, DynCodec},
    connection::{EdgeConnectionError, EdgeConnectionErrorKind, EdgeNodeConnection},
    EdgePayload,
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
        codec: DynCodec,
    }
}
impl PoemWs {
    pub fn new(inner: WebSocketStream, codec: DynCodec) -> Self {
        Self { inner, codec }
    }
}
impl Sink<EdgePayload> for PoemWs {
    type Error = EdgeConnectionError;

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx).map_err(|e| EdgeConnectionError::new(EdgeConnectionErrorKind::Underlying(Box::new(e)), "web socket poll ready failed"))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: EdgePayload) -> Result<(), Self::Error> {
        let this = self.project();
        this.inner
            .start_send(Message::Binary(
                this.codec.encode(&item).map_err(EdgeConnectionError::codec("web socket start send failed"))?,
            ))
            .map_err(|e| EdgeConnectionError::new(EdgeConnectionErrorKind::Underlying(Box::new(e)), "web socket start send failed"))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx).map_err(|e| EdgeConnectionError::new(EdgeConnectionErrorKind::Underlying(Box::new(e)), "web socket poll flush failed"))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx).map_err(|e| EdgeConnectionError::new(EdgeConnectionErrorKind::Underlying(Box::new(e)), "web socket poll close failed"))
    }
}

impl Stream for PoemWs {
    type Item = Result<EdgePayload, EdgeConnectionError>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let next = ready!(this.inner.poll_next(cx));
        match next {
            Some(Ok(Message::Binary(data))) => {
                let payload_result = this.codec.decode(&data).map_err(EdgeConnectionError::codec("axum ws poll next failed"));
                std::task::Poll::Ready(Some(payload_result))
            }
            Some(Ok(Message::Text(data))) => {
                let payload_result = this.codec.decode(data.as_bytes()).map_err(EdgeConnectionError::codec("axum ws poll next failed"));
                std::task::Poll::Ready(Some(payload_result))
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
            Some(Err(e)) => std::task::Poll::Ready(Some(Err(EdgeConnectionError::new(
                EdgeConnectionErrorKind::Underlying(Box::new(e)),
                "web socket poll next failed",
            )))),
            None => std::task::Poll::Ready(None),
        }
    }
}

impl EdgeNodeConnection for PoemWs {}
