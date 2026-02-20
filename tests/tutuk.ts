import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createTaskQueue,
  taskKey,
  taskQueueAuthorityKey,
  tuktukConfigKey,
  taskQueueKey,
} from "@helium/tuktuk-sdk";
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";
import { assert } from "chai";

describe("whitelist-transfer-hook tuktuk tests", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .whitelistTransferHook as Program<WhitelistTransferHook>;

  const tuktukProgramId = new anchor.web3.PublicKey(
    "tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA",
  );

  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("queue_authority")],
    program.programId,
  )[0];

  const whitelist = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("whitelist")],
    program.programId,
  )[0];
  const whitelistEntry = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("whitelist"), provider.publicKey.toBytes()],
    program.programId,
  )[0];

  let localTaskQueue: anchor.web3.PublicKey;

  it("Initialize local Tuktuk instance", async () => {
    // 1. We must fetch the IDL of the tuktuk program cloned to our localnet manually
    // Fetch from devnet since the local validator doesn't have the IDL account
    const devnetProvider = new anchor.AnchorProvider(
      new anchor.web3.Connection("https://api.devnet.solana.com"),
      (provider as anchor.AnchorProvider).wallet,
      anchor.AnchorProvider.defaultOptions(),
    );
    const tuktukIdl = await anchor.Program.fetchIdl(
      tuktukProgramId,
      devnetProvider,
    );
    if (!tuktukIdl) throw new Error("Could not fetch Tuktuk IDL from devnet");
    const tuktukProgram = new anchor.Program(tuktukIdl, provider);

    // 2. Initialize Tuktuk Config
    const tuktukConfig = tuktukConfigKey(tuktukProgramId)[0];

    // Check if config exists, if not initialize it
    const configAcc = await provider.connection.getAccountInfo(tuktukConfig);
    if (!configAcc) {
      try {
        await tuktukProgram.methods
          .initializeTuktukConfigV0({
            minDeposit: new anchor.BN(0),
          })
          .accounts({
            payer: provider.publicKey,
            approver: provider.publicKey,
            authority: provider.publicKey,
            tuktukConfig: tuktukConfig,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        // Wait a bit for the transaction to propagate
        await new Promise((resolve) => setTimeout(resolve, 1000));
      } catch (err) {
        console.error("Initialize config failed!", err);
        throw err;
      }
    }
  });

  it("Create Local Task Queue", async () => {
    const devnetProvider = new anchor.AnchorProvider(
      new anchor.web3.Connection("https://api.devnet.solana.com"),
      (provider as anchor.AnchorProvider).wallet,
      anchor.AnchorProvider.defaultOptions(),
    );
    const tuktukIdl = await anchor.Program.fetchIdl(
      tuktukProgramId,
      devnetProvider,
    );
    const tuktukProgram = new anchor.Program(tuktukIdl, provider);
    const tuktukConfig = tuktukConfigKey(tuktukProgramId)[0];

    // 3. Create a Task Queue
    const createQueueIx = await createTaskQueue(tuktukProgram as any, {
      name: "local-queue",
      capacity: 10,
      minCrankReward: new anchor.BN(100),
      lookupTables: [],
      staleTaskAge: 0,
    });

    // Since we created it, the localTaskQueue PDA uses ID 0
    localTaskQueue = taskQueueKey(tuktukConfig, 0, tuktukProgramId)[0];

    try {
      await createQueueIx.rpc();
      // Wait a bit for the transaction to propagate
      await new Promise((resolve) => setTimeout(resolve, 1000));
    } catch (err) {
      console.error("Create queue failed:", err);
      throw err;
    }
  });

  it("Authorize Queue Authority", async () => {
    const devnetProvider = new anchor.AnchorProvider(
      new anchor.web3.Connection("https://api.devnet.solana.com"),
      (provider as anchor.AnchorProvider).wallet,
      anchor.AnchorProvider.defaultOptions(),
    );
    const tuktukIdl = await anchor.Program.fetchIdl(
      tuktukProgramId,
      devnetProvider,
    );
    const tuktukProgram = new anchor.Program(tuktukIdl, provider);
    const tuktukConfig = tuktukConfigKey(tuktukProgramId)[0];
    localTaskQueue = taskQueueKey(tuktukConfig, 0, tuktukProgramId)[0];

    // 4. Authorize whitelistTransferHook to use the new queue
    const taskQueueAuthority = taskQueueAuthorityKey(
      localTaskQueue,
      queueAuthority,
      tuktukProgramId,
    )[0];

    try {
      await tuktukProgram.methods
        .addQueueAuthorityV0()
        .accounts({
          payer: provider.publicKey,
          updateAuthority: provider.publicKey,
          queueAuthority: queueAuthority,
          taskQueueAuthority: taskQueueAuthority,
          taskQueue: localTaskQueue,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    } catch (err) {
      console.error("Failed to add queue authority. Program Logs:");
      if (err.logs) {
        console.error(err.logs);
      } else {
        console.error(err);
      }
      throw err;
    }
  });

  it("Schedule add_to_whitelist task", async () => {
    let taskID = 0; // The first task on the queue will be 0

    // Recreate the authority PDA dynamically using the new local queue
    if (!localTaskQueue) {
      const tuktukConfig = tuktukConfigKey(tuktukProgramId)[0];
      localTaskQueue = taskQueueKey(tuktukConfig, 0, tuktukProgramId)[0];
    }

    const taskQueueAuthority = taskQueueAuthorityKey(
      localTaskQueue,
      queueAuthority,
      tuktukProgramId,
    )[0];

    try {
      const tx = await program.methods
        .schedule(taskID)
        .accountsPartial({
          user: provider.publicKey,
          whitelist: whitelist,
          whitelistEntry: whitelistEntry,
          taskQueue: localTaskQueue,
          taskQueueAuthority: taskQueueAuthority,
          task: taskKey(localTaskQueue, taskID, tuktukProgramId)[0],
          queueAuthority: queueAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          tuktukProgram: tuktukProgramId,
        })
        .rpc();
      assert(
        tuktukProgramId.equals(
          new anchor.web3.PublicKey(
            "tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA",
          ),
        ),
      );
      console.log("\nYour transaction signature", tx);
    } catch (e) {
      console.error(e);
      if (e.logs) {
        console.error("Program Logs:", e.logs);
      }
      throw e;
    }
  });
});
