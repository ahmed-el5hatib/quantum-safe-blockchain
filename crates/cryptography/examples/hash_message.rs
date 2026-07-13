use cryptography::core::traits::HashFunction;
use cryptography::providers::Sha256Provider;

fn main() {
    let hasher = Sha256Provider;

    let message = b"Hello, Quantum Safe Blockchain!";
    let digest = hasher.hash(message);
    println!("SHA-256: {}", digest);
    println!("Bytes: {:x?}", digest.as_bytes());
}
