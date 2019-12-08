//use std::mem::MaybeUninit;
use std::os::raw::c_int;

#[link(name = "bls384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
extern "C" {
    // global functions
    fn blsInit(curve: c_int, compiledTimeVar: c_int) -> c_int;
    fn mclBn_getFrByteSize() -> u32;
    fn mclBn_getFpByteSize() -> u32;

    fn blsSecretKeySetByCSPRNG(x: *mut SecretKey);
    fn blsSecretKeySetHexStr(x: *mut SecretKey, buf: *const u8, bufSize: usize) -> c_int;
    fn blsGetPublicKey(y: *mut PublicKey, x: *const SecretKey);
    fn blsSignHashWithDomain(
        sig: *mut Signature,
        seckey: *const SecretKey,
        msg: *const Message,
    ) -> c_int;
    fn blsVerifyHashWithDomain(
        sig: *const Signature,
        pubKey: *const PublicKey,
        msg: *const Message,
    ) -> c_int;
    fn blsVerifyAggregatedHashWithDomain(
        aggSig: *const Signature,
        pubs: *const PublicKey,
        msgs: *const Message,
        n: usize,
    ) -> c_int;

    fn blsSecretKeyIsEqual(lhs: *const SecretKey, rhs: *const SecretKey) -> i32;
    fn blsPublicKeyIsEqual(lhs: *const PublicKey, rhs: *const PublicKey) -> i32;
    fn blsSignatureIsEqual(lhs: *const Signature, rhs: *const Signature) -> i32;

    fn blsSecretKeySerialize(buf: *mut u8, maxBufSize: usize, x: *const SecretKey) -> usize;
    fn blsPublicKeySerialize(buf: *mut u8, maxBufSize: usize, x: *const PublicKey) -> usize;
    fn blsSignatureSerialize(buf: *mut u8, maxBufSize: usize, x: *const Signature) -> usize;

    fn blsSecretKeyDeserialize(x: *mut SecretKey, buf: *const u8, bufSize: usize) -> usize;
    fn blsPublicKeyDeserialize(x: *mut PublicKey, buf: *const u8, bufSize: usize) -> usize;
    fn blsSignatureDeserialize(x: *mut Signature, buf: *const u8, bufSize: usize) -> usize;

    fn blsPublicKeyAdd(pubkey: *mut PublicKey, x: *const PublicKey);
    fn blsSignatureAdd(sig: *mut Signature, x: *const Signature);

}

