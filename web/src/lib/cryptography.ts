import sodium from 'libsodium-wrappers-sumo'

async function derive_salt(uuid: string, pwd: string) {
    let state = sodium.crypto_generichash_init(
        null, sodium.crypto_pwhash_SALTBYTES)
    sodium.crypto_generichash_update(state, sodium.from_string("salt$"))
    sodium.crypto_generichash_update(state, sodium.from_string(uuid))
    sodium.crypto_generichash_update(state, sodium.from_string("$"))
    sodium.crypto_generichash_update(state, sodium.from_string(pwd))
    sodium.crypto_generichash_update(state, sodium.from_string("$salt"))
    return sodium.crypto_generichash_final(state, sodium.crypto_pwhash_SALTBYTES)
}

export async function restricted_hash(uuid: string, pwd: string) {
    await sodium.ready

    const salt = await derive_salt(uuid, pwd)
    const ret = sodium.crypto_pwhash(
        32,
        pwd,
        salt,
        sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
        sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
        sodium.crypto_pwhash_ALG_ARGON2ID13
    )
    return sodium.to_hex(ret)
}
