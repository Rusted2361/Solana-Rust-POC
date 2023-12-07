use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand,
};
use solana_client::rpc_client::{RpcClient};
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
use std::fs;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum StakeInstruction{
    GenerateVault,
    PayRent,
    DivideRent{
        #[allow(dead_code)]
        owners_share:u64,
    },
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
        )
        .subcommand(SubCommand::with_name("pay_rent")
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
        .subcommand(SubCommand::with_name("divide_rent")
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
            // .arg(Arg::with_name("token_balance")
            //     .short("tb")
            //     .long("token_balance")
            //     .required(true)
            //     .takes_value(true)
            // )
            .arg(Arg::with_name("json")
                .short("j")
                .long("json")
                .required(true)
                .takes_value(true)
            )
        )
        .get_matches();

    let program_id = "8jPy71sq7e4sueLqy4QtzRfXhqHwahEjpr1fu9aMn3HW".parse::<Pubkey>().unwrap();
    let reward_mint = "H9qtPoMgHYoyjmKxPnQDdxZiL4fuNijHaGnE3sMCPbdV".parse::<Pubkey>().unwrap();
    
    if let Some(matches) = matches.subcommand_matches("divide_rent") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();


        let destanation = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &reward_mint);
        let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&"vault".as_bytes()], &program_id);
        let source = spl_associated_token_account::get_associated_token_address(&vault, &reward_mint);
        // let reward_source = spl_associated_token_account::get_associated_token_address(&vault, &reward_mint);
        let ( stake_data, _ ) = Pubkey::find_program_address(&[&wallet_pubkey.to_bytes(),&reward_mint.to_bytes()], &program_id);
        //let token_balance = matches.value_of("token_balance").unwrap().parse::<u64>().unwrap()*1000000000;
        let config_path = matches.value_of("json").unwrap();
        let config = fs::read_to_string(&config_path).expect("Unable to read file"); 
        let whitelist: Vec<String> = serde_json::from_str(&config).expect("Wrong json format");
        println!("Total wallet addresses in whitelist: {}", whitelist.len()as u64);
        let usd = 1000000000;
        let owners_share = usd/whitelist.len()as u64;
        println!("owners_share: {}", owners_share);
        for (i,address_str) in whitelist.iter().enumerate(){
            let land_owner = address_str.parse::<Pubkey>().expect("Wrong json address format");
            let land_owner_ata = spl_associated_token_account::get_associated_token_address(&land_owner, &reward_mint);
            let instarctions = vec![Instruction::new_with_borsh(
                program_id,
                &StakeInstruction::DivideRent{
                    owners_share,
                },
                vec![
                    AccountMeta::new_readonly(system_program::id(), false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                    AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                    AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),
                    AccountMeta::new_readonly(reward_mint, false),
                    // AccountMeta::new(stake_data, false),
                    
                    AccountMeta::new_readonly(land_owner, false),//land_owner pubkey
                    AccountMeta::new(land_owner_ata, false),//land_owner ata
                    AccountMeta::new(wallet_pubkey, true),//user pubkey
                    AccountMeta::new(destanation, false),//user ata
                    AccountMeta::new_readonly(vault, false),//pda vault
                    AccountMeta::new(source, false),//pda ata
                ],
            )];
            let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
            let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
            tx.sign(&vec![&wallet_keypair], recent_blockhash);
            let id = client.send_transaction(&tx).expect("Transaction failed.");
            //println!("tx id: {:?}", id);
            println!("{:?}) {} Success. Check transaction: {:?}",i+1,address_str,id);
    }
    }

    if let Some(matches) = matches.subcommand_matches("pay_rent") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

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
            &StakeInstruction::PayRent,
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

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::GenerateVault,
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