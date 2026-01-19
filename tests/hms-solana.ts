import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HmsSolana } from "../target/types/hms_solana";
import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as crypto from "crypto";

describe("hms-solana", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.hmsSolana as Program<HmsSolana>;
  const provider = anchor.getProvider();

  // Test accounts
  let userKeypair: Keypair;
  let doctorKeypair: Keypair;
  let researcherKeypair: Keypair;
  let emergencyContactKeypair: Keypair;
  let userProfilePda: PublicKey;
  let recordPda: PublicKey;
  let accessGrantPda: PublicKey;

  // Test data
  const recordType = "vital_signs";
  const encryptedUri = "QmTestHashForIPFS123456789";
  const dataHash = crypto.randomBytes(32);
  const metadata = "Blood pressure reading - 120/80 mmHg";

  before(async () => {
    // Generate test keypairs
    userKeypair = Keypair.generate();
    doctorKeypair = Keypair.generate();
    researcherKeypair = Keypair.generate();
    emergencyContactKeypair = Keypair.generate();

    // Airdrop SOL to test accounts
    const airdropAmount = 2 * anchor.web3.LAMPORTS_PER_SOL;

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        userKeypair.publicKey,
        airdropAmount,
      ),
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        doctorKeypair.publicKey,
        airdropAmount,
      ),
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        researcherKeypair.publicKey,
        airdropAmount,
      ),
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        emergencyContactKeypair.publicKey,
        airdropAmount,
      ),
    );

    // Derive PDAs
    [userProfilePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_profile"), userKeypair.publicKey.toBuffer()],
      program.programId,
    );
  });

  describe("Profile Management", () => {
    it("Initializes a health profile", async () => {
      const tx = await program.methods
        .initializeProfile()
        .accountsPartial({
          user: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Profile initialization tx:", tx);

      // Verify profile was created
      const profile = await program.account.userHealthProfile.fetch(
        userProfilePda,
      );
      expect(profile.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(profile.recordCount.toNumber()).to.equal(0);
    });

    it("Fails to initialize profile twice", async () => {
      try {
        await program.methods
          .initializeProfile()
          .accountsPartial({
            user: userKeypair.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([userKeypair])
          .rpc();

        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("already in use");
      }
    });
  });

  describe("Health Records", () => {
    it("Adds a health record", async () => {
      // Derive record PDA
      [recordPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("health_record"),
          userKeypair.publicKey.toBuffer(),
          Buffer.from([0, 0, 0, 0, 0, 0, 0, 0]), // record_count = 0
        ],
        program.programId,
      );

      const tx = await program.methods
        .addRecord(recordType, encryptedUri, Array.from(dataHash), metadata)
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Add record tx:", tx);

      // Verify record was created
      const record = await program.account.healthRecord.fetch(recordPda);
      expect(record.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(record.recordType).to.equal(recordType);
      expect(record.encryptedUri).to.equal(encryptedUri);
      expect(record.metadata).to.equal(metadata);
      expect(record.isDeleted).to.be.false;

      // Verify profile was updated
      const profile = await program.account.userHealthProfile.fetch(
        userProfilePda,
      );
      expect(profile.recordCount.toNumber()).to.equal(1);
    });

    it("Updates a health record", async () => {
      const newMetadata = "Updated: Blood pressure reading - 118/78 mmHg";

      const tx = await program.methods
        .updateRecord(new anchor.BN(0), newMetadata)
        .accountsPartial({
          recordOwner: userKeypair.publicKey,
          actor: userKeypair.publicKey,
        })
        .remainingAccounts([])
        .signers([userKeypair])
        .rpc();

      console.log("Update record tx:", tx);

      // Verify record was updated
      const record = await program.account.healthRecord.fetch(recordPda);
      expect(record.metadata).to.equal(newMetadata);
    });

    it("Soft deletes a health record", async () => {
      const tx = await program.methods
        .deleteRecord(new anchor.BN(0))
        .accountsPartial({
          owner: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Delete record tx:", tx);

      // Verify record was soft deleted
      const record = await program.account.healthRecord.fetch(recordPda);
      expect(record.isDeleted).to.be.true;
    });
  });

  describe("Access Control", () => {
    let secondRecordPda: PublicKey;

    before(async () => {
      // Create another record for access control testing
      [secondRecordPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("health_record"),
          userKeypair.publicKey.toBuffer(),
          Buffer.from([1, 0, 0, 0, 0, 0, 0, 0]), // record_count = 1
        ],
        program.programId,
      );

      await program.methods
        .addRecord(
          "lab_result",
          "QmAnotherTestHash",
          Array.from(crypto.randomBytes(32)),
          "Lab test results",
        )
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();
    });

    it("Grants access to another user", async () => {
      // Derive access grant PDA
      [accessGrantPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("access_grant"),
          userKeypair.publicKey.toBuffer(),
          doctorKeypair.publicKey.toBuffer(),
        ],
        program.programId,
      );

      const expiresAt = new anchor.BN(Date.now() / 1000 + 86400); // 24 hours from now
      const permissions = 1 | 2; // READ | WRITE

      const tx = await program.methods
        .grantAccess(doctorKeypair.publicKey, expiresAt, permissions)
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Grant access tx:", tx);

      // Verify access grant was created
      const accessGrant = await program.account.accessGrant.fetch(
        accessGrantPda,
      );
      expect(accessGrant.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(accessGrant.grantee.toString()).to.equal(
        doctorKeypair.publicKey.toString(),
      );
      expect(accessGrant.permissions).to.equal(permissions);
    });

    it("Allows authorized user to read record", async () => {
      const tx = await program.methods
        .readRecord(new anchor.BN(1))
        .accountsPartial({
          recordOwner: userKeypair.publicKey,
          accessor: doctorKeypair.publicKey,
        })
        .signers([doctorKeypair])
        .rpc();

      console.log("Read record tx:", tx);
    });

    it("Allows authorized user to update record", async () => {
      const doctorNote = "Doctor's note: Results look normal";

      const tx = await program.methods
        .updateRecord(new anchor.BN(1), doctorNote)
        .accountsPartial({
          recordOwner: userKeypair.publicKey,
          actor: doctorKeypair.publicKey,
        })
        .signers([doctorKeypair])
        .rpc();

      console.log("Doctor update record tx:", tx);

      // Verify record was updated
      const record = await program.account.healthRecord.fetch(secondRecordPda);
      expect(record.metadata).to.equal(doctorNote);
    });

    it("Revokes access from user", async () => {
      const tx = await program.methods
        .revokeAccess(doctorKeypair.publicKey)
        .accountsPartial({
          owner: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Revoke access tx:", tx);

      // Verify access grant was closed
      try {
        await program.account.accessGrant.fetch(accessGrantPda);
        expect.fail("Access grant should have been closed");
      } catch (error) {
        expect(error.message).to.include("Account does not exist");
      }
    });
  });

  describe("ZK Proof Access", () => {
    let zkProofPda: PublicKey;
    const proofHash = crypto.randomBytes(32);
    const publicInputs = crypto.randomBytes(32);
    const proofData = crypto.randomBytes(256);

    it("Generates a ZK proof", async () => {
      [zkProofPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("zk_proof"), userKeypair.publicKey.toBuffer(), proofHash],
        program.programId,
      );

      const tx = await program.methods
        .generateZkProof(
          Array.from(proofHash),
          Array.from(publicInputs),
          Array.from(proofData),
        )
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Generate ZK proof tx:", tx);

      // Verify ZK proof was created
      const zkProof = await program.account.zkProofState.fetch(zkProofPda);
      expect(zkProof.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(zkProof.isValid).to.be.true;
    });

    it("Verifies a ZK proof", async () => {
      const tx = await program.methods
        .verifyZkProof(Array.from(proofHash))
        .accountsPartial({
          proofOwner: userKeypair.publicKey,
          verifier: doctorKeypair.publicKey,
        })
        .signers([doctorKeypair])
        .rpc();

      console.log("Verify ZK proof tx:", tx);

      // Verify proof verification count increased
      const zkProof = await program.account.zkProofState.fetch(zkProofPda);
      expect(zkProof.verificationCount.toNumber()).to.equal(1);
    });

    it("Accesses record with ZK proof", async () => {
      const tx = await program.methods
        .accessWithZkProof(new anchor.BN(1), Array.from(proofHash))
        .accountsPartial({
          recordOwner: userKeypair.publicKey,
          accessor: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Access with ZK proof tx:", tx);
    });
  });

  describe("Emergency Access", () => {
    let emergencyAccessPda: PublicKey;

    it("Configures emergency access", async () => {
      [emergencyAccessPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("emergency_access"),
          userKeypair.publicKey.toBuffer(),
          emergencyContactKeypair.publicKey.toBuffer(),
        ],
        program.programId,
      );

      const tx = await program.methods
        .configureEmergencyAccess(emergencyContactKeypair.publicKey)
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Configure emergency access tx:", tx);

      // Verify emergency access was configured
      const emergencyAccess = await program.account.emergencyAccess.fetch(
        emergencyAccessPda,
      );
      expect(emergencyAccess.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(emergencyAccess.emergencyContact.toString()).to.equal(
        emergencyContactKeypair.publicKey.toString(),
      );
      expect(emergencyAccess.isActive).to.be.false;
    });

    it("Activates emergency access", async () => {
      const reason = "Medical emergency - patient unconscious";

      const tx = await program.methods
        .activateEmergencyAccess(emergencyContactKeypair.publicKey, reason)
        .accountsPartial({
          owner: userKeypair.publicKey,
          activator: emergencyContactKeypair.publicKey,
        })
        .signers([emergencyContactKeypair])
        .rpc();

      console.log("Activate emergency access tx:", tx);

      // Verify emergency access was activated
      const emergencyAccess = await program.account.emergencyAccess.fetch(
        emergencyAccessPda,
      );
      expect(emergencyAccess.isActive).to.be.true;
      expect(emergencyAccess.activationReason).to.equal(reason);
    });

    it("Accesses record with emergency authorization", async () => {
      const tx = await program.methods
        .accessWithEmergency(
          new anchor.BN(1),
          emergencyContactKeypair.publicKey,
        )
        .accountsPartial({
          recordOwner: userKeypair.publicKey,
          accessor: emergencyContactKeypair.publicKey,
        })
        .signers([emergencyContactKeypair])
        .rpc();

      console.log("Access with emergency tx:", tx);
    });

    it("Deactivates emergency access", async () => {
      const tx = await program.methods
        .deactivateEmergencyAccess(emergencyContactKeypair.publicKey)
        .accountsPartial({
          owner: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Deactivate emergency access tx:", tx);

      // Verify emergency access was deactivated
      const emergencyAccess = await program.account.emergencyAccess.fetch(
        emergencyAccessPda,
      );
      expect(emergencyAccess.isActive).to.be.false;
    });
  });

  describe("DAO Governance", () => {
    let proposalPda: PublicKey;
    let votePda: PublicKey;
    const proposalId = 1;
    const researchTopic = "COVID-19 Long-term Effects Study";

    it("Creates a research proposal", async () => {
      [proposalPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("dao_governance"),
          Buffer.from("proposal"),
          new anchor.BN(proposalId).toArrayLike(Buffer, "le", 8),
        ],
        program.programId,
      );

      const tx = await program.methods
        .createResearchProposal(new anchor.BN(proposalId), researchTopic)
        .accountsPartial({
          researcher: researcherKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([researcherKeypair])
        .rpc();

      console.log("Create research proposal tx:", tx);

      // Verify proposal was created
      const proposal = await program.account.researchProposal.fetch(
        proposalPda,
      );
      expect(proposal.researcher.toString()).to.equal(
        researcherKeypair.publicKey.toString(),
      );
      expect(proposal.researchTopic).to.equal(researchTopic);
      expect(proposal.yesVotes.toNumber()).to.equal(0);
    });

    it("Votes on research proposal", async () => {
      [votePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("dao_governance"),
          Buffer.from("vote"),
          new anchor.BN(proposalId).toArrayLike(Buffer, "le", 8),
          userKeypair.publicKey.toBuffer(),
        ],
        program.programId,
      );

      const tx = await program.methods
        .voteOnResearchProposal(new anchor.BN(proposalId), true)
        .accountsPartial({
          voter: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Vote on proposal tx:", tx);

      // Verify vote was recorded
      const vote = await program.account.researchVote.fetch(votePda);
      expect(vote.voter.toString()).to.equal(userKeypair.publicKey.toString());
      expect(vote.vote).to.be.true;

      // Verify proposal vote count increased
      const proposal = await program.account.researchProposal.fetch(
        proposalPda,
      );
      expect(proposal.yesVotes.toNumber()).to.equal(1);
    });
  });

  describe("Wearable Integration", () => {
    let wearableDevicePda: PublicKey;
    let dataBatchPda: PublicKey;
    const deviceId = "smartwatch-001";
    const deviceType = "fitness_tracker";
    const deviceKeypair = Keypair.generate();
    const batchId = 1;

    it("Registers a wearable device", async () => {
      [wearableDevicePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("wearable_device"),
          userKeypair.publicKey.toBuffer(),
          Buffer.from(deviceId),
        ],
        program.programId,
      );

      const tx = await program.methods
        .registerWearableDevice(deviceId, deviceType, deviceKeypair.publicKey)
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Register wearable device tx:", tx);

      // Verify device was registered
      const device = await program.account.wearableDevice.fetch(
        wearableDevicePda,
      );
      expect(device.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(device.deviceId).to.equal(deviceId);
      expect(device.deviceType).to.equal(deviceType);
      expect(device.isActive).to.be.true;
    });

    it("Ingests wearable data", async () => {
      [dataBatchPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("wearable_device"),
          Buffer.from("batch"),
          userKeypair.publicKey.toBuffer(),
          Buffer.from(deviceId),
          new anchor.BN(batchId).toArrayLike(Buffer, "le", 8),
        ],
        program.programId,
      );

      const encryptedDataUri = "QmWearableDataBatch123";
      const batchDataHash = crypto.randomBytes(32);
      const dataPointCount = 100;
      const startTimestamp = Math.floor(Date.now() / 1000) - 3600; // 1 hour ago
      const endTimestamp = Math.floor(Date.now() / 1000);

      const tx = await program.methods
        .ingestWearableData(
          deviceId,
          new anchor.BN(batchId),
          encryptedDataUri,
          Array.from(batchDataHash),
          dataPointCount,
          new anchor.BN(startTimestamp),
          new anchor.BN(endTimestamp),
        )
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Ingest wearable data tx:", tx);

      // Verify data batch was created
      const dataBatch = await program.account.wearableDataBatch.fetch(
        dataBatchPda,
      );
      expect(dataBatch.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(dataBatch.deviceId).to.equal(deviceId);
      expect(dataBatch.dataPointCount).to.equal(dataPointCount);
      expect(dataBatch.isProcessed).to.be.false;
    });

    it("Processes wearable data to health record", async () => {
      const newRecordType = "heart_rate_data";
      const newMetadata = "Heart rate data from smartwatch";

      // Get current record count for PDA derivation
      const profile = await program.account.userHealthProfile.fetch(
        userProfilePda,
      );
      const recordCount = profile.recordCount.toNumber();

      const [newRecordPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("health_record"),
          userKeypair.publicKey.toBuffer(),
          new anchor.BN(recordCount).toArrayLike(Buffer, "le", 8),
        ],
        program.programId,
      );

      const tx = await program.methods
        .processWearableDataToRecord(
          deviceId,
          new anchor.BN(batchId),
          newRecordType,
          newMetadata,
        )
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Process wearable data to record tx:", tx);

      // Verify health record was created
      const healthRecord = await program.account.healthRecord.fetch(
        newRecordPda,
      );
      expect(healthRecord.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(healthRecord.recordType).to.equal(newRecordType);
      expect(healthRecord.metadata).to.equal(newMetadata);

      // Verify data batch was marked as processed
      const dataBatch = await program.account.wearableDataBatch.fetch(
        dataBatchPda,
      );
      expect(dataBatch.isProcessed).to.be.true;
    });

    it("Deactivates wearable device", async () => {
      const tx = await program.methods
        .deactivateWearableDevice(deviceId)
        .accountsPartial({
          owner: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Deactivate wearable device tx:", tx);

      // Verify device was deactivated
      const device = await program.account.wearableDevice.fetch(
        wearableDevicePda,
      );
      expect(device.isActive).to.be.false;
    });
  });

  describe("Cross-Device Sync", () => {
    let syncStatePda1: PublicKey;
    let syncStatePda2: PublicKey;
    let syncOperationPda: PublicKey;
    const device1Id = "mobile-app-001";
    const device2Id = "web-app-001";
    const syncKey = crypto.randomBytes(32);

    it("Initializes sync state for device 1", async () => {
      [syncStatePda1] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("sync_state"),
          userKeypair.publicKey.toBuffer(),
          Buffer.from(device1Id),
        ],
        program.programId,
      );

      const tx = await program.methods
        .initializeSyncState(device1Id, Array.from(syncKey), true)
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Initialize sync state 1 tx:", tx);

      // Verify sync state was created
      const syncState = await program.account.syncState.fetch(syncStatePda1);
      expect(syncState.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(syncState.deviceId).to.equal(device1Id);
      expect(syncState.isPrimary).to.be.true;
    });

    it("Initializes sync state for device 2", async () => {
      [syncStatePda2] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("sync_state"),
          userKeypair.publicKey.toBuffer(),
          Buffer.from(device2Id),
        ],
        program.programId,
      );

      const tx = await program.methods
        .initializeSyncState(device2Id, Array.from(syncKey), false)
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Initialize sync state 2 tx:", tx);

      // Verify sync state was created
      const syncState = await program.account.syncState.fetch(syncStatePda2);
      expect(syncState.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(syncState.deviceId).to.equal(device2Id);
      expect(syncState.isPrimary).to.be.false;
    });

    it("Starts sync operation", async () => {
      [syncOperationPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("sync_state"),
          Buffer.from("operation"),
          userKeypair.publicKey.toBuffer(),
          Buffer.from(device1Id),
          Buffer.from(device2Id),
        ],
        program.programId,
      );

      const operationType = "full_sync";

      const tx = await program.methods
        .startSyncOperation(
          device1Id,
          device2Id,
          operationType,
          Array.from(syncKey),
        )
        .accountsPartial({
          owner: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Start sync operation tx:", tx);

      // Verify sync operation was created
      const syncOperation = await program.account.syncOperation.fetch(
        syncOperationPda,
      );
      expect(syncOperation.owner.toString()).to.equal(
        userKeypair.publicKey.toString(),
      );
      expect(syncOperation.sourceDevice).to.equal(device1Id);
      expect(syncOperation.targetDevice).to.equal(device2Id);
      expect(syncOperation.operationType).to.equal(operationType);
    });

    it("Completes sync operation", async () => {
      const recordsSynced = 5;
      const newStateHash = crypto.randomBytes(32);

      const tx = await program.methods
        .completeSyncOperation(
          device1Id,
          device2Id,
          new anchor.BN(recordsSynced),
          Array.from(newStateHash),
        )
        .accountsPartial({
          owner: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Complete sync operation tx:", tx);

      // Verify sync operation was completed
      const syncOperation = await program.account.syncOperation.fetch(
        syncOperationPda,
      );
      expect(syncOperation.isSuccessful).to.be.true;
      expect(syncOperation.recordsSynced.toNumber()).to.equal(recordsSynced);

      // Verify sync states were updated
      const syncState1 = await program.account.syncState.fetch(syncStatePda1);
      const syncState2 = await program.account.syncState.fetch(syncStatePda2);
      expect(syncState1.syncCount.toNumber()).to.equal(1);
      expect(syncState2.syncCount.toNumber()).to.equal(1);
    });

    it("Updates sync primary status", async () => {
      const tx = await program.methods
        .updateSyncPrimary(device2Id, true)
        .accountsPartial({
          owner: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("Update sync primary tx:", tx);

      // Verify primary status was updated
      const syncState = await program.account.syncState.fetch(syncStatePda2);
      expect(syncState.isPrimary).to.be.true;
    });
  });

  describe("HMS NFT", () => {
    it("Creates an HMS NFT", async () => {
      const nftName = "HMS Profile NFT";
      const nftSymbol = "HMSP";
      const nftUri = "https://example.com/hms-nft.json";

      const mint = Keypair.generate();
      const tokenAccount = await anchor.utils.token.associatedAddress({
        mint: mint.publicKey,
        owner: userKeypair.publicKey
      });

      const metadataProgramId = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
      const [metadataAccount, _] = PublicKey.findProgramAddressSync(
          [
              Buffer.from("metadata"),
              metadataProgramId.toBuffer(),
              mint.publicKey.toBuffer(),
          ],
          metadataProgramId
      );
      const [masterEditionAccount, __] = PublicKey.findProgramAddressSync(
          [
              Buffer.from("metadata"),
              metadataProgramId.toBuffer(),
              mint.publicKey.toBuffer(),
              Buffer.from("edition"),
          ],
          metadataProgramId
      );

      const tx = await program.methods
        .createHmsNft(nftName, nftSymbol, nftUri)
        .accounts({
          authority: userKeypair.publicKey,
          mint: mint.publicKey,
          tokenAccount,
          metadataAccount,
          masterEditionAccount,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          tokenMetadataProgram: metadataProgramId,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([userKeypair, mint])
        .rpc();

      console.log("Create HMS NFT tx:", tx);

      // You can add assertions here to verify the NFT was created correctly
      // For example, fetch the mint and check its supply, or fetch the metadata account
    });
  });
});
