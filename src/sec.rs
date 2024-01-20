use crate::hmacmode::HmacKey;
use hmac::{Hmac, Mac};
use sha1::Sha1;

const PRESET_VALUE: u16 = 0xFFFF;
const POLYNOMIAL: u16 = 0x8408;
const SHA1_DIGEST_SIZE: usize = 20;
pub const CRC_RESIDUAL_OK: u16 = 0xf0b8;

type HmacSha1 = Hmac<Sha1>;

pub fn hmac_sha1(key: &HmacKey, data: &[u8]) -> [u8; SHA1_DIGEST_SIZE] {
    let mut hmac = HmacSha1::new_from_slice(&key.0).unwrap();
    hmac.update(data);
    let result = hmac.finalize();

    let mut code = [0; SHA1_DIGEST_SIZE];
    code.copy_from_slice(result.into_bytes().as_slice());

    code
}

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc_value = PRESET_VALUE;
    for &b in data {
        crc_value ^= b as u16;
        for _ in 0..8 {
            let j = crc_value & 1;
            crc_value >>= 1;
            if j != 0 {
                crc_value ^= POLYNOMIAL
            }
        }
    }
    crc_value
}
