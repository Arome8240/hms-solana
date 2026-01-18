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

    // Airdrop SOL to test accounts
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        userKeypair.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL,
      ),
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        doctorKeypair.publicKey,
        1 * anchor.web3.LAMPORTS_PER_SOL,
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
        .accounts({
          profile: userProfilePda,
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
          .accounts({
            profile: userProfilePda,
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
        .accounts({
          profile: userProfilePda,
          record: recordPda,
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
        .accounts({
          profile: userProfilePda,
          record: recordPda,
          recordOwner: userKeypair.publicKey,
          actor: userKeypair.publicKey,
          accessGrant: null,
        })
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
        .accounts({
          profile: userProfilePda,
          record: recordPda,
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
        .accounts({
          profile: userProfilePda,
          record: secondRecordPda,
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
        .accounts({
          profile: userProfilePda,
          accessGrant: accessGrantPda,
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
        .accounts({
          profile: userProfilePda,
          record: secondRecordPda,
          recordOwner: userKeypair.publicKey,
          accessor: doctorKeypair.publicKey,
          accessGrant: accessGrantPda,
        })
        .signers([doctorKeypair])
        .rpc();

      console.log("Read record tx:", tx);
    });

    it("Allows authorized user to update record", async () => {
      const doctorNote = "Doctor's note: Results look normal";

      const tx = await program.methods
        .updateRecord(new anchor.BN(1), doctorNote)
        .accounts({
          profile: userProfilePda,
          record: secondRecordPda,
          recordOwner: userKeypair.publicKey,
          actor: doctorKeypair.publicKey,
          accessGrant: accessGrantPda,
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
        .accounts({
          profile: userProfilePda,
          accessGrant: accessGrantPda,
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

    it("Prevents unauthorized access after revocation", async () => {
      try {
        await program.methods
          .readRecord(new anchor.BN(1))
          .accounts({
            profile: userProfilePda,
            record: secondRecordPda,
            recordOwner: userKeypair.publicKey,
            accessor: doctorKeypair.publicKey,
            accessGrant: null,
          })
          .signers([doctorKeypair])
          .rpc();

        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("UnauthorizedAccess");
      }
    });
  });
});
