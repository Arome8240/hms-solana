use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::{ResearchProposalCreated, ResearchVoteCast, ResearchAccessGranted},
    state::{ResearchProposal, ResearchVote, UserHealthProfile},
};

#[derive(Accounts)]
#[instruction(proposal_id: u64, research_topic: String)]
pub struct CreateResearchProposal<'info> {
    #[account(
        init,
        payer = researcher,
        space = ResearchProposal::LEN,
        seeds = [DAO_GOVERNANCE_SEED, b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, ResearchProposal>,

    #[account(mut)]
    pub researcher: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_research_proposal(
    ctx: Context<CreateResearchProposal>,
    proposal_id: u64,
    research_topic: String,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let researcher = &ctx.accounts.researcher;

    // Initialize the research proposal
    **proposal = ResearchProposal::new(
        proposal_id,
        researcher.key(),
        research_topic.clone(),
        ctx.bumps.proposal,
    )?;

    // Emit event
    emit!(ResearchProposalCreated {
        proposal_id,
        researcher: researcher.key(),
        research_topic,
        expires_at: proposal.expires_at,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Research proposal {} created by: {}", proposal_id, researcher.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64, vote: bool)]
pub struct VoteOnResearchProposal<'info> {
    #[account(
        mut,
        seeds = [DAO_GOVERNANCE_SEED, b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, ResearchProposal>,

    #[account(
        init,
        payer = voter,
        space = ResearchVote::LEN,
        seeds = [DAO_GOVERNANCE_SEED, b"vote", proposal_id.to_le_bytes().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, ResearchVote>,

    #[account(
        seeds = [USER_PROFILE_SEED, voter.key().as_ref()],
        bump = voter_profile.bump,
        constraint = voter_profile.owner == voter.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub voter_profile: Account<'info, UserHealthProfile>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn vote_on_research_proposal(
    ctx: Context<VoteOnResearchProposal>,
    proposal_id: u64,
    vote: bool,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let vote_record = &mut ctx.accounts.vote_record;
    let voter = &ctx.accounts.voter;

    // Cast the vote on the proposal
    proposal.cast_vote(vote)?;

    // Record the individual vote
    **vote_record = ResearchVote::new(
        proposal_id,
        voter.key(),
        vote,
        ctx.bumps.vote_record,
    )?;

    // Emit event
    emit!(ResearchVoteCast {
        proposal_id,
        voter: voter.key(),
        vote,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Vote cast on proposal {} by: {}", proposal_id, voter.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteResearchProposal<'info> {
    #[account(
        mut,
        seeds = [DAO_GOVERNANCE_SEED, b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, ResearchProposal>,

    pub executor: Signer<'info>,
}

pub fn execute_research_proposal(
    ctx: Context<ExecuteResearchProposal>,
    proposal_id: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let executor = &ctx.accounts.executor;

    // Execute the proposal (this marks it as approved)
    proposal.execute()?;

    msg!("Research proposal {} executed by: {}", proposal_id, executor.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64, record_id: u64)]
pub struct AccessWithResearchGrant<'info> {
    #[account(
        seeds = [DAO_GOVERNANCE_SEED, b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump,
        constraint = proposal.is_approved @ HealthManagerError::InsufficientResearchVotes,
        constraint = proposal.researcher == researcher.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub proposal: Account<'info, ResearchProposal>,

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

    /// CHECK: This is the owner of the record, validated in constraints
    pub record_owner: UncheckedAccount<'info>,

    pub researcher: Signer<'info>,
}

pub fn access_with_research_grant(
    ctx: Context<AccessWithResearchGrant>,
    proposal_id: u64,
    record_id: u64,
) -> Result<()> {
    let proposal = &ctx.accounts.proposal;
    let record = &ctx.accounts.record;
    let researcher = &ctx.accounts.researcher;
    let record_owner = &ctx.accounts.record_owner;

    // Additional validation can be added here
    // For example, checking if the record type matches the research topic

    // Emit event for audit trail
    emit!(ResearchAccessGranted {
        proposal_id,
        researcher: researcher.key(),
        data_owner: record_owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    emit!(crate::events::AuthorizedRecordAccess {
        owner: record_owner.key(),
        record_id,
        accessor: researcher.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Record {} accessed for research by: {}", record_id, researcher.key());
    Ok(())
}