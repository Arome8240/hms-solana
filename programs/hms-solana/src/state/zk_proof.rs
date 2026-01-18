use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct ZkProofState {
    /// Owner of the ZK proof
    pub owner: Pubkey,
    /// Hash of the ZK proof for verification
    pub proof_hash: [u8; 32],
    /// Public inputs for the ZK proof
    pub public_inputs: [u8; ZK_PUBLIC_INPUT_SIZE],
    /// The actual ZK proof data
    pub proof_data: [u8; ZK_PROOF_SIZE],
    /// Timestamp when proof was generated
    pub created_at: i64,
    /// Timestamp when proof was last verified
    pub last_verified: i64,
    /// Number of successful verifications
    pub verification_count: u64,
    /// Whether the proof is currently valid
    pub is_valid: bool,
    /// PDA bump
    pub bump: u8,
}

impl ZkProofState {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // proof_hash
        ZK_PUBLIC_INPUT_SIZE + // public_inputs
        ZK_PROOF_SIZE + // proof_data
        8 +  // created_at
        8 +  // last_verified
        8 +  // verification_count
        1 +  // is_valid
        1;   // bump

    pub fn new(
        owner: Pubkey,
        proof_hash: [u8; 32],
        public_inputs: [u8; ZK_PUBLIC_INPUT_SIZE],
        proof_data: [u8; ZK_PROOF_SIZE],
        bump: u8,
    ) -> Result<Self> {
        let now = Clock::get()?.unix_timestamp;

        Ok(Self {
            owner,
            proof_hash,
            public_inputs,
            proof_data,
            created_at: now,
            last_verified: 0,
            verification_count: 0,
            is_valid: true,
            bump,
        })
    }

    pub fn verify_proof(&mut self) -> Result<bool> {
        // In a real implementation, this would use a ZK proof verification library
        // For now, we'll simulate verification based on proof hash
        let verification_result = self.simulate_verification();

        if verification_result {
            self.last_verified = Clock::get()?.unix_timestamp;
            self.verification_count += 1;
        }

        self.is_valid = verification_result;
        Ok(verification_result)
    }

    // Simulated verification - replace with actual ZK proof verification
    fn simulate_verification(&self) -> bool {
        // Simple simulation: proof is valid if hash is non-zero
        self.proof_hash != [0u8; 32]
    }

    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }
}