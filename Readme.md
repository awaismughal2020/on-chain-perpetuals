# On-Chain Perpetuals

A decentralized perpetual futures exchange built on Solana using the Anchor framework. This project implements a perpetual futures decentralized exchange (DEX) with features like position management, funding rates, and liquidation logic.

## Prerequisites

- **Rust**: Required for compiling the smart contract code.
- **Solana Tool Suite**: Needed for interacting with the Solana blockchain.
- **Anchor**: A framework for Solana smart contract development.
- **Node.js and Yarn**: Required for running tests and managing TypeScript dependencies.

## Setup Instructions

### 1. Setup Environment

Install the required tools:

- Install Rust:
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- Install Solana Tool Suite:
  ```sh
  sh -c "$(curl -sSfL https://release.solana.com/v1.18.1/install)"
  ```
- Install Anchor:
  ```sh
  cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked --force
  ```
- Install Node.js and Yarn (refer to their official websites for installation instructions).

### 2. Build the Program

Navigate to the project root directory and compile the Rust code:

```sh
anchor build
```

This command generates:
- A BPF binary for deployment.
- An IDL file at `target/idl/perp_dex.json`.
- TypeScript types at `target/types/perp_dex.ts`.

### 3. Configure Solana CLI

Set up the Solana CLI for local testing:

- Configure the Solana cluster to use a local validator:
  ```sh
  solana config set --url localhost
  solana-test-validator
  ```

- In a separate terminal, configure your keypair and request test SOL:
  ```sh
  solana config set --keypair ~/.config/solana/id.json
  solana airdrop 100
  ```

### 4. Deploy the Program

Deploy the compiled program to the Solana blockchain:

```sh
anchor deploy
```

After deployment, Anchor updates the program ID in:
- `src/lib.rs` (via the `declare_id!` macro).
- `programs/perp_dex/Cargo.toml`.

**Note**: Copy the final program ID into the `declare_id!` macro to ensure consistency.

### 5. Run Tests

Run integration tests to verify the program's functionality against the local validator:

```sh
anchor test
```

This command builds, deploys, and executes the tests defined in `tests/perp_dex.ts`.

## Upgrading the Program

To upgrade an existing deployed program:
1. Rebuild the updated version:
   ```sh
   anchor build
   ```
2. Redeploy using the same buffer and program keypair:
   ```sh
   anchor deploy
   ```

**Important**: This program uses `zero_copy` for account serialization. To maintain data layout compatibility:
- Add new fields to the `_padding` section at the end of account structs.
- Avoid changing the order or size of existing fields, as this is a breaking change and requires a full program migration.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
