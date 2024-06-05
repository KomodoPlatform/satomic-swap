use crate::error_code::NOT_SUPPORTED;
use crate::instruction::AtomicSwapInstruction;
use crate::payment::PaymentState;
use crate::utils::{
    account::{create_account_and_write_data, get_common_accounts},
    hash::{calculate_hash, calculate_hash_from_secret},
    payment::{create_payment_object, transfer_lamports, update_common_payment_state},
    validation::{validate_common_payment_params, validate_accounts},
};
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AtomicSwapInstruction::unpack(instruction_data[0], instruction_data)?;
    match instruction {
        AtomicSwapInstruction::LamportsPayment {
            secret_hash,
            lock_time,
            amount,
            receiver,
            rent_exemption_lamports,
            vault_bump_seed,
            vault_bump_seed_data,
        } => {
            validate_common_payment_params(receiver, amount)?;
            let (sender_account, vault_pda_data, vault_pda, system_program_account) = get_common_accounts(accounts)?;
            validate_accounts(sender_account, vault_pda_data, vault_pda, system_program_account, None)?;
            let vault_seeds: &[&[u8]] = &[
                b"swap",
                &lock_time.to_le_bytes()[..],
                &secret_hash[..],
                &[vault_bump_seed],
            ];
            let vault_seeds_data: &[&[u8]] = &[
                b"swap_data",
                &lock_time.to_le_bytes()[..],
                &secret_hash[..],
                &[vault_bump_seed_data],
            ];
            let payment_hash = calculate_hash(&receiver, sender_account.key, &secret_hash, None, amount);
            let payment_bytes = create_payment_object(payment_hash, lock_time).pack();
            create_account_and_write_data(
                sender_account,
                vault_pda_data,
                rent_exemption_lamports,
                program_id,
                system_program_account,
                vault_seeds_data,
                payment_bytes,
            )?;
            transfer_lamports(
                sender_account,
                vault_pda,
                system_program_account,
                vault_seeds,
                amount + rent_exemption_lamports,
            )?;

            Ok(())
        }
        AtomicSwapInstruction::SPLTokenPayment {
            secret_hash,
            lock_time,
            amount,
            receiver,
            token_program,
            rent_exemption_lamports,
            vault_bump_seed: _,
            vault_bump_seed_data,
        } => {
            validate_common_payment_params(receiver, amount)?;
            let (sender_account, vault_pda_data, vault_pda, system_program_account) = get_common_accounts(accounts)?;
            validate_accounts(sender_account, vault_pda_data, vault_pda, system_program_account, None)?;
            let vault_seeds_data: &[&[u8]] = &[
                b"swap_data",
                &lock_time.to_le_bytes()[..],
                &secret_hash[..],
                &[vault_bump_seed_data],
            ];
            let payment_hash = calculate_hash(&receiver, sender_account.key, &secret_hash, Some(&token_program), amount);
            let payment_bytes = create_payment_object(payment_hash, lock_time).pack();
            create_account_and_write_data(
                sender_account,
                vault_pda_data,
                rent_exemption_lamports,
                program_id,
                system_program_account,
                vault_seeds_data,
                payment_bytes,
            )?;

            Ok(())
        }
        AtomicSwapInstruction::ReceiverSpend {
            secret,
            lock_time,
            amount,
            sender,
            token_program,
            vault_bump_seed,
            vault_bump_seed_data: _,
        } => {
            let (receiver_account, vault_pda_data, vault_pda, system_program_account) = get_common_accounts(accounts)?;
            validate_accounts(receiver_account, vault_pda_data, vault_pda, system_program_account, Some(program_id))?;
            let secret_hash = calculate_hash_from_secret(&secret);
            let payment_hash = calculate_hash(receiver_account.key, &sender, &secret_hash.to_bytes(), Some(&token_program), amount);
            let vault_seeds: &[&[u8]] = &[
                b"swap",
                &lock_time.to_le_bytes()[..],
                &secret_hash.to_bytes()[..],
                &[vault_bump_seed],
            ];
            update_common_payment_state(vault_pda_data, payment_hash, PaymentState::PaymentSent, PaymentState::ReceiverSpent)?;
            if token_program != Pubkey::new_from_array([0; 32]) {
                return Err(ProgramError::Custom(NOT_SUPPORTED));
            }
            transfer_lamports(vault_pda, receiver_account, system_program_account, vault_seeds, amount)?;

            Ok(())
        }
        AtomicSwapInstruction::SenderRefund {
            secret_hash,
            lock_time,
            amount,
            receiver,
            token_program,
            vault_bump_seed,
            vault_bump_seed_data: _,
        } => {
            let (sender_account, vault_pda_data, vault_pda, system_program_account) = get_common_accounts(accounts)?;
            validate_accounts(sender_account, vault_pda_data, vault_pda, system_program_account, Some(program_id))?;
            let payment_hash = calculate_hash(&receiver, sender_account.key, &secret_hash, Some(&token_program), amount);
            let vault_seeds: &[&[u8]] = &[
                b"swap",
                &lock_time.to_le_bytes()[..],
                &secret_hash[..],
                &[vault_bump_seed],
            ];
            update_common_payment_state(vault_pda_data, payment_hash, PaymentState::PaymentSent, PaymentState::SenderRefunded)?;
            if token_program != Pubkey::new_from_array([0; 32]) {
                return Err(ProgramError::Custom(NOT_SUPPORTED));
            }
            transfer_lamports(vault_pda, sender_account, system_program_account, vault_seeds, amount)?;

            Ok(())
        }
    }
}
