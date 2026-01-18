use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::{ZkProofGenerated, ZkProofVerified},
    state::{UserHealthProfile, ZkProofState},
};

#[derive(Accounts)]
#[instruction(proof_hash: [u8; 32], public_inputs: [u8; 32], proof_data: [u8; 256])]
pub struct GenerateZkProof<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        init,
        payer = owner,
        space = ZkProofState::LEN,
        seeds = [ZK_PROOF_SEED, owner.key().as_ref(), proof_hash.as_ref()],
        bump
    )]
    pub zk_proof: Account<'info, ZkProofState>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn generate_zk_proof(
    ctx: Context<GenerateZkProof>,
    proof_hash: [u8; 32],
    public_inputs: [u8; ZK_PUBLIC_INPUT_SIZE],
    proof_data: [u8; ZK_PROOF_SIZE],
) -> Result<()> {
    let zk_proof = &mut ctx.accounts.zk_proof;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Initialize the ZK proof state
    **zk_proof = ZkProofState::new(
        owner.key(),
        proof_hash,
        public_inputs,
        proof_data,
        ctx.bumps.zk_proof,
    )?;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(ZkProofGenerated {
        owner: owner.key(),
        proof_hash,
        public_inputs,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("ZK proof generated for user: {}", owner.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(proof_hash: [u8; 32])]
pub struct VerifyZkProof<'info> {
    #[account(
        mut,
        seeds = [ZK_PROOF_SEED, proof_owner.key().as_ref(), proof_hash.as_ref()],
        bump = zk_proof.bump,
        constraint = zk_proof.owner == proof_owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub zk_proof: Account<'info, ZkProofState>,

    /// CHECK: This is the owner of the proof, validated in constraints
    pub proof_owner: UncheckedAccount<'info>,

    pub verifier: Signer<'info>,
}

pub fn verify_zk_proof(
    ctx: Context<VerifyZkProof>,
    _proof_hash: [u8; 32],
) -> Result<()> {
    let zk_proof = &mut ctx.accounts.zk_proof;
    let verifier = &ctx.accounts.verifier;

    // Verify the ZK proof
    let verification_result = zk_proof.verify_proof()?;

    require!(
        verification_result,
        HealthManagerError::ZkProofVerificationFailed
    );

    // Emit event
    emit!(ZkProofVerified {
        verifier: verifier.key(),
        proof_hash: zk_proof.proof_hash,
        verification_result,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("ZK proof verified by: {}", verifier.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(record_id: u64, proof_hash: [u8; 32])]
pub struct AccessWithZkProof<'info> {
    #[account(
        seeds = [USER_PROFILE_SEED, record_owner.key().as_ref()],
        bump = profile.bump
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        seeds = [HEALTH_RECORD_SEED, record_owner.key().as_ref(), record_id.to_le_bytes().as_ref()],
        bump = record.bump,
        constraint = record.owner == record_owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = record.is_accessible() @ HealthManagerError::RecordSoftDeleted
    )]
    pub record: Account<'info, crate::state::HealthRecord>,

    #[account(
        seeds = [ZK_PROOF_SEED, accessor.key().as_ref(), proof_hash.as_ref()],
        bump = zk_proof.bump,
        constraint = zk_proof.owner == accessor.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = zk_proof.is_valid @ HealthManagerError::InvalidZkProof
    )]
    pub zk_proof: Account<'info, ZkProofState>,

    /// CHECK: This is the owner of the record, validated in constraints
    pub record_owner: UncheckedAccount<'info>,

    pub accessor: Signer<'info>,
}

pub fn access_with_zk_proof(
    ctx: Context<AccessWithZkProof>,
    record_id: u64,
    _proof_hash: [u8; 32],
) -> Result<()> {
    let _record = &ctx.accounts.record;
    let accessor = &ctx.accounts.accessor;
    let record_owner = &ctx.accounts.record_owner;

    // ZK proof validation is handled by account constraints
    // Additional business logic can be added here

    // Emit event for audit trail
    emit!(crate::events::AuthorizedRecordAccess {
        owner: record_owner.key(),
        record_id,
        accessor: accessor.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Record {} accessed with ZK proof by: {}", record_id, accessor.key());
    Ok(())
}