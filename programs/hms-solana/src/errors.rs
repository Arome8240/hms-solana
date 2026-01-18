use anchor_lang::prelude::*;

#[error_code]
pub enum HealthManagerError {
    #[msg("Profile already exists for this user")]
    ProfileAlreadyExists,

    #[msg("Profile not found for this user")]
    ProfileNotFound,

    #[msg("Unauthorized access to health record")]
    UnauthorizedAccess,

    #[msg("Access grant has expired")]
    AccessGrantExpired,

    #[msg("Invalid permissions specified")]
    InvalidPermissions,

    #[msg("Record not found")]
    RecordNotFound,

    #[msg("Record type exceeds maximum length")]
    RecordTypeTooLong,

    #[msg("Encrypted URI exceeds maximum length")]
    EncryptedUriTooLong,

    #[msg("Metadata exceeds maximum length")]
    MetadataTooLong,

    #[msg("Invalid access duration")]
    InvalidAccessDuration,

    #[msg("Cannot grant access to self")]
    CannotGrantAccessToSelf,

    #[msg("Access grant not found")]
    AccessGrantNotFound,

    #[msg("Record is soft deleted")]
    RecordSoftDeleted,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,

    #[msg("Insufficient permissions")]
    InsufficientPermissions,

    // ZK Proof Errors
    #[msg("Invalid ZK proof")]
    InvalidZkProof,

    #[msg("ZK proof verification failed")]
    ZkProofVerificationFailed,

    #[msg("Invalid public inputs for ZK proof")]
    InvalidZkPublicInputs,

    // Emergency Access Errors
    #[msg("Emergency access not configured")]
    EmergencyAccessNotConfigured,

    #[msg("Emergency access cooldown active")]
    EmergencyAccessCooldown,

    #[msg("Maximum emergency contacts exceeded")]
    MaxEmergencyContactsExceeded,

    #[msg("Emergency access already active")]
    EmergencyAccessAlreadyActive,

    // DAO Governance Errors
    #[msg("Insufficient votes for research access")]
    InsufficientResearchVotes,

    #[msg("Research proposal expired")]
    ResearchProposalExpired,

    #[msg("Research proposal not found")]
    ResearchProposalNotFound,

    #[msg("Already voted on this proposal")]
    AlreadyVoted,

    // Wearable Integration Errors
    #[msg("Wearable device not registered")]
    WearableDeviceNotRegistered,

    #[msg("Invalid wearable device signature")]
    InvalidWearableSignature,

    #[msg("Wearable data too old")]
    WearableDataTooOld,

    #[msg("Device ID exceeds maximum length")]
    DeviceIdTooLong,

    // Cross-Device Sync Errors
    #[msg("Sync key mismatch")]
    SyncKeyMismatch,

    #[msg("Sync state not found")]
    SyncStateNotFound,

    #[msg("Invalid sync operation")]
    InvalidSyncOperation,

    #[msg("Sync conflict detected")]
    SyncConflict,
}