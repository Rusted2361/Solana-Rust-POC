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
    GenerateVault{
        #[allow(dead_code)]
        min_lock_period:u64,
        #[allow(dead_code)]
        time_apy:u64,
        #[allow(dead_code)]
        apy_inc_period:u64,
        #[allow(dead_code)]
        apy_per_amount:u64,
        #[allow(dead_code)]
        tier_amount:u64,
    },
    Stake{
        #[allow(dead_code)]
        amount:u64,
        #[allow(dead_code)]
        lock_period:u64,
    },
    Unstake,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct StakeData{
    staker: Pubkey,
    lock_period: u64,
    timestamp: u64,
    amount: u64,
    active: bool,
}


#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct RateData{
    min_lock_period:u64,
    time_apy:u64,
    apy_inc_period:u64,
    apy_per_amount:u64,
    tier_amount:u64,
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
    let reward_mint = "J59XyaPv5eXsVi8BydxuNiqHg9rXqnGSArV46hZwHpDs".parse::<Pubkey>().unwrap();

    match instruction{
        StakeInstruction::Unstake=>{
            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let mint_info = next_account_info(accounts_iter)?;
            let token_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let assoc_acccount_info = next_account_info(accounts_iter)?;
            let stake_info = next_account_info(accounts_iter)?;
            let vault_info = next_account_info(accounts_iter)?;
            let payer_reward_holder_info = next_account_info(accounts_iter)?;
            let payer_mint_holder_info = next_account_info(accounts_iter)?;
            let vault_mint_holder_info = next_account_info(accounts_iter)?;
            let reward_mint_info = next_account_info(accounts_iter)?;

            let clock = Clock::get()?;

            let ( stake_address, _stake_bump ) = Pubkey::find_program_address(&[&payer.key.to_bytes(),&mint_info.key.to_bytes()], &program_id);
            let ( vault_address, vault_bump ) = Pubkey::find_program_address(&[&vault_word.as_bytes()], &program_id);
            let payer_reward_holder = spl_associated_token_account::get_associated_token_address(payer.key, &reward_mint);
            let payer_mint_holder = spl_associated_token_account::get_associated_token_address(payer.key, mint_info.key);
            let vault_mint_holder = spl_associated_token_account::get_associated_token_address(vault_info.key, mint_info.key);
            

            if *mint_info.key!=reward_mint{
                //unauthorized access
                return Err(ProgramError::Custom(0x333));
            }

            if !payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x11));
            }
            
            if *token_info.key!=spl_token::id(){
                //wrong token_info
                return Err(ProgramError::Custom(0x345));
            }

            if stake_address!=*stake_info.key{
                //wrong stake_info
                return Err(ProgramError::Custom(0x60));
            }

            if vault_address!=*vault_info.key{
                //wrong stake_info
                return Err(ProgramError::Custom(0x61));
            }

            if payer_reward_holder!=*payer_reward_holder_info.key{
                //wrong payer_reward_holder_info
                return Err(ProgramError::Custom(0x62));
            }


            if payer_mint_holder!=*payer_mint_holder_info.key{
                //wrong payer_mint_holder_info
                return Err(ProgramError::Custom(0x64));
            }

            if vault_mint_holder!=*vault_mint_holder_info.key{
                //wrong vault_mint_holder_info
                return Err(ProgramError::Custom(0x65));
            }
       

            if reward_mint!=*reward_mint_info.key{
                //wrong reward_mint_info
                return Err(ProgramError::Custom(0x67));
            }


            let rate = if let Ok(data) = RateData::try_from_slice(&vault_info.data.borrow()){
                data
            } else {
                // can't deserialize rate data
                return Err(ProgramError::Custom(0x911));
            };

            let mut stake_data = if let Ok(data) = StakeData::try_from_slice(&stake_info.data.borrow()){
                data
            } else {
                // can't deserialize stake data
                return Err(ProgramError::Custom(0x913));
            };


            if !stake_data.active{
                //staking is inactive
                return Err(ProgramError::Custom(0x107));
            }

            if stake_data.staker!=*payer.key{
                //unauthorized access
                return Err(ProgramError::Custom(0x108));
            }

            if clock.unix_timestamp as u64-stake_data.timestamp < stake_data.lock_period{
                //can't unstake because minimal period of staking is not reached yet
                return Err(ProgramError::Custom(0x109));
            }

            msg!("seconds passed {:?}",clock.unix_timestamp as u64-stake_data.timestamp);//seconds_passed
            msg!("time period passed {:?}",(clock.unix_timestamp as u64-stake_data.timestamp)/rate.apy_inc_period);//seconds_passed/apy_increament_period
            let time_apy = (clock.unix_timestamp as u64-stake_data.timestamp)/rate.apy_inc_period*rate.time_apy;
            msg!("time_apy {:?}",time_apy);
            let amount_apy = stake_data.amount/rate.tier_amount*rate.apy_per_amount;
            msg!("amount_apy {:?}",amount_apy);

            let total_reward = stake_data.amount*(time_apy+amount_apy)/100;


            if payer_reward_holder_info.owner != token_info.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        payer.key,
                        reward_mint_info.key,
                    ),
                    &[
                        payer.clone(), 
                        payer_reward_holder_info.clone(), 
                        payer.clone(),
                        reward_mint_info.clone(),
                        system_program.clone(),
                        token_info.clone(),
                        rent_info.clone(),
                        assoc_acccount_info.clone(),
                    ],
                    
                )?;
            }

            invoke_signed(
                &spl_token::instruction::mint_to(
                    token_info.key,
                    reward_mint_info.key,
                    payer_reward_holder_info.key,
                    vault_info.key,
                    &[],
                    total_reward,
                )?,
                &[
                    reward_mint_info.clone(),
                    payer_reward_holder_info.clone(),
                    vault_info.clone(), 
                    token_info.clone()
                ],
                &[&[&vault_word.as_bytes(), &[vault_bump]]],
            )?;


            if payer_mint_holder_info.owner != token_info.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        payer.key,
                        mint_info.key,
                    ),
                    &[
                        payer.clone(), 
                        payer_mint_holder_info.clone(), 
                        payer.clone(),
                        mint_info.clone(),
                        system_program.clone(),
                        token_info.clone(),
                        rent_info.clone(),
                        assoc_acccount_info.clone(),
                    ],
                    
                )?;
            }

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_info.key,
                    vault_mint_holder_info.key,
                    payer_mint_holder_info.key,
                    vault_info.key,
                    &[],
                    stake_data.amount,
                )?,
                &[
                    vault_mint_holder_info.clone(),
                    payer_mint_holder_info.clone(),
                    vault_info.clone(), 
                    token_info.clone()
                ],
                &[&[&vault_word.as_bytes(), &[vault_bump]]],
            )?;

            stake_data.active=false;
            stake_data.serialize(&mut &mut stake_info.data.borrow_mut()[..])?;
        },
        StakeInstruction::Stake{amount,lock_period}=>{
            let payer = next_account_info(accounts_iter)?;
            let mint = next_account_info(accounts_iter)?;
            // let metadata_account_info = next_account_info(accounts_iter)?;
            
            let vault_info = next_account_info(accounts_iter)?;
            let source = next_account_info(accounts_iter)?;
            let destination = next_account_info(accounts_iter)?;

            let token_program = next_account_info(accounts_iter)?;
            let sys_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_assoc = next_account_info(accounts_iter)?;
            
            let stake_data_info = next_account_info(accounts_iter)?;
            // let whitelist_info = next_account_info(accounts_iter)?;

            let clock = Clock::get()?;

            if *mint.key!=reward_mint{
                //wrong mint
                return Err(ProgramError::Custom(0x800));
            }

            if *token_program.key!=spl_token::id(){
                //wrong token_info
                return Err(ProgramError::Custom(0x345));
            }

            let rent = &Rent::from_account_info(rent_info)?;
            let ( stake_data, stake_data_bump ) = Pubkey::find_program_address(&[&payer.key.to_bytes(),&mint.key.to_bytes()], &program_id);

            if !payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x11));
            }

            if stake_data!=*stake_data_info.key{
                //msg!("invalid stake_data account!");
                return Err(ProgramError::Custom(0x10));
            }

            let size: u64 = 32+8+8+8+1;
            if stake_data_info.owner != program_id{
                let required_lamports = rent
                .minimum_balance(size as usize)
                .max(1)
                .saturating_sub(stake_data_info.lamports());
                invoke(
                    &system_instruction::transfer(payer.key, &stake_data, required_lamports),
                    &[
                        payer.clone(),
                        stake_data_info.clone(),
                        sys_info.clone(),
                    ],
                )?;
                invoke_signed(
                    &system_instruction::allocate(&stake_data, size),
                    &[
                        stake_data_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[&payer.key.to_bytes(),&mint.key.to_bytes(), &[stake_data_bump]]],
                )?;

                invoke_signed(
                    &system_instruction::assign(&stake_data, program_id),
                    &[
                        stake_data_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[&payer.key.to_bytes(),&mint.key.to_bytes(), &[stake_data_bump]]],
                )?;
            }

            if let Ok(data) = StakeData::try_from_slice(&stake_data_info.data.borrow()){
                if data.active{
                    //staking is already active
                    return Err(ProgramError::Custom(0x904));
                }
            };

            let stake_struct = StakeData{
                staker: *payer.key,
                lock_period: lock_period,
                timestamp: clock.unix_timestamp as u64,
                amount: amount,
                active: true,
            };
            stake_struct.serialize(&mut &mut stake_data_info.data.borrow_mut()[..])?;


            let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&vault_word.as_bytes()], &program_id);
            if vault != *vault_info.key{
                //msg!("Wrong vault");
                return Err(ProgramError::Custom(0x07));
            }

            let rate_data = if let Ok(data) = RateData::try_from_slice(&vault_info.data.borrow()){
                data
            } else {
                // can't deserialize rate data
                return Err(ProgramError::Custom(0x911));
            };

            if rate_data.min_lock_period>lock_period{
                // lock period too short
                return Err(ProgramError::Custom(0x811));
            }

            if &spl_associated_token_account::get_associated_token_address(payer.key, mint.key) != source.key {
                // msg!("Wrong source");
                return Err(ProgramError::Custom(0x08));
            }

            if &spl_associated_token_account::get_associated_token_address(&vault, mint.key) != destination.key{
                //msg!("Wrong destination");
                return Err(ProgramError::Custom(0x09));
            }

            if destination.owner != token_program.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        vault_info.key,
                        mint.key,
                    ),
                    &[
                        payer.clone(), 
                        destination.clone(), 
                        vault_info.clone(),
                        mint.clone(),
                        sys_info.clone(),
                        token_program.clone(),
                        rent_info.clone(),
                        token_assoc.clone(),
                    ],
                )?;
            }
            invoke(
                &spl_token::instruction::transfer(
                    token_program.key,
                    source.key,
                    destination.key,
                    payer.key,
                    &[],
                    amount,
                )?,
                &[
                    source.clone(),
                    destination.clone(),
                    payer.clone(), 
                    token_program.clone()
                ],
            )?;
        },

        StakeInstruction::GenerateVault{
            min_lock_period,
            time_apy,
            apy_inc_period,
            apy_per_amount,
            tier_amount,
        }=>{
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
            let rate_struct = RateData{
                min_lock_period,
                time_apy,
                apy_inc_period,
                apy_per_amount,
                tier_amount,
            };
            rate_struct.serialize(&mut &mut pda.data.borrow_mut()[..])?;
        },
    };
        
    Ok(())
}


