use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct EmergencyAccess {
    /// Owner of the health records
    pub owner: Pubkey,
    /// Emergency contact who can access records
    pub emergency_contact: Pubkey,
    /// Whether emergency access is currently active
    pub is_active: bool,
    /// Timestamp when emergency access was last activated
    pub last_activated: i64,
    /// Timestamp when emergency access expires (if active)
    pub expires_at: i64,
    /// Reason for emergency access activation
    pub activation_reason: String,
    /// Who activated the emergency access
    pub activated_by: Pubkey,
    /// Timestamp when configured
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl EmergencyAccess {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // emergency_contact
        1 +  // is_active
        8 +  // last_activated
        8 +  // expires_at
        4 + MAX_METADATA_LENGTH + // activation_reason
        32 + // activated_by
        8 +  // created_at
        1;   // bump

    pub fn new(
        owner: Pubkey,
        emergency_contact: Pubkey,
        bump: u8,
    ) -> Result<Self> {
        let now = Clock::get()?.unix_timestamp;

        Ok(Self {
            owner,
            emergency_contact,
            is_active: false,
            last_activated: 0,
            expires_at: 0,
            activation_reason: String::new(),
            activated_by: Pubkey::default(),
            created_at: now,
            bump,
        })
    }

    pub fn activate(
        &mut self,
        reason: String,
        activated_by: Pubkey,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        // Check cooldown period
        require!(
            now >= self.last_activated + EMERGENCY_COOLDOWN,
            crate::errors::HealthManagerError::EmergencyAccessCooldown
        );

        require!(
            reason.len() <= MAX_METADATA_LENGTH,
            crate::errors::HealthManagerError::MetadataTooLong
        );

        self.is_active = true;
        self.last_activated = now;
        self.expires_at = now + EMERGENCY_ACCESS_DURATION;
        self.activation_reason = reason;
        self.activated_by = activated_by;

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.expires_at = 0;
        self.activation_reason = String::new();
        self.activated_by = Pubkey::default();
    }

    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }

        let now = Clock::get().unwrap().unix_timestamp;
        now < self.expires_at
    }

    pub fn check_and_expire(&mut self) -> bool {
        if self.is_active && !self.is_valid() {
            self.deactivate();
            return true; // Was expired
        }
        false
    }
}