use solana_program::program::{invoke_signed, invoke};
#[allow(unused_imports)]
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    msg,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, Sysvar, rent::Rent},
    self,
};
use solana_program::borsh::try_from_slice_unchecked;
use borsh::{BorshDeserialize, BorshSerialize,BorshSchema};
use spl_token;
use spl_associated_token_account;


// Declare and export the program's entrypoint
entrypoint!(process_instruction);

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum StakeInstruction{
    GenerateVault
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct StakeData{
    staker: Pubkey,
    lock_period: u64,
    timestamp: u64,
    amount: u64,
    active: bool,
}

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let instruction: StakeInstruction = try_from_slice_unchecked(instruction_data).unwrap();
    let vault_word = "vault";

    let admin = "HRqXXua5SSsr1C7pBWhtLxjD9HcreNd4ZTKJD7em7mtP".parse::<Pubkey>().unwrap();
    let reward_mint = "9EXqHNEpmujXDtvP12QSiQDcAeCqa8KNGgTNGqTP3t9C".parse::<Pubkey>().unwrap();

    match instruction{

        StakeInstruction::GenerateVault=>{
            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let pda = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;

            if *payer.key!=admin||!payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x02));
            }

            let rent = &Rent::from_account_info(rent_info)?;

            let (vault_pda, vault_bump_seed) =
                Pubkey::find_program_address(&[vault_word.as_bytes()], &program_id);
            
            if pda.key!=&vault_pda{
                //msg!("Wrong account generated by client");
                return Err(ProgramError::Custom(0x00));
            }
            if pda.owner!=program_id{
                let size = 8*5;
           
                let required_lamports = rent
                .minimum_balance(size as usize)
                .max(1)
                .saturating_sub(pda.lamports());

                invoke(
                    &system_instruction::transfer(payer.key, &vault_pda, required_lamports),
                    &[
                        payer.clone(),
                        pda.clone(),
                        system_program.clone(),
                    ],
                )?;

                invoke_signed(
                    &system_instruction::allocate(&vault_pda, size),
                    &[
                        pda.clone(),
                        system_program.clone(),
                    ],
                    &[&[vault_word.as_bytes(), &[vault_bump_seed]]],
                )?;

                invoke_signed(
                    &system_instruction::assign(&vault_pda, program_id),
                    &[
                        pda.clone(),
                        system_program.clone(),
                    ],
                    &[&[vault_word.as_bytes(), &[vault_bump_seed]]],
                )?;
            }
        },
    };
        
    Ok(())
}


