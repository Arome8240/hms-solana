use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;
use constants::*;

declare_id!("n3qd2EbGLXbVCJhB1H8pQGDTzHfvDJEPqp4PuhZBiz2");

#[program]
pub mod hms_solana {
    use super::*;

    /// Initialize a health profile for a user
    pub fn initialize_profile(ctx: Context<InitializeProfile>) -> Result<()> {
        instructions::initialize_profile(ctx)
    }

    /// Add a new health record
    pub fn add_record(
        ctx: Context<AddRecord>,
        record_type: String,
        encrypted_uri: String,
        data_hash: [u8; 32],
        metadata: String,
    ) -> Result<()> {
        instructions::add_record(ctx, record_type, encrypted_uri, data_hash, metadata)
    }

    /// Update an existing health record's metadata
    pub fn update_record(
        ctx: Context<UpdateRecord>,
        record_id: u64,
        metadata: String,
    ) -> Result<()> {
        instructions::update_record(ctx, record_id, metadata)
    }

    /// Soft delete a health record (owner only)
    pub fn delete_record(
        ctx: Context<DeleteRecord>,
        record_id: u64,
    ) -> Result<()> {
        instructions::delete_record(ctx, record_id)
    }

    /// Grant access to another user
    pub fn grant_access(
        ctx: Context<GrantAccess>,
        grantee: Pubkey,
        expires_at: i64,
        permissions: u8,
    ) -> Result<()> {
        instructions::grant_access(ctx, grantee, expires_at, permissions)
    }

    /// Revoke access from another user
    pub fn revoke_access(
        ctx: Context<RevokeAccess>,
        grantee: Pubkey,
    ) -> Result<()> {
        instructions::revoke_access(ctx, grantee)
    }

    /// Read a health record (with access control)
    pub fn read_record(
        ctx: Context<ReadRecord>,
        record_id: u64,
    ) -> Result<()> {
        instructions::read_record(ctx, record_id)
    }

    // ZK Proof Instructions
    /// Generate a ZK proof for privacy-preserving access
    pub fn generate_zk_proof(
        ctx: Context<GenerateZkProof>,
        proof_hash: [u8; 32],
        public_inputs: [u8; ZK_PUBLIC_INPUT_SIZE],
        proof_data: [u8; ZK_PROOF_SIZE],
    ) -> Result<()> {
        instructions::generate_zk_proof(ctx, proof_hash, public_inputs, proof_data)
    }

    /// Verify a ZK proof
    pub fn verify_zk_proof(
        ctx: Context<VerifyZkProof>,
        proof_hash: [u8; 32],
    ) -> Result<()> {
        instructions::verify_zk_proof(ctx, proof_hash)
    }

    /// Access record with ZK proof validation
    pub fn access_with_zk_proof(
        ctx: Context<AccessWithZkProof>,
        record_id: u64,
        proof_hash: [u8; 32],
    ) -> Result<()> {
        instructions::access_with_zk_proof(ctx, record_id, proof_hash)
    }

    // Emergency Access Instructions
    /// Configure emergency access for a contact
    pub fn configure_emergency_access(
        ctx: Context<ConfigureEmergencyAccess>,
        emergency_contact: Pubkey,
    ) -> Result<()> {
        instructions::configure_emergency_access(ctx, emergency_contact)
    }

    /// Activate emergency access
    pub fn activate_emergency_access(
        ctx: Context<ActivateEmergencyAccess>,
        emergency_contact: Pubkey,
        reason: String,
    ) -> Result<()> {
        instructions::activate_emergency_access(ctx, emergency_contact, reason)
    }

    /// Deactivate emergency access
    pub fn deactivate_emergency_access(
        ctx: Context<DeactivateEmergencyAccess>,
        emergency_contact: Pubkey,
    ) -> Result<()> {
        instructions::deactivate_emergency_access(ctx, emergency_contact)
    }

    /// Access record with emergency authorization
    pub fn access_with_emergency(
        ctx: Context<AccessWithEmergency>,
        record_id: u64,
        emergency_contact: Pubkey,
    ) -> Result<()> {
        instructions::access_with_emergency(ctx, record_id, emergency_contact)
    }

