use num_bigint::BigInt;
use sha1::Digest;
use sha1::Sha1;

pub fn calc_hash(name: &str) -> String {
    BigInt::from_signed_bytes_be(&Sha1::digest(name)).to_str_radix(16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn calc_hashes() {
        assert_eq!(
            "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1",
            calc_hash("jeb_")
        );
        assert_eq!(
            "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48",
            calc_hash("Notch")
        );
        assert_eq!(
            "88e16a1019277b15d58faf0541e11910eb756f6",
            calc_hash("simon")
        );
    }
}
