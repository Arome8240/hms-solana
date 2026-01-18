use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct AccessGrant {
     pub owner: Pubkey,
    pub grantee: Pubkey,
   pub expires_at: i64,
    pub permissions: u8,
    pub created_at: i64,
    pub last_updated: i64,
    pub bump: u8,
}

impl AccessGrant {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // grantee
        8 +  // expires_at
        1 +  // permissions
        8 +  // created_at
        8 +  // last_updated
        1;   // bump

    pub fn new(
        owner: Pubkey,
        grantee: Pubkey,
        expires_at: i64,
        permissions: u8,
        bump: u8,
    ) -> Result<Self> {
        let now = Clock::get()?.unix_timestamp;

        require!(
            expires_at > now,
            crate::errors::HealthManagerError::InvalidTimestamp
        );
        require!(
            expires_at <= now + MAX_ACCESS_DURATION,
            crate::errors::HealthManagerError::InvalidAccessDuration
        );
        require!(
            permissions > 0 && permissions <= (PERMISSION_READ | PERMISSION_WRITE | PERMISSION_SHARE),
            crate::errors::HealthManagerError::InvalidPermissions
        );

        Ok(Self {
            owner,
            grantee,
            expires_at,
            permissions,
            created_at: now,
            last_updated: now,
            bump,
        })
    }

    pub fn is_valid(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now < self.expires_at
    }

    pub fn has_permission(&self, permission: u8) -> bool {
        self.is_valid() && (self.permissions & permission) != 0
    }

    pub fn update_permissions(&mut self, permissions: u8) -> Result<()> {
        require!(
            permissions > 0 && permissions <= (PERMISSION_READ | PERMISSION_WRITE | PERMISSION_SHARE),
            crate::errors::HealthManagerError::InvalidPermissions
        );

        self.permissions = permissions;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn extend_expiry(&mut self, new_expires_at: i64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        require!(
            new_expires_at > now,
            crate::errors::HealthManagerError::InvalidTimestamp
        );
        require!(
            new_expires_at <= now + MAX_ACCESS_DURATION,
            crate::errors::HealthManagerError::InvalidAccessDuration
        );

        self.expires_at = new_expires_at;
        self.last_updated = now;
        Ok(())
    }
}