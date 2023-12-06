# Solana Token Contract

This repository contains the smart contract and client-side tools for creating and trasnferring tokens in a contract vault.

## Token Creation

1. **Create Token:**
    ```bash
    spl-token create-token --decimals
    ```

2. **Create Token Account:**
    ```bash
    spl-token create-account <token>
    ```

3. **Mint Tokens:**
    ```bash
    spl-token mint <token>
    ```
4. **Check Tokens balance:**
    ```bash
    spl-token balance <token>
    ```
## Contract Configuration

1. **Admin Key Pair:**
    - Add your key pair public key as the controller of the vault generation in the contract.

2. **Reward Mint:**
    - Add the token that the contract holds and gives as a reward.

## Deploy Contract

1. **Compile:**
    ```bash
    cargo build-bpf
    ```

2. **Deploy Contract:**
    ```bash
    solana program deploy target/deploy/SplTokenSolana.so 
    ```

## Client Side

1. **Edit and Compile Client Side:**
    - Change the `program_id`.
    - Change the `reward_mint`.

2. **Vault Generation (Client Side):**

    a. **Generate Vault Address (with Admin Key Pair):**
        ```bash
        ./target/debug/tokens-client generate_vault_address -e dev -s <keypair.json> --min_lock_period <seconds>
        ```
        i.e
        ./target/debug/tokens-client generate_vault_address -e dev -s devnet-test.json --min_lock_period 1

    b. **Pay Rent:**
        ```bash
        ./target/debug/tokens-client pay_rent -s <keypair.json> -e dev -a <token-amount> -l 2
        ```
     b. **Divide Rent:**
        ```bash
        ./target/debug/tokens-client divide_rent -e dev -s devnet-test.json --token_balance 2 --json owner_list.json
        ```

**Note:** Replace placeholders such as `<token>`, `<keypair.json>`, `<token-amount>`, etc., with actual values.

