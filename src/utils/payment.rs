use crate::error_code::{
    INVALID_PAYMENT_HASH, INVALID_PAYMENT_STATE, SWAP_ACCOUNT_NOT_FOUND,
};
use crate::payment::{Payment, PaymentState};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    system_instruction,
};

pub(crate) fn create_payment_object(payment_hash: solana_program::hash::Hash, lock_time: u64) -> Payment {
    Payment {
        payment_hash: payment_hash.to_bytes(),
        lock_time,
        state: PaymentState::PaymentSent,
    }
}

pub(crate) fn transfer_lamports<'a, 'b>(
    source_account: &'a AccountInfo<'b>,
    destination_account: &'a AccountInfo<'b>,
    system_program_account: &'a AccountInfo<'b>,
    vault_seeds: &[&[u8]],
    amount: u64,
) -> ProgramResult {
    let transfer_instruction = system_instruction::transfer(source_account.key, destination_account.key, amount);
    let account_infos = vec![
        source_account.clone(),
        destination_account.clone(),
        system_program_account.clone(),
    ];
    invoke_signed(&transfer_instruction, &account_infos, &[vault_seeds])?;

    Ok(())
}

pub(crate) fn update_common_payment_state(
    vault_pda_data: &AccountInfo,
    payment_hash: solana_program::hash::Hash,
    expected_state: PaymentState,
    new_state: PaymentState,
) -> ProgramResult {
    let swap_account_data = &mut vault_pda_data
        .try_borrow_mut_data()
        .map_err(|_| ProgramError::Custom(SWAP_ACCOUNT_NOT_FOUND))?;
    let mut swap_payment = Payment::unpack(swap_account_data)?;
    if swap_payment.payment_hash != payment_hash.to_bytes() {
        return Err(ProgramError::Custom(INVALID_PAYMENT_HASH));
    }
    if swap_payment.state != expected_state {
        return Err(ProgramError::Custom(INVALID_PAYMENT_STATE));
    }
    swap_payment.state = new_state;
    let payment_bytes = swap_payment.pack();
    if swap_account_data.len() < payment_bytes.len() {
        return Err(ProgramError::AccountDataTooSmall);
    }
    swap_account_data[..payment_bytes.len()].copy_from_slice(&payment_bytes);

    Ok(())
}
