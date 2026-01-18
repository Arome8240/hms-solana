# Health Management System (HMS) - Solana

A production-ready, privacy-first health management system built on Solana using Rust and Anchor. This program serves as the on-chain coordination layer for mobile health management applications, enabling users to own, control, and selectively share health records without exposing sensitive medical data.

## 🔐 Privacy-First Architecture

**Critical**: This system does NOT store sensitive medical data in plaintext on-chain. Instead, it stores:

- Encrypted references (IPFS/Arweave CIDs)
- Data integrity hashes
- Access permissions and metadata
- Audit trails

## 🏗️ System Architecture

### Core PDAs (Program Derived Addresses)

1. **UserHealthProfile** - One per wallet

   - Root account for user's health data
   - Tracks record count and timestamps
   - Seeds: `["user_profile", user_pubkey]`

2. **HealthRecord** - Per health record

   - Stores encrypted references and metadata
   - Supports soft deletion
   - Seeds: `["health_record", user_pubkey, record_id]`

3. **AccessGrant** - Per access relationship
   - Time-bound permissions (READ, WRITE, SHARE)
   - Automatic expiration validation
   - Seeds: `["access_grant", owner_pubkey, grantee_pubkey]`

### Permission System

Permissions use bitmasks for efficient storage:

- `READ (0x01)`: View record references and metadata
- `WRITE (0x02)`: Update record metadata
- `SHARE (0x04)`: Grant access to others (future enhancement)

## 🚀 Core Instructions

### 1. Profile Management

```rust
// Initialize health profile (once per user)
initialize_profile()
```

### 2. Record Management

```rust
// Add new health record
add_record(record_type, encrypted_uri, data_hash, metadata)

// Update record metadata (owner or authorized users)
update_record(record_id, metadata)

// Soft delete record (owner only)
delete_record(record_id)
```

### 3. Access Control

```rust
// Grant time-bound access to another user
grant_access(grantee, expires_at, permissions)

// Revoke access immediately
revoke_access(grantee)

// Read record with access validation
read_record(record_id)
```

## 📱 Mobile Integration

### Client-Side Flow Example

```typescript
import { PublicKey, Keypair } from "@solana/web3.js";
import { Program, AnchorProvider } from "@coral-xyz/anchor";

// 1. Initialize user profile
const [profilePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("user_profile"), userWallet.publicKey.toBuffer()],
  program.programId,
);

await program.methods
  .initializeProfile()
  .accounts({
    profile: profilePda,
    user: userWallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// 2. Add health record
const recordId = 0; // From profile.record_count
const [recordPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("health_record"),
    userWallet.publicKey.toBuffer(),
    new anchor.BN(recordId).toArrayLike(Buffer, "le", 8),
  ],
  program.programId,
);

await program.methods
  .addRecord(
    "vital_signs",
    "QmIPFSHashOfEncryptedData",
    dataHash, // SHA-256 of original data
    "Blood pressure: 120/80 mmHg",
  )
  .accounts({
    profile: profilePda,
    record: recordPda,
    owner: userWallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// 3. Grant access to doctor
const [accessGrantPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("access_grant"),
    userWallet.publicKey.toBuffer(),
    doctorWallet.publicKey.toBuffer(),
  ],
  program.programId,
);

const expiresAt = new anchor.BN(Date.now() / 1000 + 86400); // 24 hours
const permissions = 1 | 2; // READ | WRITE

await program.methods
  .grantAccess(doctorWallet.publicKey, expiresAt, permissions)
  .accounts({
    profile: profilePda,
    accessGrant: accessGrantPda,
    owner: userWallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Event Listening for Mobile Apps

```typescript
// Listen for health events
program.addEventListener("HealthRecordAdded", (event) => {
  console.log("New record added:", event);
  // Update local cache/UI
});

program.addEventListener("AccessGranted", (event) => {
  console.log("Access granted:", event);
  // Notify user of new access
});
```

## 🔒 Security Features

### Access Control

- **Owner-only mutations**: Only record owners can modify their data
- **Time-bound access**: All grants have expiration timestamps
- **Permission validation**: Granular READ/WRITE/SHARE permissions
- **Automatic expiry**: Expired grants are rejected on-chain

### Data Integrity

- **Hash verification**: SHA-256 hashes ensure data integrity
- **Soft deletion**: Records remain for audit trails
- **Immutable timestamps**: All actions are timestamped
- **Event emission**: Complete audit trail via events

### Privacy Protection

- **No plaintext data**: Only encrypted references stored
- **Off-chain storage**: Sensitive data stored on IPFS/Arweave
- **Selective sharing**: Users control exactly what to share
- **Revocable access**: Instant access revocation

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Build the program
anchor build

# Run tests
anchor test
```

The test suite covers:

- Profile initialization and validation
- Record CRUD operations
- Access control scenarios
- Permission validation
- Error handling

## 📊 Account Sizes & Costs

| Account Type      | Size (bytes) | Rent (SOL) |
| ----------------- | ------------ | ---------- |
| UserHealthProfile | 65           | ~0.0009    |
| HealthRecord      | ~500         | ~0.007     |
| AccessGrant       | 98           | ~0.0014    |

## 🔮 Future Enhancements

### Planned Features

- **ZK-proof access validation**: Zero-knowledge proofs for privacy
- **Emergency access logic**: Break-glass access for emergencies
- **DAO governance**: Community-governed research access
- **Wearable integration**: Automated data ingestion
- **Cross-device sync**: Encrypted synchronization

### Scalability Considerations

- **Indexing**: Use Helius/Triton for efficient querying
- **Compression**: State compression for large datasets
- **Batching**: Batch operations for gas efficiency
- **Caching**: Client-side caching strategies

## 🏥 Healthcare Compliance

### HIPAA Considerations

- **No PHI on-chain**: Personal Health Information stays off-chain
- **Access controls**: Granular permission system
- **Audit trails**: Complete event logging
- **Data minimization**: Only necessary metadata stored

### GDPR Compliance

- **Right to erasure**: Soft deletion with data purging
- **Data portability**: Users own their data references
- **Consent management**: Explicit access grants
- **Privacy by design**: Built-in privacy protections

## 🛠️ Development Setup

```bash
# Install dependencies
yarn install

# Build program
anchor build

# Deploy to localnet
anchor deploy

# Run tests
anchor test
```

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## ⚠️ Disclaimer

This software is provided for educational and development purposes. It has not been audited for production use in healthcare environments. Always consult with legal and compliance experts before deploying in regulated environments.
