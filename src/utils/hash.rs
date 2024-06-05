use solana_program::{
    hash::{Hash, Hasher},
    pubkey::Pubkey,
};

pub(crate) fn calculate_hash(receiver: &Pubkey, sender_key: &Pubkey, secret_hash: &[u8], token_program: Option<&Pubkey>, amount: u64) -> Hash {
    let mut hasher = Hasher::default();
    hasher.hash(receiver.as_ref());
    hasher.hash(sender_key.as_ref());
    hasher.hash(secret_hash);
    if let Some(program) = token_program {
        hasher.hash(program.as_ref());
    } else {
        let zero_address = Pubkey::new_from_array([0; 32]);
        hasher.hash(&zero_address.to_bytes());
    }
    let amount_bytes = amount.to_le_bytes();
    hasher.hash(&amount_bytes);
    hasher.result()
}

pub(crate) fn calculate_hash_from_secret(secret: &[u8]) -> Hash {
    let mut hasher = Hasher::default();
    hasher.hash(secret);
    hasher.result()
}
