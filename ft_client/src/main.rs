use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Signer};
#[allow(unused_imports)]
use solana_sdk::signer::signers::Signers;
use solana_sdk::transaction::Transaction;
use solana_sdk::system_program;
use borsh::{BorshDeserialize, BorshSerialize,BorshSchema};
use solana_sdk::commitment_config::CommitmentConfig;
use spl_token;
use spl_associated_token_account;
#[allow(unused_imports)]
use solana_sdk::signer::keypair::Keypair;
#[allow(unused_imports)]
use solana_sdk::borsh::try_from_slice_unchecked;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum StakeInstruction{
    GenerateVault{
        #[allow(dead_code)]
        min_lock_period:u64,
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


fn main() {
    let matches = app_from_crate!()
        .subcommand(SubCommand::with_name("generate_vault_address")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("min_lock_period")
                .long("min_lock_period")
                .required(true)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("stake")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("amount")
                .short("a")
                .long("amount")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("lock_period")
                .short("l")
                .long("lock_period")
                .required(true)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("unstake")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
        )
        .get_matches();

    let program_id = "EJMGTrytpzA7K4u1qVgPpfBHWUTbwWFJeqZiSWJEkgmQ".parse::<Pubkey>().unwrap();
    let reward_mint = "AFBVFKJ6SZaNC2dgxaR2UyhZ91exHF7NHgwhJNp1nqHe".parse::<Pubkey>().unwrap();

    if let Some(matches) = matches.subcommand_matches("unstake") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&"vault".as_bytes()], &program_id);
        let destanation = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &reward_mint);
        let source = spl_associated_token_account::get_associated_token_address(&vault, &reward_mint);
        let reward_destanation = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &reward_mint);
        // let reward_source = spl_associated_token_account::get_associated_token_address(&vault, &reward_mint);
        let ( stake_data, _ ) = Pubkey::find_program_address(&[&wallet_pubkey.to_bytes(),&reward_mint.to_bytes()], &program_id);


        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Unstake,
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(reward_mint, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new(stake_data, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new(reward_destanation, false),
                // AccountMeta::new(reward_source, false),
                AccountMeta::new(destanation, false),
                AccountMeta::new(source, false),
                // AccountMeta::new_readonly(metadata, false),
                // AccountMeta::new(wl_data_address, false),
                AccountMeta::new(reward_mint, false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("stake") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let amount = matches.value_of("amount").unwrap().parse::<u64>().unwrap();
        let lock_period = matches.value_of("lock_period").unwrap().parse::<u64>().unwrap();

        let ( stake_data, _stake_data_bump ) = Pubkey::find_program_address(&[&wallet_pubkey.to_bytes(),&reward_mint.to_bytes()], &program_id);
        let (vault_pda, _) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);
        let source = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &reward_mint);
        let destanation = spl_associated_token_account::get_associated_token_address(&vault_pda, &reward_mint);

        let accounts = vec![
            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new_readonly(reward_mint, false),

            AccountMeta::new_readonly(vault_pda, false),
            AccountMeta::new(source, false),
            AccountMeta::new(destanation, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
            AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),

            AccountMeta::new(stake_data, false),
        ];
        // println!("{:#?}",&accounts);

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Stake{
                amount,
                lock_period,
            },
            accounts,
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("generate_vault_address") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let (vault_pda, _) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);

        let min_lock_period = matches.value_of("min_lock_period").unwrap().parse::<u64>().unwrap();

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::GenerateVault{
                min_lock_period,
            },
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("vault account generated: {:?}", vault_pda);
        println!("tx id: {:?}", id);
    }
}
