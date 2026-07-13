use cryptography::core::traits::SignatureAlgorithm;
use cryptography::providers::Ed25519Provider;

fn main() {
    let provider = Ed25519Provider;

    let keypair = provider
        .generate_keypair()
        .expect("failed to generate keypair");
    println!("Public key: {}", keypair.public_key());
    println!("Private key: {}", keypair.private_key());

    let message = b"Hello, Quantum Safe Blockchain!";
    let signature = provider
        .sign(keypair.private_key(), message)
        .expect("failed to sign");
    println!("Signature: {}", signature);

    provider
        .verify(keypair.public_key(), message, &signature)
        .expect("verification failed");
    println!("Signature verified successfully!");
}
