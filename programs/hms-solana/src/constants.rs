use anchor_lang::prelude::*;

// PDA Seeds
pub const USER_PROFILE_SEED: &[u8] = b"user_profile";
pub const HEALTH_RECORD_SEED: &[u8] = b"health_record";
pub const ACCESS_GRANT_SEED: &[u8] = b"access_grant";

// Permission Bitmasks
pub const PERMISSION_READ: u8 = 1 << 0;   // 0001
pub const PERMISSION_WRITE: u8 = 1 << 1;  // 0010
pub const PERMISSION_SHARE: u8 = 1 << 2;  // 0100

// Account Size Limits
pub const MAX_RECORD_TYPE_LENGTH: usize = 32;
pub const MAX_ENCRYPTED_URI_LENGTH: usize = 128;
pub const MAX_METADATA_LENGTH: usize = 256;

// Time Constants
pub const SECONDS_PER_DAY: i64 = 86_400;
pub const MAX_ACCESS_DURATION: i64 = SECONDS_PER_DAY * 365; // 1 year max