use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct ResearchProposal {
    /// Unique proposal ID
    pub proposal_id: u64,
    /// Researcher requesting access
    pub researcher: Pubkey,
    /// Research topic/description
    pub research_topic: String,
    /// Number of yes votes
    pub yes_votes: u64,
    /// Number of no votes
    pub no_votes: u64,
    /// Total number of unique voters
    pub total_voters: u64,
    /// Timestamp when proposal was created
    pub created_at: i64,
    /// Timestamp when proposal expires
    pub expires_at: i64,
    /// Whether the proposal has been executed
    pub is_executed: bool,
    /// Whether the proposal passed
    pub is_approved: bool,
    /// PDA bump
    pub bump: u8,
}

impl ResearchProposal {
    pub const LEN: usize = 8 + // discriminator
        8 +  // proposal_id
        32 + // researcher
        4 + MAX_METADATA_LENGTH + // research_topic
        8 +  // yes_votes
        8 +  // no_votes
        8 +  // total_voters
        8 +  // created_at
        8 +  // expires_at
        1 +  // is_executed
        1 +  // is_approved
        1;   // bump

    pub fn new(
        proposal_id: u64,
        researcher: Pubkey,
        research_topic: String,
        bump: u8,
    ) -> Result<Self> {
        require!(
            research_topic.len() <= MAX_METADATA_LENGTH,
            crate::errors::HealthManagerError::MetadataTooLong
        );

        let now = Clock::get()?.unix_timestamp;

        Ok(Self {
            proposal_id,
            researcher,
            research_topic,
            yes_votes: 0,
            no_votes: 0,
            total_voters: 0,
            created_at: now,
            expires_at: now + RESEARCH_PROPOSAL_DURATION,
            is_executed: false,
            is_approved: false,
            bump,
        })
    }

    pub fn cast_vote(&mut self, vote: bool) -> Result<()> {
        require!(
            !self.is_expired(),
            crate::errors::HealthManagerError::ResearchProposalExpired
        );

        if vote {
            self.yes_votes += 1;
        } else {
            self.no_votes += 1;
        }
        self.total_voters += 1;

        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now >= self.expires_at
    }

    pub fn can_execute(&self) -> bool {
        !self.is_executed &&
        !self.is_expired() &&
        self.yes_votes >= MIN_RESEARCH_VOTES &&
        self.yes_votes > self.no_votes
    }

    pub fn execute(&mut self) -> Result<()> {
        require!(
            self.can_execute(),
            crate::errors::HealthManagerError::InsufficientResearchVotes
        );

        self.is_executed = true;
        self.is_approved = true;

        Ok(())
    }
}

#[account]
pub struct ResearchVote {
    /// Proposal being voted on
    pub proposal_id: u64,
    /// Voter's public key
    pub voter: Pubkey,
    /// The vote (true = yes, false = no)
    pub vote: bool,
    /// Timestamp when vote was cast
    pub voted_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl ResearchVote {
    pub const LEN: usize = 8 + // discriminator
        8 +  // proposal_id
        32 + // voter
        1 +  // vote
        8 +  // voted_at
        1;   // bump

    pub fn new(
        proposal_id: u64,
        voter: Pubkey,
        vote: bool,
        bump: u8,
    ) -> Result<Self> {
        Ok(Self {
            proposal_id,
            voter,
            vote,
            voted_at: Clock::get()?.unix_timestamp,
            bump,
        })
    }
}