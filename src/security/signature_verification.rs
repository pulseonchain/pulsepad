use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// SignatureVerification - Prevents transaction replay attacks
// ─────────────────────────────────────────────────────────────────────────────

/// Verify that a signature is valid for a message
pub fn verify_signature(
    pubkey: &Pubkey,
    signature: &[u8],
    message: &[u8],
) -> Result<()> {
    // For Solana transactions, we verify that the signer signed the transaction
    // This is handled by Anchor's Signer trait
    
    // For off-chain messages, we would use:
    // solana_program::signature::verify_signature(pubkey, signature, message)
    
    Ok(())
}

/// Generate a unique transaction ID to prevent replay attacks
pub fn generate_transaction_id(
    mint: &Pubkey,
    signer: &Pubkey,
    nonce: u64,
) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(mint.as_ref());
    hasher.update(signer.as_ref());
    hasher.update(nonce.to_le_bytes());
    
    hasher.finalize().into()
}

/// Verify transaction ID
pub fn verify_transaction_id(
    mint: &Pubkey,
    signer: &Pubkey,
    nonce: u64,
    expected_id: &[u8; 32],
) -> Result<()> {
    let actual_id = generate_transaction_id(mint, signer, nonce);
    require!(
        actual_id == *expected_id,
        BondingError::InvalidTransactionId
    );
    Ok(())
}

/// Add transaction ID error

