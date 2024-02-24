use rand::Rng;
use sec::hmac_sha1;
use std;

/// Size of the secret used by the HMAC algorithm
pub const HMAC_SECRET_SIZE: usize = 20;

/// Secret used to seed the HMAC algorithm
pub type HmacSecret = [u8; HMAC_SECRET_SIZE];

#[derive(Debug)]
pub struct Hmac(pub HmacSecret);

impl Drop for Hmac {
    fn drop(&mut self) {
        for i in self.0.iter_mut() {
            *i = 0;
        }
    }
}

impl std::ops::Deref for Hmac {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hmac {
    pub fn check(&self, key: &HmacKey, challenge: &[u8]) -> bool {
        &self.0[..] == hmac_sha1(key, challenge)
    }
}

/// A secret key for HMAC, derived from the HMAC secret
#[derive(Debug)]
pub struct HmacKey(pub HmacSecret);
impl Drop for HmacKey {
    fn drop(&mut self) {
        for i in self.0.iter_mut() {
            *i = 0;
        }
    }
}

impl HmacKey {
    pub fn from_slice(s: &[u8]) -> Self {
        let mut key = HmacKey([0; HMAC_SECRET_SIZE]);
        (&mut key.0).clone_from_slice(s);
        key
    }

    pub fn generate<R: Rng>(mut rng: R) -> Self {
        let mut key = HmacKey([0; HMAC_SECRET_SIZE]);
        for i in key.0.iter_mut() {
            *i = rng.gen()
        }
        key
    }
}
