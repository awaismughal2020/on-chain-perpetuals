import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PerpDex } from '../target/types/perp_dex';
import {
  Keypair,
  PublicKey,
  SystemProgram,
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
} from '@solana/spl-token';
import { assert } from 'chai';

const MOCK_PYTH_PRICE_FEED = new Keypair();

async function createPythAccount(
  provider: anchor.AnchorProvider,
  price: number,
  expo: number
) {
  const pythData = Buffer.alloc(3312);
  pythData.writeInt32LE(0x50595448, 0);
  pythData.writeInt32LE(2, 4);
  pythData.writeInt32LE(1, 8);
  pythData.writeBigInt64LE(BigInt(price), 224);
  pythData.writeInt32LE(expo, 232);
  pythData.writeBigInt64LE(BigInt(Date.now() / 1000), 248);

  const tx = new anchor.web3.Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: MOCK_PYTH_PRICE_FEED.publicKey,
      space: pythData.length,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(pythData.length),
      programId: new PublicKey('gSbePebfvPy74C2pcN1gAmG3GzgPKaeNVsV4UvEzPqr'),
    })
  );

  await provider.connection.sendTransaction(tx, [MOCK_PYTH_PRICE_FEED]);
  const account = await provider.connection.getAccountInfo(MOCK_PYTH_PRICE_FEED.publicKey);
  account.data = pythData;
}

describe('perp_dex', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PerpDex as Program<PerpDex>;
  const admin = provider.wallet as anchor.Wallet;

  let usdcMint: PublicKey;
  let userCollateralAccount: PublicKey;
  let collateralVault: PublicKey;
  let programState: PublicKey;
  let userAccount: PublicKey;

  before(async () => {
    usdcMint = await createMint(provider.connection, admin.payer, admin.publicKey, null, 6);
    userCollateralAccount = await createAccount(
      provider.connection,
      admin.payer,
      usdcMint,
      admin.publicKey
    );
    await mintTo(
      provider.connection,
      admin.payer,
      usdcMint,
      userCollateralAccount,
      admin.payer,
      1000 * 10 ** 6
    );
    await createPythAccount(provider, 100 * 10 ** 8, -8);
  });

  it('Is initialized!', async () => {
    [programState] = PublicKey.findProgramAddressSync(
      [Buffer.from('perp_dex_state')],
      program.programId
    );

    [collateralVault] = PublicKey.findProgramAddressSync(
      [Buffer.from('collateral_vault'), usdcMint.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize(usdcMint)
      .accounts({
        admin: admin.publicKey,
        programState,
        usdcMint,
        collateralVault,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const state = await program.account.state.fetch(programState);
    assert.ok(state.admin.equals(admin.publicKey));
    assert.ok(state.usdcMint.equals(usdcMint));
  });

  it('Creates a market', async () => {
    const marketIndex = 0;
    const [marketKey] = PublicKey.findProgramAddressSync(
      [Buffer.from('market'), new anchor.BN(marketIndex).toArrayLike(Buffer, 'le', 2)],
      program.programId
    );

    await program.methods
      .createMarket(
        marketIndex,
        new anchor.BN('100000000000'),
        new anchor.BN('10000000000000'),
        new anchor.BN(1000),
        new anchor.BN(50000),
        new anchor.BN(100000),
        new anchor.BN(50000)
      )
      .accounts({
        admin: admin.publicKey,
        programState,
        market: marketKey,
        oraclePriceFeed: MOCK_PYTH_PRICE_FEED.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const market = await program.account.market.fetch(marketKey);
    assert.equal(market.marketIndex, marketIndex);
    assert.isTrue(market.initialized);
  });

  it('Creates a user account and deposits collateral', async () => {
    [userAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), admin.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .createUser()
      .accounts({
        authority: admin.publicKey,
        userAccount,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const depositAmount = new anchor.BN(100 * 10 ** 6);
    await program.methods
      .depositCollateral(depositAmount)
      .accounts({
        authority: admin.publicKey,
        userAccount,
        collateralVault,
        userCollateralAccount,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const user = await program.account.user.fetch(userAccount);
    assert.equal(user.collateral.toString(), depositAmount.toString());
  });

  it('Opens a long position', async () => {
    const marketIndex = 0;
    const [marketKey] = PublicKey.findProgramAddressSync(
      [Buffer.from('market'), new anchor.BN(marketIndex).toArrayLike(Buffer, 'le', 2)],
      program.programId
    );

    const baseAssetAmount = new anchor.BN('1000000000');
    const limitPrice = new anchor.BN('101000000000');

    await program.methods
      .openPosition(baseAssetAmount, limitPrice)
      .accounts({
        authority: admin.publicKey,
        userAccount,
        market: marketKey,
      })
      .remainingAccounts([
        {
          pubkey: marketKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: MOCK_PYTH_PRICE_FEED.publicKey,
          isSigner: false,
          isWritable: false,
        },
      ])
      .rpc();

    const user = await program.account.user.fetch(userAccount);
    const position = user.positions[0];
    assert.equal(position.marketIndex, marketIndex);
    assert.equal(position.baseAssetAmount.toString(), baseAssetAmount.toString());
    assert.isTrue(position.quoteAssetAmount.gtn(0));
  });
});