    // DAO Governance Instructions
    /// Create a research proposal for community voting
    pub fn create_research_proposal(
        ctx: Context<CreateResearchProposal>,
        proposal_id: u64,
        research_topic: String,
    ) -> Result<()> {
        instructions::create_research_proposal(ctx, proposal_id, research_topic)
    }

    /// Vote on a research proposal
    pub fn vote_on_research_proposal(
        ctx: Context<VoteOnResearchProposal>,
        proposal_id: u64,
        vote: bool,
    ) -> Result<()> {
        instructions::vote_on_research_proposal(ctx, proposal_id, vote)
    }

    /// Execute an approved research proposal
    pub fn execute_research_proposal(
        ctx: Context<ExecuteResearchProposal>,
        proposal_id: u64,
    ) -> Result<()> {
        instructions::execute_research_proposal(ctx, proposal_id)
    }

    /// Access record with research grant authorization
    pub fn access_with_research_grant(
        ctx: Context<AccessWithResearchGrant>,
        proposal_id: u64,
        record_id: u64,
    ) -> Result<()> {
        instructions::access_with_research_grant(ctx, proposal_id, record_id)
    }

    // Wearable Integration Instructions
    /// Register a wearable device
    pub fn register_wearable_device(
        ctx: Context<RegisterWearableDevice>,
        device_id: String,
        device_type: String,
        device_pubkey: Pubkey,
    ) -> Result<()> {
        instructions::register_wearable_device(ctx, device_id, device_type, device_pubkey)
    }

    /// Ingest data from wearable device
    pub fn ingest_wearable_data(
        ctx: Context<IngestWearableData>,
        device_id: String,
        batch_id: u64,
        encrypted_data_uri: String,
        data_hash: [u8; 32],
        data_point_count: u32,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<()> {
        instructions::ingest_wearable_data(
            ctx,
            device_id,
            batch_id,
            encrypted_data_uri,
            data_hash,
            data_point_count,
            start_timestamp,
            end_timestamp,
        )
    }

    /// Process wearable data into health record
    pub fn process_wearable_data_to_record(
        ctx: Context<ProcessWearableDataToRecord>,
        device_id: String,
        batch_id: u64,
        record_type: String,
        metadata: String,
    ) -> Result<()> {
        instructions::process_wearable_data_to_record(ctx, device_id, batch_id, record_type, metadata)
    }

    /// Deactivate a wearable device
    pub fn deactivate_wearable_device(
        ctx: Context<DeactivateWearableDevice>,
        device_id: String,
    ) -> Result<()> {
        instructions::deactivate_wearable_device(ctx, device_id)
    }

    // Cross-Device Sync Instructions
    /// Initialize sync state for a device
    pub fn initialize_sync_state(
        ctx: Context<InitializeSyncState>,
        device_id: String,
        encrypted_sync_key: [u8; MAX_SYNC_KEY_LENGTH],
        is_primary: bool,
    ) -> Result<()> {
        instructions::initialize_sync_state(ctx, device_id, encrypted_sync_key, is_primary)
    }

    /// Start a sync operation between devices
    pub fn start_sync_operation(
        ctx: Context<StartSyncOperation>,
        source_device: String,
        target_device: String,
        operation_type: String,
        sync_key: [u8; MAX_SYNC_KEY_LENGTH],
    ) -> Result<()> {
        instructions::start_sync_operation(ctx, source_device, target_device, operation_type, sync_key)
    }

    /// Complete a sync operation successfully
    pub fn complete_sync_operation(
        ctx: Context<CompleteSyncOperation>,
        source_device: String,
        target_device: String,
        records_synced: u64,
        new_state_hash: [u8; 32],
    ) -> Result<()> {
        instructions::complete_sync_operation(ctx, source_device, target_device, records_synced, new_state_hash)
    }

    /// Mark a sync operation as failed
    pub fn fail_sync_operation(
        ctx: Context<FailSyncOperation>,
        source_device: String,
        target_device: String,
        error_message: String,
    ) -> Result<()> {
        instructions::fail_sync_operation(ctx, source_device, target_device, error_message)
    }

    /// Update sync primary device status
    pub fn update_sync_primary(
        ctx: Context<UpdateSyncPrimary>,
        device_id: String,
        is_primary: bool,
    ) -> Result<()> {
        instructions::update_sync_primary(ctx, device_id, is_primary)
    }
}
