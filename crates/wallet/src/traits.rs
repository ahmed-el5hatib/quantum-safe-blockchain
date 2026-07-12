use crate::{CryptoResult, KeyPair, PrivateKey, PublicKey, Signature};

pub trait Wallet: Send + Sync + fmt::Debug {
    fn create_account(&mut self, name: &str) -> CoreResult<AccountId>;
    fn get_account(&self, id: &AccountId) -> CoreResult<Option<Account>>;
    fn list_accounts(&self) -> CoreResult<Vec<AccountId>>;
    fn remove_account(&mut self, id: &AccountId) -> CoreResult<()>;
}

pub trait Account: Send + Sync + fmt::Debug + Clone {
    fn id(&self) -> &AccountId;
    fn name(&self) -> &str;
    fn address(&self) -> &Address;
    fn public_key(&self) -> &dyn PublicKey;
}

pub trait Address:
    Send
    + Sync
    + fmt::Debug
    + Clone
    + Eq
    + PartialEq
    + serde::Serialize
    + for<'de> serde::Deserialize<'de>
{
    fn as_bytes(&self) -> &[u8];
    fn to_string(&self) -> String;
    fn from_string(s: &str) -> CoreResult<Self>
    where
        Self: Sized;
}

pub trait WalletManager: Send + Sync + fmt::Debug {
    fn create_wallet(&self, password: &[u8]) -> CoreResult<Box<dyn Wallet>>;
    fn open_wallet(&self, data: &[u8], password: &[u8]) -> CoreResult<Box<dyn Wallet>>;
}

pub trait Signer: Send + Sync + fmt::Debug {
    fn sign(&self, private_key: &dyn PrivateKey, message: &[u8]) -> CryptoResult<Signature>;
}

pub trait Verifier: Send + Sync + fmt::Debug {
    fn verify(
        &self,
        public_key: &dyn PublicKey,
        message: &[u8],
        signature: &Signature,
    ) -> CryptoResult<()>;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountId(pub String);
