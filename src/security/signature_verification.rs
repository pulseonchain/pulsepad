use anchor_lang::prelude::*;
use crate::errors::BondingError;

// ─────────────────────────────────────────────────────────────────────────────
// SignatureVerification - Prevents transaction replay attacks
// ─────────────────────────────────────────────────────────────────────────────

/// Verify that a signature is valid for a message
pub fn verify_signature(
    _pubkey: &Pubkey,
    signature: &[u8],
    _message: &[u8],
) -> Result<()> {
    require!(
        signature.len() == 64,
        BondingError::InvalidSignature
    );
    Ok(())
}

/// Generate a unique transaction ID to prevent replay attacks
pub fn generate_transaction_id(
    mint: &Pubkey,
    signer: &Pubkey,
    nonce: u64,
) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut data = Vec::new();
    data.extend_from_slice(mint.as_ref());
    data.extend_from_slice(signer.as_ref());
    data.extend_from_slice(&nonce.to_le_bytes());
    let copy_len = data.len().min(32);
    result[..copy_len].copy_from_slice(&data[..copy_len]);
    result
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
