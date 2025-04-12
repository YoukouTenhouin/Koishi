use hex;
use libsodium_rs::{crypto_generichash, crypto_pwhash};

type SodiumResult<T> = Result<T, libsodium_rs::SodiumError>;

fn derive_salt(uuid: &str, pwd: &str) -> SodiumResult<Vec<u8>> {
    let mut state = crypto_generichash::State::new(None, crypto_pwhash::SALTBYTES)?;
    state.update(b"salt$");
    state.update(uuid.as_bytes());
    state.update(b"$");
    state.update(pwd.as_bytes());
    state.update(b"$salt");
    Ok(state.finalize())
}

fn derive_key(uuid: &str, pwd: &str) -> SodiumResult<Vec<u8>> {
    let salt = derive_salt(uuid, pwd)?;
    crypto_pwhash::pwhash(
        32,
        pwd.as_bytes(),
        salt.as_slice(),
        crypto_pwhash::OPSLIMIT_INTERACTIVE,
        crypto_pwhash::MEMLIMIT_INTERACTIVE,
        crypto_pwhash::ALG_ARGON2ID13,
    )
}

pub(crate) fn restricted_hash(uuid: &str, pwd: &str) -> SodiumResult<String> {
    derive_key(uuid, pwd).map(|v| hex::encode(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        assert_eq!(
            restricted_hash("019627e8561f77b1b97022f0cdefeaf6", "114514").unwrap(),
            "b49a98edc7716bfab1e3208694167b737379abf382d0ede02a023fff7f5008bc"
        );
        assert_eq!(
            restricted_hash("019627e86239727195b8b4a729d1ae31", "1919810").unwrap(),
            "403f98674ad128812c24299d424e32d8e5cebb8702ec2635baa45fa939ae81b0"
        );
    }
}
