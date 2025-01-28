import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  Account,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintToChecked,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { BN } from "bn.js";
import { assert } from "chai";
import { before } from "mocha";
import { AnchorEscrow } from "../target/types/anchor_escrow";

const airdropSOL = async (
  to: anchor.web3.PublicKey,
  provider: anchor.AnchorProvider,
  amount: number
) => {
  try {
    const tx = await provider.connection.requestAirdrop(
      to,
      anchor.web3.LAMPORTS_PER_SOL * amount
    );

    await provider.connection.confirmTransaction(tx, "confirmed");
  } catch (e) {
    console.log(`U got an error while trying to airdrop 'SOL: ${e}`);
  }
};

const mint_account = async (
  provider: anchor.AnchorProvider,
  payer: anchor.web3.Keypair
): Promise<anchor.web3.PublicKey> => {
  try {
    const mint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      payer.publicKey,
      6
    );

    return mint;
  } catch (e) {
    console.log(`U got an error for creating the mint account: ${e}`);
  }
};

const ata_accounts = async (
  provider: anchor.AnchorProvider,
  payer: anchor.web3.Keypair,
  mint_acc: anchor.web3.PublicKey
): Promise<Account> => {
  const ata = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    payer,
    mint_acc,
    payer.publicKey
  );

  return ata;
};

const mint_tokens = async (
  provider: anchor.AnchorProvider,
  payer: anchor.web3.Keypair,
  mint_acc: anchor.web3.PublicKey,
  token_account: Account,
  amount: number
) => {
  console.log("Minting Tokens......");
  const tx = await mintToChecked(
    provider.connection,
    payer,
    mint_acc,
    token_account.address,
    payer.publicKey,
    amount * 1_000_000, //  for 6 decimal 10^6 is 1 token
    6
  );

  console.log(
    `Tokens are Minted to token account ${
      token_account.address
    } and tx hash is ${tx.toString()}`
  );
};

const create_pda = async (
  programId: anchor.web3.PublicKey,
  maker: anchor.web3.Keypair,
  secret_seed: anchor.BN,
  seed_const: String
): Promise<anchor.web3.PublicKey> => {
  const pda = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      maker.publicKey.toBuffer(),
      secret_seed.toArrayLike(Buffer, "le", 8),
    ],
    programId
  )[0];
  return pda;
};

const create_vault_account = async (
  provider: anchor.AnchorProvider,
  payer: anchor.web3.Keypair,
  mint_acc: anchor.web3.PublicKey,
  owner_ata: anchor.web3.PublicKey
): Promise<anchor.web3.PublicKey> => {
  try {
    const vault = getAssociatedTokenAddressSync(
      mint_acc,
      owner_ata,
      true,
      TOKEN_PROGRAM_ID
    );

    return vault;
  } catch (e) {
    console.error(`You got error while creating the vault account: ${e}`);
  }
};

