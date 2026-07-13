use cryptography::core::traits::{PublicKey, SignatureAlgorithm};
use cryptography::providers::{Ed25519Provider, Ed25519PublicKey};

fn main() {
    let provider = Ed25519Provider;
    let keypair = provider
        .generate_keypair()
        .expect("failed to generate keypair");

    let exported = keypair.public_key().as_bytes();
    println!(
        "Exported public key ({} bytes): {}",
        exported.len(),
        hex::encode(exported)
    );

    let imported = Ed25519PublicKey::from_bytes(exported).expect("failed to import public key");
    println!("Imported public key: {}", imported);
}
