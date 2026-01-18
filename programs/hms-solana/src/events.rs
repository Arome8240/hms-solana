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