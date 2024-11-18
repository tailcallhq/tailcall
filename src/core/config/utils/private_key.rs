use rustls_pki_types::PrivateKeyDer;

#[derive(Debug)]
pub struct PrivateKey(PrivateKeyDer<'static>);

impl Clone for PrivateKey {
    fn clone(&self) -> Self {
        Self(self.0.clone_key())
    }
}

impl From<PrivateKeyDer<'static>> for PrivateKey {
    fn from(value: PrivateKeyDer<'static>) -> Self {
        Self(value)
    }
}

impl PrivateKey {
    pub fn into_inner(self) -> PrivateKeyDer<'static> {
        self.0
    }
}
