use solana_program::{
    hash::{Hash, Hasher},
    pubkey::Pubkey,
};

pub(crate) fn calculate_hash(
    receiver: &Pubkey,
    sender_key: &Pubkey,
    secret_hash: &[u8],
    token_program: Option<&Pubkey>,
    amount: u64,
) -> Hash {
    let mut hasher = Hasher::default();
    hasher.hash(receiver.as_ref());
    hasher.hash(sender_key.as_ref());
    hasher.hash(secret_hash);
    match token_program {
        Some(program) => hasher.hash(program.as_ref()),
        None => hasher.hash(&Pubkey::new_from_array([0; 32]).to_bytes()),
    }
    hasher.hash(&amount.to_le_bytes());
    hasher.result()
}

pub(crate) fn calculate_hash_from_secret(secret: &[u8]) -> Hash {
    let mut hasher = Hasher::default();
    hasher.hash(secret);
    hasher.result()
}
