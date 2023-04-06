use std::sync::RwLock;

use lazy_static::lazy_static;

use crate::{
    mini_tardis::crypto::sm::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
    modules::res_process::ResContainerNode,
};

lazy_static! {
    pub(crate) static ref SERV_URL: RwLock<String> = RwLock::new(String::new());
    pub(crate) static ref GLOBAL_API_MODE: RwLock<bool> = RwLock::new(false);
    pub(crate) static ref RES_CONTAINER: RwLock<Option<ResContainerNode>> = RwLock::new(None);
    pub(crate) static ref ENCRYPT_SERV_PUB_KEY: RwLock<Option<TardisCryptoSm2PublicKey>> = RwLock::new(None);
    pub(crate) static ref ENCRYPT_FD_SM2_KEYS: RwLock<Option<(String, TardisCryptoSm2PrivateKey)>> = RwLock::new(None);
}
