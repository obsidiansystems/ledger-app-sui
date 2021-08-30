use nanos_sdk::bindings::*;
use nanos_sdk::ecc::{CurvesId, DerEncodedEcdsaSignature};
use nanos_sdk::io::SyscallError;
use core::default::Default;
use core::fmt;

pub const BIP32_PATH: [u32; 5] = nanos_sdk::ecc::make_bip32_path(b"m/44'/535348'/0'/0/0");

/// Helper function that derives the seed over secp256k1
pub fn bip32_derive_secp256k1(path: &[u32]) -> Result<[u8; 32], SyscallError> {
    let mut raw_key = [0u8; 32];
    nanos_sdk::ecc::bip32_derive(CurvesId::Secp256k1, path, &mut raw_key)?;
    Ok(raw_key)
}

/// Helper function that signs with ECDSA in deterministic nonce,
/// using SHA256
#[allow(dead_code)]
pub fn detecdsa_sign(
    m: &[u8],
    ec_k: &cx_ecfp_private_key_t,
) -> Option<(DerEncodedEcdsaSignature, u32)> {
    nanos_sdk::ecc::ecdsa_sign(ec_k, CX_RND_RFC6979 | CX_LAST, CX_SHA256, m)
}

pub fn get_pubkey(path: &[u32]) -> Result<nanos_sdk::bindings::cx_ecfp_public_key_t, SyscallError> {
    let raw_key = bip32_derive_secp256k1(path)?;
    let mut ec_k = nanos_sdk::ecc::ec_init_key(CurvesId::Secp256k1, &raw_key)?;
    nanos_sdk::ecc::ec_get_pubkey(CurvesId::Secp256k1, &mut ec_k)
}

#[allow(dead_code)]
pub fn get_private_key(path: &[u32]) -> Result<nanos_sdk::bindings::cx_ecfp_private_key_t, SyscallError> {
    let raw_key = bip32_derive_secp256k1(path)?;
    nanos_sdk::ecc::ec_init_key(CurvesId::Secp256k1, &raw_key)
}

// Public Key Hash type; update this to match the target chain's notion of an address and how to
// format one.

pub struct PKH([u8; 32]);

#[allow(dead_code)]
pub fn get_pkh(key : nanos_sdk::bindings::cx_ecfp_public_key_t) -> PKH {
    let mut public_key_hash = PKH::default();
    unsafe { 
        cx_hash_sha256(key.W.as_ptr(), key.W_len, public_key_hash.0[..].as_mut_ptr(), public_key_hash.0.len() as u32);
    }
    public_key_hash
}

impl Default for PKH {
    fn default() -> PKH {
        PKH(<[u8; 32]>::default())
    }
}

impl fmt::Display for PKH {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pkh-")?;
        for byte in self.0 {
            write!(f, "{:X}", byte)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Hasher(cx_sha256_s);

impl Hasher {
    pub fn new() -> Hasher {
        let mut rv = cx_sha256_s::default();
        unsafe { cx_sha256_init_no_throw(&mut rv) };
        Self(rv)
    }

    pub fn update(&mut self, bytes: &[u8]) {
        unsafe {
            cx_hash_update(&mut self.0 as *mut cx_sha256_s as *mut cx_hash_t, bytes.as_ptr(), bytes.len() as u32);
        }
    }

    pub fn finalize(&mut self) -> Hash {
        let mut rv = <[u8; 32]>::default();
        unsafe { cx_hash_final(&mut self.0 as *mut cx_sha256_s as *mut cx_hash_t, rv.as_mut_ptr()) };
        Hash(rv)
    }
}

pub struct Hash(pub [u8; 32]);

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:X}", byte)?;
        }
        Ok(())
    }
}


