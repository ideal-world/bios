use std::collections::HashMap;
use tardis::basic::error::TardisError;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::prelude::*;
use tardis::{basic::result::TardisResult, db::sea_orm};

use asteroid_mq::prelude::{
    DurableMessage, EndpointAddr, MaybeBase64Bytes, Message, MessageAckExpectKind, MessageDurableConfig, MessageHeader, MessageId, MessageStatusKind, MessageTargetKind, Subject,
    TopicCode,
};
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "mq_message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub message_id: String,
    #[sea_orm(indexed)]
    pub topic: String,
    #[sea_orm(indexed)]
    pub archived: bool,
    pub ack_kind: i16,
    pub target_kind: i16,
    #[sea_orm(column_type = "DateTime")]
    pub expire_time: Option<DateTime<Utc>>,
    pub max_receiver: Option<i32>,
    pub subjects: Vec<String>,
    pub payload: Vec<u8>,
    pub status: Vec<u8>,
    #[sea_orm(column_type = "DateTime")]
    pub time: DateTime<Utc>,
}

const EP_ADDR_SIZE: usize = size_of::<EndpointAddr>();
const STATUS_SIZE: usize = 1;
const ENTRY_SIZE: usize = EP_ADDR_SIZE + STATUS_SIZE;
impl Model {
    pub fn status_update(&mut self, mut status: HashMap<EndpointAddr, MessageStatusKind>) {
        for entry in self.status.chunks_mut(ENTRY_SIZE) {
            if entry.len() != ENTRY_SIZE {
                break;
            }
            let (addr, kind) = entry.split_at_mut(EP_ADDR_SIZE);
            let mut addr_bytes = [0u8; EP_ADDR_SIZE];
            addr_bytes.copy_from_slice(addr);

            let addr = EndpointAddr::from(addr_bytes);
            if let Some(update_kind) = status.remove(&addr) {
                kind[0] = update_kind as u8;
            }
        }
        for (addr, kind) in status {
            self.status.extend(addr.bytes);
            self.status.push(kind as u8);
        }
    }
    pub fn status_to_binary(status: HashMap<EndpointAddr, MessageStatusKind>) -> Vec<u8> {
        let mut status_vec = Vec::new();
        for (addr, kind) in status {
            status_vec.extend(addr.bytes);
            status_vec.push(kind as u8);
        }
        status_vec
    }
    pub fn status_from_binary(status: Vec<u8>) -> HashMap<EndpointAddr, MessageStatusKind> {
        let mut status_map = HashMap::new();
        for entry in status.chunks(ENTRY_SIZE) {
            if entry.len() != ENTRY_SIZE {
                break;
            }
            let (addr, kind) = entry.split_at(EP_ADDR_SIZE);
            let mut addr_bytes = [0u8; EP_ADDR_SIZE];
            addr_bytes.copy_from_slice(addr);
            let addr = EndpointAddr::from(addr_bytes);
            let Some(kind) = MessageStatusKind::try_from_u8(kind[0]) else {
                continue;
            };
            status_map.insert(addr, kind);
        }
        status_map
    }
    pub fn from_durable_message(topic: TopicCode, durable_message: DurableMessage) -> Self {
        Model {
            topic: topic.to_string(),
            message_id: durable_message.message.header.message_id.to_base64(),
            ack_kind: durable_message.message.header.ack_kind as u8 as i16,
            target_kind: durable_message.message.header.target_kind as u8 as i16,
            expire_time: durable_message.message.header.durability.as_ref().map(|d| d.expire),
            max_receiver: durable_message.message.header.durability.and_then(|d| d.max_receiver.map(|r| r as i32)),
            subjects: durable_message.message.header.subjects.iter().map(ToString::to_string).collect(),
            payload: durable_message.message.payload.0.to_vec(),
            status: Self::status_to_binary(durable_message.status),
            time: durable_message.time,
            archived: false,
        }
    }
    pub fn try_into_durable_message(self) -> TardisResult<DurableMessage> {
        let message = DurableMessage {
            message: Message {
                header: MessageHeader {
                    message_id: MessageId::from_base64(&self.message_id).map_err(|e| TardisError::internal_error(&e.to_string(), "base-64-decode"))?,
                    ack_kind: MessageAckExpectKind::try_from_u8(self.ack_kind as u8).expect("valid ack kind"),
                    target_kind: MessageTargetKind::from(self.target_kind as u8),
                    durability: self.expire_time.map(|expire| MessageDurableConfig {
                        expire,
                        max_receiver: self.max_receiver.map(|r| r as u32),
                    }),
                    subjects: self.subjects.into_iter().map(Subject::new).collect(),
                },
                payload: MaybeBase64Bytes::new(self.payload.into()),
            },
            status: Self::status_from_binary(self.status),
            time: self.time,
        };
        Ok(message)
    }
}
