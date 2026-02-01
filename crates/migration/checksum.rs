use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn sha256_hex_is_deterministic() {
        let a = sha256_hex(b"hello\n");
        let b = sha256_hex(b"hello\n");
        assert_eq!(a, b);
    }

    #[test]
    fn sha256_hex_matches_known_vector() {
        let got = sha256_hex(b"abc");
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(got, expected);
    }
}
