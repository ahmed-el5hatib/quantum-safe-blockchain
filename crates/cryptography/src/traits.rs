use std::fmt;

use blockchain_core::HashDigest;
use serde::{Deserialize, Serialize};

pub trait SignatureAlgorithm: Send + Sync + fmt::Debug {
    type PublicKey: PublicKey;
    type PrivateKey: PrivateKey;
    type Signature: Signature;

    fn generate_keypair(&self) -> CryptoResult<KeyPair<Self::PublicKey, Self::PrivateKey>>;
    fn sign(&self, private_key: &Self::PrivateKey, message: &[u8])
        -> CryptoResult<Self::Signature>;
    fn verify(
        &self,
        public_key: &Self::PublicKey,
        message: &[u8],
        signature: &Self::Signature,
    ) -> CryptoResult<()>;
    fn algorithm_name(&self) -> &str;
}

pub trait HashFunction: Send + Sync + fmt::Debug {
    fn hash(&self, data: &[u8]) -> HashDigest;
    fn hash_len(&self) -> usize;
    fn algorithm_name(&self) -> &str;
}

pub trait KeyEncapsulationMechanism: Send + Sync + fmt::Debug {
    type PublicKey: PublicKey;
    type PrivateKey: PrivateKey;
    type EncapsulatedKey: AsRef<[u8]> + Send + Sync + fmt::Debug;

    fn generate_keypair(&self) -> CryptoResult<KeyPair<Self::PublicKey, Self::PrivateKey>>;
    fn encapsulate(
        &self,
        public_key: &Self::PublicKey,
    ) -> CryptoResult<(Self::EncapsulatedKey, Vec<u8>)>;
    fn decapsulate(
        &self,
        private_key: &Self::PrivateKey,
        ciphertext: &[u8],
    ) -> CryptoResult<Vec<u8>>;
    fn algorithm_name(&self) -> &str;
}

pub trait RandomGenerator: Send + Sync + fmt::Debug {
    fn fill(&self, dest: &mut [u8]) -> CryptoResult<()>;
    fn try_fill(&self, dest: &mut [u8]) -> CryptoResult<()>;
    fn algorithm_name(&self) -> &str;
}

pub trait PublicKey:
    Send + Sync + fmt::Debug + Clone + serde::Serialize + for<'de> serde::Deserialize<'de>
{
    fn as_bytes(&self) -> &[u8];
    fn algorithm_name(&self) -> &str;
}

pub trait PrivateKey:
    Send + Sync + fmt::Debug + Clone + serde::Serialize + for<'de> serde::Deserialize<'de>
{
    fn as_bytes(&self) -> &[u8];
    fn algorithm_name(&self) -> &str;
}

pub trait Signature:
    Send + Sync + fmt::Debug + Clone + serde::Serialize + for<'de> serde::Deserialize<'de>
{
    fn as_bytes(&self) -> &[u8];
    fn algorithm_name(&self) -> &str;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyPair<Public, Private> {
    pub public_key: Public,
    pub private_key: Private,
}

impl<Public, Private> KeyPair<Public, Private>
where
    Public: PublicKey,
    Private: PrivateKey,
{
    pub fn new(public_key: Public, private_key: Private) -> Self {
        Self {
            public_key,
            private_key,
        }
    }
}