pub enum CurveType {
    BN254 = 0,
    BN381 = 1,
    SNARK = 4,
    BLS12_381 = 5,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlsError {
    InvalidData,
    BadSize,
    InternalError,
}

const MCLBN_FP_UNIT_SIZE: usize = 6;
const MCLBN_FR_UNIT_SIZE: usize = 4;
const BLS_COMPILER_TIME_VAR_ADJ: usize = 200;
const MCLBN_COMPILED_TIME_VAR: c_int =
    (MCLBN_FR_UNIT_SIZE * 10 + MCLBN_FP_UNIT_SIZE + BLS_COMPILER_TIME_VAR_ADJ) as c_int;

pub const HASH_SIZE: usize = 32;
pub const DOMAIN_SIZE: usize = 8;
pub const HASH_AND_DOMAIN_SIZE: usize = HASH_SIZE + DOMAIN_SIZE;

macro_rules! common_impl {
    ($t:ty, $is_equal_fn:ident) => {
        impl PartialEq for $t {
            fn eq(&self, rhs: &Self) -> bool {
                unsafe { $is_equal_fn(self, rhs) == 1 }
            }
        }
        impl $t {
            pub fn zero() -> $t {
                Default::default()
            }
            pub unsafe fn uninit() -> $t {
                std::mem::uninitialized()
            }
        }
    };
}

macro_rules! serialize_impl {
    ($t:ty, $size:expr, $serialize_fn:ident, $deserialize_fn:ident) => {
        impl $t {
            pub fn deserialize(&mut self, buf: &[u8]) -> bool {
                unsafe { $deserialize_fn(self, buf.as_ptr(), buf.len()) > 0 }
            }
            pub fn from_serialized(buf: &[u8]) -> Result<$t, BlsError> {
                let mut v = unsafe { <$t>::uninit() };
                if v.deserialize(buf) {
                    return Ok(v);
                }
                Err(BlsError::InvalidData)
            }
            pub fn serialize(&self) -> Vec<u8> {
                let size = unsafe { $size } as usize;
                let mut buf: Vec<u8> = Vec::with_capacity(size);
                let n: usize;
                unsafe {
                    n = $serialize_fn(buf.as_mut_ptr(), size, self);
                }
                if n == 0 {
                    panic!("serialize");
                }
                unsafe {
                    buf.set_len(n);
                }
                buf
            }
        }
    };
}

pub fn init(curve_type: CurveType) -> bool {
    unsafe { blsInit(curve_type as c_int, MCLBN_COMPILED_TIME_VAR) == 0 }
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Message {
    pub hash: [u8; HASH_SIZE],
    pub domain: [u8; DOMAIN_SIZE],
}

impl Message {
    pub fn zero() -> Message {
        Default::default()
    }
    pub unsafe fn uninit() -> Message {
        std::mem::uninitialized()
    }
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct SecretKey {
    d: [u64; MCLBN_FR_UNIT_SIZE],
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct PublicKey {
    x: [u64; MCLBN_FP_UNIT_SIZE],
    y: [u64; MCLBN_FP_UNIT_SIZE],
    z: [u64; MCLBN_FP_UNIT_SIZE],
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Signature {
    x: [u64; MCLBN_FP_UNIT_SIZE * 2],
    y: [u64; MCLBN_FP_UNIT_SIZE * 2],
    z: [u64; MCLBN_FP_UNIT_SIZE * 2],
}

common_impl![SecretKey, blsSecretKeyIsEqual];
serialize_impl![
    SecretKey,
    mclBn_getFrByteSize(),
    blsSecretKeySerialize,
    blsSecretKeyDeserialize
];

common_impl![PublicKey, blsPublicKeyIsEqual];
serialize_impl![
    PublicKey,
    mclBn_getFpByteSize(),
    blsPublicKeySerialize,
    blsPublicKeyDeserialize
];

common_impl![Signature, blsSignatureIsEqual];
serialize_impl![
    Signature,
    mclBn_getFpByteSize() * 2,
    blsSignatureSerialize,
    blsSignatureDeserialize
];

impl SecretKey {
    pub fn set_by_csprng(&mut self) {
        unsafe { blsSecretKeySetByCSPRNG(self) }
    }
    pub fn set_hex_str(&mut self, s: &str) -> bool {
        unsafe { blsSecretKeySetHexStr(self, s.as_ptr(), s.len()) > 0 }
    }
    pub fn from_hex_str(s: &str) -> Result<SecretKey, BlsError> {
        let mut v = unsafe { SecretKey::uninit() };
        if v.set_hex_str(&s) {
            return Ok(v);
        }
        Err(BlsError::InvalidData)
    }
    pub fn get_publickey(&self) -> PublicKey {
        let mut v = unsafe { PublicKey::uninit() };
        unsafe {
            blsGetPublicKey(&mut v, self);
        }
        v
    }
    pub fn sign_message(&self, msg: &Message) -> Result<Signature, BlsError> {
        let mut v = unsafe { Signature::uninit() };
        unsafe {
            if blsSignHashWithDomain(&mut v, self, msg) == 0 {
                return Ok(v);
            }
        }
        Err(BlsError::InternalError)
    }
}

impl PublicKey {
    pub fn add_assign(&mut self, x: *const PublicKey) {
        unsafe {
            blsPublicKeyAdd(self, x);
        }
    }
}

impl Signature {
    pub fn verify_message(&self, pubkey: *const PublicKey, msg: &Message) -> bool {
        unsafe { blsVerifyHashWithDomain(self, pubkey, msg) == 1 }
    }
    pub fn verify_aggregated_message(&self, pubkeys: &[PublicKey], msgs: &[Message]) -> bool {
        let n = pubkeys.len();
        if msgs.len() != n {
            return false;
        }
        unsafe { blsVerifyAggregatedHashWithDomain(self, pubkeys.as_ptr(), msgs.as_ptr(), n) == 1 }
    }
    pub fn add_assign(&mut self, x: *const Signature) {
        unsafe {
            blsSignatureAdd(self, x);
        }
    }
}
