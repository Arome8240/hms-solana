/**
 * Health Management System - Client SDK Example
 *
 * This file demonstrates how to interact with the HMS Solana program
 * from a mobile app or web client.
 */

import {
  Connection,
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { Program, AnchorProvider, Wallet, BN } from "@coral-xyz/anchor";
import { HmsSolana } from "./target/types/hms_solana";
import * as crypto from "crypto";

// Program ID (replace with your deployed program ID)
const PROGRAM_ID = new PublicKey("n3qd2EbGLXbVCJhB1H8pQGDTzHfvDJEPqp4PuhZBiz2");

// Permission constants
const PERMISSION_READ = 1;
const PERMISSION_WRITE = 2;
const PERMISSION_SHARE = 4;

export class HealthManagementClient {
  private connection: Connection;
  private program: Program<HmsSolana>;
  private wallet: Wallet;

  constructor(
    connection: Connection,
    wallet: Wallet,
    program: Program<HmsSolana>,
  ) {
    this.connection = connection;
    this.wallet = wallet;
    this.program = program;
  }

  /**
   * Initialize a health profile for the current user
   */
  async initializeProfile(): Promise<string> {
    const [profilePda] = this.getUserProfilePda(this.wallet.publicKey);

    const tx = await this.program.methods
      .initializeProfile()
      .accounts({
        profile: profilePda,
        user: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Profile initialized:", tx);
    return tx;
  }

  /**
   * Add a new health record
   */
  async addHealthRecord(
    recordType: string,
    encryptedDataUri: string,
    originalData: Buffer,
    metadata: string,
  ): Promise<{ signature: string; recordId: number }> {
    const profile = await this.getUserProfile();
    const recordId = profile.recordCount.toNumber();

    const [profilePda] = this.getUserProfilePda(this.wallet.publicKey);
    const [recordPda] = this.getHealthRecordPda(
      this.wallet.publicKey,
      recordId,
    );

    // Generate data hash for integrity verification
    const dataHash = crypto.createHash("sha256").update(originalData).digest();

    const tx = await this.program.methods
      .addRecord(recordType, encryptedDataUri, Array.from(dataHash), metadata)
      .accounts({
        profile: profilePda,
        record: recordPda,
        owner: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Health record added:", tx);
    return { signature: tx, recordId };
  }

  /**
   * Update health record metadata
   */
  async updateHealthRecord(
    recordOwner: PublicKey,
    recordId: number,
    newMetadata: string,
  ): Promise<string> {
    const [profilePda] = this.getUserProfilePda(recordOwner);
    const [recordPda] = this.getHealthRecordPda(recordOwner, recordId);

    // Check if we need access grant
    let accessGrantPda = null;
    if (!recordOwner.equals(this.wallet.publicKey)) {
      [accessGrantPda] = this.getAccessGrantPda(
        recordOwner,
        this.wallet.publicKey,
      );
    }

    const tx = await this.program.methods
      .updateRecord(new BN(recordId), newMetadata)
      .accounts({
        profile: profilePda,
        record: recordPda,
        recordOwner: recordOwner,
        actor: this.wallet.publicKey,
        accessGrant: accessGrantPda,
      })
      .rpc();

    console.log("Health record updated:", tx);
    return tx;
  }

  /**
   * Soft delete a health record (owner only)
   */
  async deleteHealthRecord(recordId: number): Promise<string> {
    const [profilePda] = this.getUserProfilePda(this.wallet.publicKey);
    const [recordPda] = this.getHealthRecordPda(
      this.wallet.publicKey,
      recordId,
    );

    const tx = await this.program.methods
      .deleteRecord(new BN(recordId))
      .accounts({
        profile: profilePda,
        record: recordPda,
        owner: this.wallet.publicKey,
      })
      .rpc();

    console.log("Health record deleted:", tx);
    return tx;
  }

  /**
   * Grant access to another user
   */
  async grantAccess(
    grantee: PublicKey,
    durationHours: number,
    permissions: number[],
  ): Promise<string> {
    const [profilePda] = this.getUserProfilePda(this.wallet.publicKey);
    const [accessGrantPda] = this.getAccessGrantPda(
      this.wallet.publicKey,
      grantee,
    );

    const expiresAt = new BN(
      Math.floor(Date.now() / 1000) + durationHours * 3600,
    );
    const permissionBitmask = permissions.reduce((acc, perm) => acc | perm, 0);

    const tx = await this.program.methods
      .grantAccess(grantee, expiresAt, permissionBitmask)
      .accounts({
        profile: profilePda,
        accessGrant: accessGrantPda,
        owner: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Access granted:", tx);
    return tx;
  }

  /**
   * Revoke access from a user
   */
  async revokeAccess(grantee: PublicKey): Promise<string> {
    const [profilePda] = this.getUserProfilePda(this.wallet.publicKey);
    const [accessGrantPda] = this.getAccessGrantPda(
      this.wallet.publicKey,
      grantee,
    );

    const tx = await this.program.methods
      .revokeAccess(grantee)
      .accounts({
        profile: profilePda,
        accessGrant: accessGrantPda,
        owner: this.wallet.publicKey,
      })
      .rpc();

    console.log("Access revoked:", tx);
    return tx;
  }

  /**
   * Read a health record (with access control)
   */
  async readHealthRecord(
    recordOwner: PublicKey,
    recordId: number,
  ): Promise<{ signature: string; record: any }> {
    const [profilePda] = this.getUserProfilePda(recordOwner);
    const [recordPda] = this.getHealthRecordPda(recordOwner, recordId);

    // Check if we need access grant
    let accessGrantPda = null;
    if (!recordOwner.equals(this.wallet.publicKey)) {
      [accessGrantPda] = this.getAccessGrantPda(
        recordOwner,
        this.wallet.publicKey,
      );
    }

    const tx = await this.program.methods
      .readRecord(new BN(recordId))
      .accounts({
        profile: profilePda,
        record: recordPda,
        recordOwner: recordOwner,
        accessor: this.wallet.publicKey,
        accessGrant: accessGrantPda,
      })
      .rpc();

    // Fetch the record data
    const record = await this.program.account.healthRecord.fetch(recordPda);

    console.log("Health record accessed:", tx);
    return { signature: tx, record };
  }

  /**
   * Get user's health profile
   */
  async getUserProfile(userPubkey?: PublicKey) {
    const pubkey = userPubkey || this.wallet.publicKey;
    const [profilePda] = this.getUserProfilePda(pubkey);
    return await this.program.account.userHealthProfile.fetch(profilePda);
  }

  /**
   * Get all health records for a user
   */
  async getUserHealthRecords(userPubkey?: PublicKey) {
    const pubkey = userPubkey || this.wallet.publicKey;
    const profile = await this.getUserProfile(pubkey);
    const records = [];

    for (let i = 0; i < profile.recordCount.toNumber(); i++) {
      try {
        const [recordPda] = this.getHealthRecordPda(pubkey, i);
        const record = await this.program.account.healthRecord.fetch(recordPda);
        records.push({ id: i, ...record });
      } catch (error) {
        // Record might not exist or be inaccessible
        console.warn(`Could not fetch record ${i}:`, error.message);
      }
    }

    return records;
  }

  /**
   * Get access grants for a user
   */
  async getAccessGrants(userPubkey?: PublicKey) {
    const pubkey = userPubkey || this.wallet.publicKey;

    // This would require indexing in a real application
    // For now, we'll return a placeholder
    console.warn("getAccessGrants requires an indexer for efficient querying");
    return [];
  }

  /**
   * Listen to program events
   */
  setupEventListeners() {
    // Health Profile Events
    this.program.addEventListener("HealthProfileCreated", (event) => {
      console.log("Profile created:", event);
    });

    // Health Record Events
    this.program.addEventListener("HealthRecordAdded", (event) => {
      console.log("Record added:", event);
    });

    this.program.addEventListener("HealthRecordUpdated", (event) => {
      console.log("Record updated:", event);
    });

    this.program.addEventListener("HealthRecordDeleted", (event) => {
      console.log("Record deleted:", event);
    });

    // Access Control Events
    this.program.addEventListener("AccessGranted", (event) => {
      console.log("Access granted:", event);
    });

    this.program.addEventListener("AccessRevoked", (event) => {
      console.log("Access revoked:", event);
    });

    this.program.addEventListener("AuthorizedRecordAccess", (event) => {
      console.log("Record accessed:", event);
    });
  }

  // Helper methods for PDA derivation
  private getUserProfilePda(userPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_profile"), userPubkey.toBuffer()],
      this.program.programId,
    );
  }

  private getHealthRecordPda(
    userPubkey: PublicKey,
    recordId: number,
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("health_record"),
        userPubkey.toBuffer(),
        new BN(recordId).toArrayLike(Buffer, "le", 8),
      ],
      this.program.programId,
    );
  }

  private getAccessGrantPda(
    owner: PublicKey,
    grantee: PublicKey,
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("access_grant"), owner.toBuffer(), grantee.toBuffer()],
      this.program.programId,
    );
  }
}

// Example usage for a mobile health app
export class MobileHealthApp {
  private client: HealthManagementClient;

  constructor(client: HealthManagementClient) {
    this.client = client;
  }

  /**
   * Complete user onboarding flow
   */
  async onboardUser(): Promise<void> {
    try {
      // Initialize health profile
      await this.client.initializeProfile();

      // Setup event listeners
      this.client.setupEventListeners();

      console.log("User onboarded successfully!");
    } catch (error) {
      if (error.message.includes("already in use")) {
        console.log("User already has a profile");
      } else {
        throw error;
      }
    }
  }

  /**
   * Record vital signs from wearable device
   */
  async recordVitalSigns(
    heartRate: number,
    bloodPressure: { systolic: number; diastolic: number },
    temperature: number,
  ): Promise<void> {
    const vitalData = {
      heartRate,
      bloodPressure,
      temperature,
      timestamp: Date.now(),
    };

    // Encrypt data (implement your encryption logic)
    const encryptedData = Buffer.from(JSON.stringify(vitalData));
    const encryptedUri = await this.uploadToIPFS(encryptedData);

    const metadata = `Vital signs recorded at ${new Date().toISOString()}`;

    await this.client.addHealthRecord(
      "vital_signs",
      encryptedUri,
      encryptedData,
      metadata,
    );

    console.log("Vital signs recorded successfully!");
  }

  /**
   * Share records with healthcare provider
   */
  async shareWithDoctor(
    doctorPubkey: PublicKey,
    durationHours: number = 24,
  ): Promise<void> {
    await this.client.grantAccess(doctorPubkey, durationHours, [
      PERMISSION_READ,
      PERMISSION_WRITE,
    ]);

    console.log(`Access granted to doctor for ${durationHours} hours`);
  }

  /**
   * Emergency access scenario
   */
  async grantEmergencyAccess(emergencyContactPubkey: PublicKey): Promise<void> {
    // Grant read-only access for 7 days
    await this.client.grantAccess(
      emergencyContactPubkey,
      24 * 7, // 7 days
      [PERMISSION_READ],
    );

    console.log("Emergency access granted");
  }

  // Mock IPFS upload (implement with your preferred storage solution)
  private async uploadToIPFS(data: Buffer): Promise<string> {
    // This would integrate with IPFS, Arweave, or other decentralized storage
    return `Qm${crypto.randomBytes(22).toString("hex")}`;
  }
}

// Usage example
export async function exampleUsage() {
  // Setup connection and wallet
  const connection = new Connection("https://api.devnet.solana.com");
  const wallet = new Wallet(Keypair.generate());

  // Initialize program (you'd load this from IDL)
  // const program = new Program(idl, PROGRAM_ID, provider);

  // Create client
  // const client = new HealthManagementClient(connection, wallet, program);
  // const app = new MobileHealthApp(client);

  // Onboard user
  // await app.onboardUser();

  // Record some health data
  // await app.recordVitalSigns(72, { systolic: 120, diastolic: 80 }, 98.6);

  // Share with doctor
  // const doctorPubkey = new PublicKey("...");
  // await app.shareWithDoctor(doctorPubkey, 48);
}

export {
  PERMISSION_READ,
  PERMISSION_WRITE,
  PERMISSION_SHARE,
  HealthManagementClient,
  MobileHealthApp,
};
