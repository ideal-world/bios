use spacegate_shell::hyper::body::Bytes;

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
