use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// ProofVerification - Verify various cryptographic proofs
// ─────────────────────────────────────────────────────────────────────────────

/// Verify Whitelist Merkle proof
pub fn verify_whitelist_proof(
    wallet: &Pubkey,
    inclusion_proof: &[u8],
    root_hash: &[u8; 32],
) -> Result<()> {
    // In production, implement proper Merkle proof verification
    // This is a placeholder showing the structure
    
    require!(
        !inclusion_proof.is_empty(),
        BondingError::InvalidProof
    );
    
    // Verify the proof structure
    // - Each element should be 32 bytes (hash)
    // - Total proof length should be 32 * log2(number_of_leaves)
    
    let num_proofs = inclusion_proof.len() / 32;
    require!(
        num_proofs > 0 && num_proofs <= 32, // reasonable max for 2^32 leaves
        BondingError::InvalidProof
    );

    // In actual implementation, compute the root hash from wallet + proof
    // and verify it matches the expected root_hash
    
    Ok(())
}

/// Verify referral proof
pub fn verify_referral_proof(
    referrer: &Pubkey,
    referral_proof: &[u8],
    root_hash: &[u8; 32],
) -> Result<()> {
    require!(
        !referral_proof.is_empty(),
        BondingError::InvalidProof
    );

    // Verify proof structure
    require!(
        referral_proof.len() % 32 == 0,
        BondingError::InvalidProof
    );

    // Compute root and compare with root_hash
    // (Implementation would use Merkle proof logic)

    Ok(())
}

/// Verify token audit proof
pub fn verify_token_audit_proof(
    token_address: &Pubkey,
    audit_proof: &[u8],
    verified_hash: &[u8; 32],
) -> Result<()> {
    require!(
        !audit_proof.is_empty(),
        BondingError::InvalidProof
    );

    // Verify proof structure
    require!(
        audit_proof.len() % 32 == 0,
        BondingError::InvalidProof
    );

    // Compute hash and verify
    // (Implementation would use Merkle proof logic)

    Ok(())
}

/// Verify signature on message
pub fn verify_signature(
    pubkey: &Pubkey,
    signature: &[u8],
    message: &[u8],
) -> Result<()> {
    // Verify signature using Solana's signature verification
    // This is typically done through CPI or precompiled contracts
    
    // For Solana transactions, signature verification is implicit
    // For off-chain messages, use solana_program::signature::verify_signature
    
    // Placeholder implementation
    require!(
        signature.len() == 64, // ed25519 signature size
        BondingError::InvalidSignature
    );

    Ok(())
}

/// Verify account is owned by expected program
pub fn verify_account_owner(
    account: &AccountInfo<'_>,
    expected_owner: &Pubkey,
) -> Result<()> {
    require!(
        account.owner == *expected_owner,
        BondingError::InvalidAccountOwner
    );
    Ok(())
}

/// Verify account rent exemption
pub fn verify_rent_exempt(account: &AccountInfo<'_>) -> Result<()> {
    let rent = Rent::get()?;
    let data_len = account.data_len();
    let minimum_balance = rent.minimum_balance(data_len);
    
    require!(
        account.lamports() >= minimum_balance,
        BondingError::AccountNotRentExempt
    );
    Ok(())
}

/// Add proof verification errors

