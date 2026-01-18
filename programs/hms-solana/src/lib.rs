use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

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
}