describe("anchor-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorEscrow as Program<AnchorEscrow>;
  let secure_seed = new BN(100);
  let amount_req = new BN(2 * 1_000_000);
  let amount_deposited = new BN(10 * 1_000_000);

  // All Required Accounts
  let alice: anchor.web3.Keypair;
  let bob: anchor.web3.Keypair;
  let mint_a: anchor.web3.PublicKey;
  let mint_b: anchor.web3.PublicKey;
  let token_account_a_alice: Account;
  let token_account_b_alice: Account;
  let token_account_b_bob: Account;
  let token_account_a_bob: Account;

  let escrow_state: anchor.web3.PublicKey;
  let vault_account: anchor.web3.PublicKey;

  before("Config for Testing", async () => {
    try {
      // Keypair generating
      alice = anchor.web3.Keypair.generate();
      bob = anchor.web3.Keypair.generate();

      console.log(
        `The alice Pubkey is ${alice.publicKey.toString()} and bob Pubkey is ${bob.publicKey.toString()}`
      );

      // airdrop sol for alice and bob
      await airdropSOL(alice.publicKey, provider, 10); // for alice
      await airdropSOL(bob.publicKey, provider, 10); // for bob

      // create mint account
      mint_a = await mint_account(provider, alice); // for alice
      mint_b = await mint_account(provider, bob); // for bob

      // create token accounts
      token_account_a_alice = await ata_accounts(provider, alice, mint_a);
      token_account_b_alice = await ata_accounts(provider, alice, mint_b);
      token_account_b_bob = await ata_accounts(provider, bob, mint_b);
      token_account_a_bob = await ata_accounts(provider, bob, mint_a);

      // mint tokens
      await mint_tokens(provider, alice, mint_a, token_account_a_alice, 100);
      await mint_tokens(provider, bob, mint_b, token_account_b_bob, 100);

      // create escorw accout
      escrow_state = await create_pda(
        program.programId,
        alice,
        secure_seed,
        "escrow"
      );

      // vault token account
      vault_account = await create_vault_account(
        provider,
        alice,
        mint_a,
        escrow_state
      );
    } catch (e) {
      console.log(`Error before starting the test itself: ${e}`);
    }
  });

  it("Is initialized!", async () => {
    try {
      const alic_account_info_before = await getAccount(
        provider.connection,
        token_account_a_alice.address
      );

      console.log(
        `The amount holded by alice account beefore the TRX is: ${
          Number(alic_account_info_before.amount) / 1_000_000
        }`
      );
      const tx = await program.methods
        .initialize(secure_seed, amount_req, amount_deposited)
        .accountsStrict({
          maker: alice.publicKey,
          mintA: mint_a,
          mintB: mint_b,
          makerAtaA: token_account_a_alice.address,
          escrowState: escrow_state,
          vault: vault_account,

          // program accounts
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([alice])
        .rpc();

      console.log(
        `Successfully completed by depositing the token to vault account 🥳: ${tx.toString()}`
      );

      const vault_account_info = await getAccount(
        provider.connection,
        vault_account
      );

      const alic_account_info = await getAccount(
        provider.connection,
        token_account_a_alice.address
      );

      console.log(
        `The amount holded by vault account is: ${
          Number(vault_account_info.amount) / 1_000_000
        }`
      );

      console.log(
        `The amount holded by alice account after TRX is: ${
          Number(alic_account_info.amount) / 1_000_000
        }`
      );
    } catch (e) {
      throw new Error(`You got an error while testing Maker Instruction: ${e}`);
    }
  });

  it("Exchange the amount/value", async () => {
    try {
      await program.methods
        .exchange()
        .accountsStrict({
          maker: alice.publicKey,
          taker: bob.publicKey,

          mintA: mint_a,
          mintB: mint_b,

          makerAtaB: token_account_b_alice.address,
          takerAtaA: token_account_a_bob.address,
          takerAtaB: token_account_b_bob.address,

          escrowState: escrow_state,
          vaultAccount: vault_account,

          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([bob])
        .rpc();

      let bob_ata_a = await getAccount(
        provider.connection,
        token_account_a_bob.address
      );
      console.log(
        `The amount inside of this token account is:- ${
          Number(bob_ata_a.amount) / 1_000_000
        }`
      );
    } catch (e) {
      console.log(
        `You got an error while trying to exchnage the tokens:- ${e}`
      );
      throw new Error(e);
    }
  });

  it("Withdraw All Amount", async () => {
    try {
      await program.methods
        .refund()
        .accountsStrict({
          maker: alice.publicKey,
          mintA: mint_a,
          makerAtaA: token_account_a_alice.address,
          escrowState: escrow_state,
          vaultAccount: vault_account,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([alice])
        .rpc();

      const pda = await program.account.escrowState.fetchNullable(escrow_state);

      assert.equal(pda, null);
    } catch (e) {
      throw new Error(
        `You got an error while trying to withdraw all amounts ${e}`
      );
    }
  });
});
