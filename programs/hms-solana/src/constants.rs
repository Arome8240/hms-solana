// PDA Seeds
pub const USER_PROFILE_SEED: &[u8] = b"user_profile";
pub const HEALTH_RECORD_SEED: &[u8] = b"health_record";
pub const ACCESS_GRANT_SEED: &[u8] = b"access_grant";
pub const EMERGENCY_ACCESS_SEED: &[u8] = b"emergency_access";
pub const DAO_GOVERNANCE_SEED: &[u8] = b"dao_governance";
pub const WEARABLE_DEVICE_SEED: &[u8] = b"wearable_device";
pub const SYNC_STATE_SEED: &[u8] = b"sync_state";
pub const ZK_PROOF_SEED: &[u8] = b"zk_proof";

// Permission Bitmasks
pub const PERMISSION_READ: u8 = 1 << 0;   // 0001
pub const PERMISSION_WRITE: u8 = 1 << 1;  // 0010
pub const PERMISSION_SHARE: u8 = 1 << 2;  // 0100
pub const PERMISSION_EMERGENCY: u8 = 1 << 3; // 1000
pub const PERMISSION_RESEARCH: u8 = 1 << 4;  // 10000

// Account Size Limits
pub const MAX_RECORD_TYPE_LENGTH: usize = 32;
pub const MAX_ENCRYPTED_URI_LENGTH: usize = 128;
pub const MAX_METADATA_LENGTH: usize = 256;
pub const MAX_DEVICE_ID_LENGTH: usize = 64;
pub const MAX_SYNC_KEY_LENGTH: usize = 32;

// Time Constants
pub const SECONDS_PER_DAY: i64 = 86_400;
pub const MAX_ACCESS_DURATION: i64 = SECONDS_PER_DAY * 365; // 1 year max
pub const EMERGENCY_ACCESS_DURATION: i64 = SECONDS_PER_DAY * 7; // 7 days
pub const WEARABLE_DATA_RETENTION: i64 = SECONDS_PER_DAY * 30; // 30 days

// ZK Proof Constants
pub const ZK_PROOF_SIZE: usize = 256; // Size of ZK proof in bytes
pub const ZK_PUBLIC_INPUT_SIZE: usize = 32; // Size of public inputs

// DAO Governance Constants
pub const MIN_RESEARCH_VOTES: u64 = 100;
pub const RESEARCH_PROPOSAL_DURATION: i64 = SECONDS_PER_DAY * 14; // 2 weeks

// Emergency Access Constants
pub const MAX_EMERGENCY_CONTACTS: usize = 5;
pub const EMERGENCY_COOLDOWN: i64 = SECONDS_PER_DAY; // 24 hours between activations