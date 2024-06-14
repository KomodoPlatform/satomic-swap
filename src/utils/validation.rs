use crate::error_code::{AMOUNT_ZERO, INVALID_OWNER, RECEIVER_SET_TO_DEFAULT};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

pub(crate) fn validate_common_payment_params(receiver: Pubkey, amount: u64) -> ProgramResult {
    if receiver == Pubkey::default() {
        return Err(ProgramError::Custom(RECEIVER_SET_TO_DEFAULT));
    }
    if amount == 0 {
        return Err(ProgramError::Custom(AMOUNT_ZERO));
    }

    Ok(())
}

pub(crate) fn validate_accounts<'a, 'b>(
    sender_account: &'a AccountInfo<'b>,
    vault_pda_data: &'a AccountInfo<'b>,
    vault_pda: &'a AccountInfo<'b>,
    system_program_account: &'a AccountInfo<'b>,
    program_id: Option<&Pubkey>,
) -> ProgramResult {
    if !sender_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !vault_pda_data.is_writable || !vault_pda.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_pda.owner != &system_program::ID
        || !system_program::check_id(system_program_account.key)
    {
        return Err(ProgramError::IncorrectProgramId);
    }
    if let Some(pid) = program_id {
        if vault_pda_data.owner != pid {
            return Err(ProgramError::Custom(INVALID_OWNER));
        }
    }

    Ok(())
}
