use anchor_lang::prelude::*;

#[error_code]
pub enum HealthManagerError {
    #[msg("Profile already exists for this user")]
    ProfileAlreadyExists,

    #[msg("Profile not found for this user")]
    ProfileNotFound,

    #[msg("Unauthorized access to health record")]
    UnauthorizedAccess,

    #[msg("Access grant has expired")]
    AccessGrantExpired,

    #[msg("Invalid permissions specified")]
    InvalidPermissions,

    #[msg("Record not found")]
    RecordNotFound,

    #[msg("Record type exceeds maximum length")]
    RecordTypeTooLong,

    #[msg("Encrypted URI exceeds maximum length")]
    EncryptedUriTooLong,

    #[msg("Metadata exceeds maximum length")]
    MetadataTooLong,

    #[msg("Invalid access duration")]
    InvalidAccessDuration,

    #[msg("Cannot grant access to self")]
    CannotGrantAccessToSelf,

    #[msg("Access grant not found")]
    AccessGrantNotFound,

    #[msg("Record is soft deleted")]
    RecordSoftDeleted,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,

    #[msg("Insufficient permissions")]
    InsufficientPermissions,
}