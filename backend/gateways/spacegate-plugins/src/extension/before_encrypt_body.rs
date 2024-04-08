use spacegate_shell::hyper::body::Bytes;

#[derive(Clone)]
pub struct BeforeEncryptBody {
    inner: Bytes,
}

impl BeforeEncryptBody {
    pub fn new(body: Bytes) -> Self {
        BeforeEncryptBody { inner: body }
    }
    pub fn get(self) -> Bytes {
        self.inner
    }
}
