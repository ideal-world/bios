use std::sync::Arc;

use tardis::{basic::result::TardisResult, tokio};

mod event_listener;
mod message_send_listener;

pub async fn init() -> TardisResult<()> {
    // deprecate mq tunnel
    event_listener::EventListener::default().run().await?;
    tokio::task::spawn(async move {
        let period = tokio::time::Duration::from_secs(2);
        let mut interval = tokio::time::interval(period);
        let task = Arc::new(message_send_listener::MessageSendListener::default());
        loop {
            // TODO: here should have a barrier to waiting for web server's startup
            // it should modify bios
            let _ = task.run().await;
            // let task = task_raw.clone();
            // let _result = tokio::spawn(async move {
            //     task.run().await
            // });
            interval.tick().await;
        }
    });
    Ok(())
}
