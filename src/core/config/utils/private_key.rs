use rustls_pki_types::PrivateKeyDer;



#[derive(Debug)]
pub struct PrivateKeyUtil(PrivateKeyDer<'static>);

impl Clone for PrivateKeyUtil {
    fn clone(&self) -> Self {
        Self(self.0.clone_key())
    }
}

impl From<PrivateKeyDer<'static>> for PrivateKeyUtil {
    fn from(value: PrivateKeyDer<'static>) -> Self {
        Self(value)
    }
}

impl PrivateKeyUtil {
    pub fn into_inner(self) -> PrivateKeyDer<'static> {
        self.0
    }
}