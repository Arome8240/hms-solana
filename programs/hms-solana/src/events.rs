use anchor_lang::prelude::*;

#[event]
pub struct HealthProfileCreated {
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct HealthRecordAdded {
    pub owner: Pubkey,
    pub record_id: u64,
    pub record_type: String,
    pub actor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct HealthRecordUpdated {
    pub owner: Pubkey,
    pub record_id: u64,
    pub actor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct HealthRecordDeleted {
    pub owner: Pubkey,
    pub record_id: u64,
    pub actor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AccessGranted {
    pub owner: Pubkey,
    pub grantee: Pubkey,
    pub permissions: u8,
    pub expires_at: i64,
    pub actor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AccessRevoked {
    pub owner: Pubkey,
    pub grantee: Pubkey,
    pub actor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AuthorizedRecordAccess {
    pub owner: Pubkey,
    pub record_id: u64,
    pub accessor: Pubkey,
    pub timestamp: i64,
}

// ZK Proof Events
#[event]
pub struct ZkProofGenerated {
    pub owner: Pubkey,
    pub proof_hash: [u8; 32],
    pub public_inputs: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct ZkProofVerified {
    pub verifier: Pubkey,
    pub proof_hash: [u8; 32],
    pub verification_result: bool,
    pub timestamp: i64,
}

// Emergency Access Events
#[event]
pub struct EmergencyAccessConfigured {
    pub owner: Pubkey,
    pub emergency_contact: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyAccessActivated {
    pub owner: Pubkey,
    pub emergency_contact: Pubkey,
    pub activator: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyAccessDeactivated {
    pub owner: Pubkey,
    pub emergency_contact: Pubkey,
    pub deactivator: Pubkey,
    pub timestamp: i64,
}

// DAO Governance Events
#[event]
pub struct ResearchProposalCreated {
    pub proposal_id: u64,
    pub researcher: Pubkey,
    pub research_topic: String,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct ResearchVoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote: bool,
    pub timestamp: i64,
}

#[event]
pub struct ResearchAccessGranted {
    pub proposal_id: u64,
    pub researcher: Pubkey,
    pub data_owner: Pubkey,
    pub timestamp: i64,
}

// Wearable Integration Events
#[event]
pub struct WearableDeviceRegistered {
    pub owner: Pubkey,
    pub device_id: String,
    pub device_type: String,
    pub timestamp: i64,
}

#[event]
pub struct WearableDataIngested {
    pub owner: Pubkey,
    pub device_id: String,
    pub data_type: String,
    pub record_id: u64,
    pub timestamp: i64,
}

// Cross-Device Sync Events
#[event]
pub struct SyncStateInitialized {
    pub owner: Pubkey,
    pub device_id: String,
    pub sync_version: u64,
    pub timestamp: i64,
}

#[event]
pub struct SyncOperationCompleted {
    pub owner: Pubkey,
    pub source_device: String,
    pub target_device: String,
    pub sync_version: u64,
    pub records_synced: u64,
    pub timestamp: i64,
}