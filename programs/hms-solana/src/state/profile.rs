use anchor_lang::prelude::*;

#[account]
pub struct UserHealthProfile {
    /// The owner of this health profile
    pub owner: Pubkey,
    /// Total number of health records created (including soft-deleted)
    pub record_count: u64,
    /// Timestamp when profile was created
    pub created_at: i64,
    /// Timestamp when profile was last updated
    pub last_updated: i64,
    /// Profile bump for PDA validation
    pub bump: u8,
}

impl UserHealthProfile {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        8 +  // record_count
        8 +  // created_at
        8 +  // last_updated
        1;   // bump

    pub fn new(owner: Pubkey, bump: u8) -> Self {
        let now = Clock::get().unwrap().unix_timestamp;
        Self {
            owner,
            record_count: 0,
            created_at: now,
            last_updated: now,
            bump,
        }
    }

    pub fn increment_record_count(&mut self) {
        self.record_count += 1;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }

    pub fn update_timestamp(&mut self) {
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }
}