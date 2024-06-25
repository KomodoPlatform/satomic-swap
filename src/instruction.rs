use crate::error_code::{
    INVALID_AMOUNT, INVALID_ATOMIC_SWAP_INSTRUCTION, INVALID_INPUT_LENGTH, INVALID_LOCK_TIME,
    INVALID_RECEIVER_PUBKEY, INVALID_SECRET, INVALID_SECRET_HASH, INVALID_SENDER_PUBKEY,
    INVALID_TOKEN_PROGRAM,
};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum AtomicSwapInstruction {
    LamportsPayment {
        secret_hash: [u8; 32],
        lock_time: u64,
        amount: u64,
        receiver: Pubkey,
        rent_exemption_lamports: u64,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
    },
    SPLTokenPayment {
        secret_hash: [u8; 32],
        lock_time: u64,
        amount: u64,
        receiver: Pubkey,
        token_program: Pubkey,
        rent_exemption_lamports: u64,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
    },
    ReceiverSpend {
        secret: [u8; 32],
        lock_time: u64,
        amount: u64,
        sender: Pubkey,
        token_program: Pubkey,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
    },
    SenderRefund {
        secret_hash: [u8; 32],
        lock_time: u64,
        amount: u64,
        receiver: Pubkey,
        token_program: Pubkey,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
    },
}

impl AtomicSwapInstruction {
    fn validate_common_fields(
        secret_hash: &[u8],
        lock_time: u64,
        amount: u64,
        receiver: &Pubkey,
        additional_pubkey: Option<&Pubkey>,
    ) -> Result<(), ProgramError> {
        if secret_hash.len() != 32 {
            return Err(ProgramError::Custom(INVALID_SECRET_HASH));
        }
        if lock_time == 0 {
            return Err(ProgramError::Custom(INVALID_LOCK_TIME));
        }
        if amount == 0 {
            return Err(ProgramError::Custom(INVALID_AMOUNT));
        }
        if receiver.to_bytes().len() != 32 {
            return Err(ProgramError::Custom(INVALID_RECEIVER_PUBKEY));
        }
        if let Some(pubkey) = additional_pubkey {
            if pubkey.to_bytes().len() != 32 {
                return Err(ProgramError::Custom(INVALID_TOKEN_PROGRAM));
            }
        }
        Ok(())
    }

    fn validate_secret(secret: &[u8]) -> Result<(), ProgramError> {
        if secret.len() != 32 {
            return Err(ProgramError::Custom(INVALID_SECRET));
        }
        Ok(())
    }

    pub fn unpack(
        instruction_byte: u8,
        input: &[u8],
    ) -> Result<AtomicSwapInstruction, ProgramError> {
        if input.len()
            != match instruction_byte {
                0 => 92,
                1 => 124,
                2 => 116,
                3 => 116,
                _ => return Err(ProgramError::Custom(INVALID_ATOMIC_SWAP_INSTRUCTION)),
            }
        {
            return Err(ProgramError::Custom(INVALID_INPUT_LENGTH));
        }
        let instruction = AtomicSwapInstruction::try_from_slice(&input[1..])
            .map_err(|_| ProgramError::Custom(INVALID_ATOMIC_SWAP_INSTRUCTION))?;

        match &instruction {
            AtomicSwapInstruction::LamportsPayment {
                secret_hash,
                lock_time,
                amount,
                receiver,
                rent_exemption_lamports: _,
                vault_bump_seed: _,
                vault_bump_seed_data: _,
            } => {
                Self::validate_common_fields(secret_hash, *lock_time, *amount, receiver, None)?;
            }
            AtomicSwapInstruction::SPLTokenPayment {
                secret_hash,
                lock_time,
                amount,
                receiver,
                token_program,
                rent_exemption_lamports: _,
                vault_bump_seed: _,
                vault_bump_seed_data: _,
            } => {
                Self::validate_common_fields(
                    secret_hash,
                    *lock_time,
                    *amount,
                    receiver,
                    Some(token_program),
                )?;
            }
            AtomicSwapInstruction::ReceiverSpend {
                secret,
                lock_time,
                amount,
                sender,
                token_program,
                vault_bump_seed: _,
                vault_bump_seed_data: _,
            } => {
                Self::validate_secret(secret)?;
                if sender.to_bytes().len() != 32 {
                    return Err(ProgramError::Custom(INVALID_SENDER_PUBKEY));
                }
                Self::validate_common_fields(
                    secret,
                    *lock_time,
                    *amount,
                    sender,
                    Some(token_program),
                )?;
            }
            AtomicSwapInstruction::SenderRefund {
                secret_hash,
                lock_time,
                amount,
                receiver,
                token_program,
                vault_bump_seed: _,
                vault_bump_seed_data: _,
            } => {
                Self::validate_common_fields(
                    secret_hash,
                    *lock_time,
                    *amount,
                    receiver,
                    Some(token_program),
                )?;
            }
        }

        Ok(instruction)
    }

    #[allow(dead_code)]
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = vec![match *self {
            AtomicSwapInstruction::LamportsPayment { .. } => 0,
            AtomicSwapInstruction::SPLTokenPayment { .. } => 1,
            AtomicSwapInstruction::ReceiverSpend { .. } => 2,
            AtomicSwapInstruction::SenderRefund { .. } => 3,
        }];
        buf.extend(to_vec(&self).unwrap());
        buf
    }
}
