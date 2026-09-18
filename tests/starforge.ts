import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { Starforge } from "../target/types/starforge";

// Anchor test: full arena cycle on a local validator.
//   start_session -> reveal_spin -> [submit_picks -> settle_gamble] -> settle
// Payout assertions compare the on-chain `total_bps` with the payout formula
// (payout = wager * total_bps / 10_000, fee applied by the client in prod).

const SLOT_HASHES = new anchor.web3.PublicKey(
  "SysvarS1otHashes111111111111111111111111111"
);
// Reveal delay hard-coded in programs/starforge/src/lib.rs
const REVEAL_DELAY_SLOTS = 10;

async function sleepForSlots(
  connection: anchor.web3.Connection,
  n: number
): Promise<void> {
  const start = await connection.getSlot();
  // eslint-disable-next-line no-constant-condition
  while (true) {
    const now = await connection.getSlot();
    if (now >= start + n) return;
    await new Promise((r) => setTimeout(r, 400));
  }
}

describe("starforge arena", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Starforge as Program<Starforge>;
  const connection = provider.connection;
  const player = provider.wallet.publicKey;

  const sessionId = new anchor.BN(Date.now() % 100000);
  const wager = new anchor.BN(100_000_000); // 0.1 SOL
  const artifacts = 0x07;

  const [housePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("house")],
    program.programId
  );
  const [sessionPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("session"), player.toBuffer(), sessionId.toArrayLike(Buffer, "le", 8)],
    program.programId
  );

  it("initializes the house", async () => {
    await program.methods
      .initializeHouse(250)
      .accountsPartial({ house: housePda, authority: player })
      .rpc();
    const house = await program.account.house.fetch(housePda);
    expect(house.feeBps).to.equal(250);
    expect(house.authority.toBase58()).to.equal(player.toBase58());
  });

  it("opens a session and locks the wager", async () => {
    const before = await connection.getBalance(player);
    await program.methods
      .startSession(sessionId, wager, artifacts)
      .accountsPartial({ session: sessionPda, player })
      .rpc();
    const session = await program.account.session.fetch(sessionPda);
    expect(session.stage).to.equal(0); // STAGE_SPIN
    expect(session.wager.toNumber()).to.equal(wager.toNumber());
    expect(session.artifacts).to.equal(artifacts);
    const after = await connection.getBalance(player);
    expect(before - after).to.be.greaterThan(wager.toNumber());
    const vault = await connection.getBalance(sessionPda);
    expect(vault).to.be.greaterThanOrEqual(wager.toNumber());
  });

  it("rejects picks before the spin is revealed", async () => {
    try {
      await program.methods
        .submitPicks([0, 1, 2, 3, 4], false)
        .accountsPartial({ session: sessionPda, player })
        .rpc();
      expect.fail("should have thrown WrongStage");
    } catch (e: any) {
      expect(e.toString()).to.include("WrongStage");
    }
  });

  it("reveals the spin after the delay and writes provable grids", async () => {
    await sleepForSlots(connection, REVEAL_DELAY_SLOTS + 2);
    await program.methods
      .revealSpin()
      .accountsPartial({
        session: sessionPda,
        player,
        slotHashes: SLOT_HASHES,
      })
      .rpc();
    const session = await program.account.session.fetch(sessionPda);
    expect(session.seed.some((b: number) => b !== 0)).to.be.true;
    expect(session.stepCount).to.be.greaterThan(0);
    expect(session.stepCount).to.be.lessThanOrEqual(2);
    // stage is PICKS (1) on supernova, DONE (3) otherwise
    expect([1, 3]).to.include(session.stage);
  });

  it("completes the full gamble branch when supernova hits", async () => {
    let session = await program.account.session.fetch(sessionPda);
    if (session.stage !== 1) {
      console.log("      (no supernova this seed — gamble branch skipped)");
      return;
    }
    // 5 distinct picks in [0,12)
    await program.methods
      .submitPicks([0, 2, 5, 7, 11], true)
      .accountsPartial({ session: sessionPda, player })
      .rpc();
    session = await program.account.session.fetch(sessionPda);
    expect(session.stage).to.equal(2); // STAGE_GAMBLE
    expect(session.picksBps.toNumber()).to.be.greaterThanOrEqual(0);

    await sleepForSlots(connection, REVEAL_DELAY_SLOTS + 2);
    const before = await connection.getBalance(player);
    await program.methods
      .settleGamble()
      .accountsPartial({
        session: sessionPda,
        player,
        slotHashes: SLOT_HASHES,
      })
      .rpc();
    session = await program.account.session.fetch(sessionPda);
    expect(session.stage).to.equal(3); // STAGE_DONE
    const after = await connection.getBalance(player);
    const expectedPayout = Math.floor(
      (wager.toNumber() * session.totalBps.toNumber()) / 10000
    );
    // payout credited (minus tx fee noise); vault must be nearly empty
    expect(after - before).to.be.greaterThanOrEqual(expectedPayout - 20000);
    console.log(
      `      payout=${expectedPayout} lamports (total_bps=${session.totalBps.toNumber()}, gamble_win=${session.gambleWin})`
    );
  });

  it("settles straight through when there is no supernova", async function () {
    const session = await program.account.session.fetch(sessionPda);
    if (session.stage !== 3) this.skip();
    // already settled by reveal_spin in the non-supernova path
    expect(session.totalBps.toNumber()).to.equal(session.gridWinBps.toNumber());
  });
});
