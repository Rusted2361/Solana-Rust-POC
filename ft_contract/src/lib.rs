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
    GenerateVault,
    pay_rent,
    divide_rent{
        #[allow(dead_code)]
        owners_share:u64,
    }
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
    let usd_token = "H9qtPoMgHYoyjmKxPnQDdxZiL4fuNijHaGnE3sMCPbdV".parse::<Pubkey>().unwrap();
    match instruction{
        StakeInstruction::divide_rent{owners_share}=>{
            let system_program = next_account_info(accounts_iter)?;//system program account
            let token_info = next_account_info(accounts_iter)?;//token program account
            let rent_info = next_account_info(accounts_iter)?;//rent program account
            let assoc_acccount_info = next_account_info(accounts_iter)?;//associated token program account
            let mint_info = next_account_info(accounts_iter)?;//USD token
            //let stake_info = next_account_info(accounts_iter)?;
            
            let land_owner = next_account_info(accounts_iter)?;//land_owner pubkey
            let land_owner_ata = next_account_info(accounts_iter)?;//land_owner ata
            let payer = next_account_info(accounts_iter)?;//user pubkey
            let payer_mint_holder_info = next_account_info(accounts_iter)?;//user ata
            let vault_info = next_account_info(accounts_iter)?;//pda vault
            let vault_mint_holder_info = next_account_info(accounts_iter)?;//pda ata

            //let ( stake_address, _stake_bump ) = Pubkey::find_program_address(&[&payer.key.to_bytes(),&mint_info.key.to_bytes()], &program_id);
            let ( vault_address, vault_bump ) = Pubkey::find_program_address(&[&vault_word.as_bytes()], &program_id);
            //let payer_reward_holder = spl_associated_token_account::get_associated_token_address(payer.key, &usd_token);
            let payer_mint_holder = spl_associated_token_account::get_associated_token_address(payer.key, mint_info.key);
            let vault_mint_holder = spl_associated_token_account::get_associated_token_address(vault_info.key, mint_info.key);


            if *mint_info.key!=usd_token{
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

            // if stake_address!=*stake_info.key{
            //     //wrong stake_info
            //     return Err(ProgramError::Custom(0x60));
            // }

            if vault_address!=*vault_info.key{
                //wrong stake_info
                return Err(ProgramError::Custom(0x61));
            }

            if payer_mint_holder!=*payer_mint_holder_info.key{
                //wrong payer_mint_holder_info
                return Err(ProgramError::Custom(0x64));
            }

            if vault_mint_holder!=*vault_mint_holder_info.key{
                //wrong vault_mint_holder_info
                return Err(ProgramError::Custom(0x65));
            }


            if usd_token!=*mint_info.key{
                //wrong mint_info
                return Err(ProgramError::Custom(0x67));
            }

            // let mut stake_data = if let Ok(data) = StakeData::try_from_slice(&stake_info.data.borrow()){
            //     data
            // } else {
            //     // can't deserialize stake data
            //     return Err(ProgramError::Custom(0x913));
            // };


            // if stake_data.staker!=*payer.key{
            //     //unauthorized access
            //     return Err(ProgramError::Custom(0x108));
            // }

            if payer_mint_holder_info.owner != token_info.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,//funding address
                        land_owner.key,//wallet address
                        mint_info.key,//usd token
                    ),
                    &[
                        payer.clone(), //Funding address
                        land_owner_ata.clone(), //destination ata account
                        land_owner.clone(),//Wallet address
                        mint_info.clone(),//usd token account
                        system_program.clone(),//system program account
                        token_info.clone(),//token program account
                        rent_info.clone(),//rent program account
                        assoc_acccount_info.clone(),//associated token program account
                    ],

                )?;
            }

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_info.key,
                    vault_mint_holder_info.key,
                    land_owner_ata.key,
                    vault_info.key,
                    &[],
                    owners_share
                )?,
                &[
                    vault_mint_holder_info.clone(),
                    land_owner_ata.clone(),
                    vault_info.clone(), 
                    token_info.clone()
                ],
                &[&[&vault_word.as_bytes(), &[vault_bump]]],
            )?;

            // if payer_mint_holder_info.owner != token_info.key{
            //     invoke(
            //         &spl_associated_token_account::create_associated_token_account(
            //             payer.key,
            //             payer.key,
            //             mint_info.key,
            //         ),
            //         &[
            //             payer.clone(), 
            //             payer_mint_holder_info.clone(), //destination ata account
            //             payer.clone(),
            //             mint_info.clone(),
            //             system_program.clone(),
            //             token_info.clone(),
            //             rent_info.clone(),
            //             assoc_acccount_info.clone(),
            //         ],

            //     )?;
            // }

            // invoke_signed(
            //     &spl_token::instruction::transfer(
            //         token_info.key,
            //         vault_mint_holder_info.key,
            //         payer_mint_holder_info.key,
            //         vault_info.key,
            //         &[],
            //         owners_share
            //     )?,
            //     &[
            //         vault_mint_holder_info.clone(),
            //         payer_mint_holder_info.clone(),
            //         vault_info.clone(), 
            //         token_info.clone()
            //     ],
            //     &[&[&vault_word.as_bytes(), &[vault_bump]]],
            // )?;
            msg!("token_balance is {}",owners_share);
            msg!("land_owner is {}",land_owner.key);
            msg!("land_owner_ata is {}",land_owner_ata.key);
        },
        StakeInstruction::pay_rent=>{
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

            if *mint.key!=usd_token{
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
                // if data.active{
                //     //staking is already active
                //     return Err(ProgramError::Custom(0x904));
                // }
            };

            // let stake_struct = StakeData{
            //     staker: *payer.key,
            //     lock_period: lock_period,
            //     timestamp: clock.unix_timestamp as u64,
            //     amount: amount,
            //     active: true,
            // };
            // stake_struct.serialize(&mut &mut stake_data_info.data.borrow_mut()[..])?;


            let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&vault_word.as_bytes()], &program_id);
            if vault != *vault_info.key{
                //msg!("Wrong vault");
                return Err(ProgramError::Custom(0x07));
            }

            // let rate_data = if let Ok(data) = RateData::try_from_slice(&vault_info.data.borrow()){
            //     data
            // } else {
            //     // can't deserialize rate data
            //     return Err(ProgramError::Custom(0x911));
            // };

            // if rate_data.min_lock_period>lock_period{
            //     // lock period too short
            //     return Err(ProgramError::Custom(0x811));
            // }
            // if amount < 2000 {
            //     panic!("you are staking than least amount");
            //    }
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
                    1000000000,
                )?,
                &[
                    source.clone(),
                    destination.clone(),
                    payer.clone(), 
                    token_program.clone()
                ],
            )?;
        },
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
            // let rate_struct = RateData{
            //     min_lock_period,
            // };
            // rate_struct.serialize(&mut &mut pda.data.borrow_mut()[..])?;
        },
    };
        
    Ok(())
}


