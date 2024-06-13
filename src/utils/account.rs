use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

use crate::STORAGE_SPACE_ALLOCATED;

pub(crate) fn get_common_accounts<'a, 'b>(
    accounts: &'a [AccountInfo<'b>],
) -> Result<
    (
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
    ),
    ProgramError,
> {
    let accounts_iter = &mut accounts.iter();
    let sender_account = next_account_info(accounts_iter)?;
    let vault_pda_data = next_account_info(accounts_iter)?;
    let vault_pda = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    Ok((
        sender_account,
        vault_pda_data,
        vault_pda,
        system_program_account,
    ))
}

pub(crate) fn create_account_and_write_data<'a, 'b>(
    sender_account: &'a AccountInfo<'b>,
    vault_pda_data: &'a AccountInfo<'b>,
    rent_exemption_lamports: u64,
    program_id: &Pubkey,
    system_program_account: &'a AccountInfo<'b>,
    vault_seeds_data: &[&[u8]],
    payment_bytes: Vec<u8>,
) -> ProgramResult {
    let create_instruction = system_instruction::create_account(
        sender_account.key,
        vault_pda_data.key,
        rent_exemption_lamports,
        STORAGE_SPACE_ALLOCATED,
        program_id,
    );
    let account_infos = vec![
        sender_account.clone(),
        vault_pda_data.clone(),
        system_program_account.clone(),
    ];
    invoke_signed(&create_instruction, &account_infos, &[vault_seeds_data])?;
    let data = &mut vault_pda_data.try_borrow_mut_data()?;
    if data.len() < payment_bytes.len() {
        return Err(ProgramError::AccountDataTooSmall);
    }
    data[..payment_bytes.len()].copy_from_slice(&payment_bytes);

    Ok(())
}
