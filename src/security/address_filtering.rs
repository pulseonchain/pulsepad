use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// AddressFiltering - Whitelist/blacklist functionality
// Seeds: [b"address_filter", type, address]
// ─────────────────────────────────────────────────────────────────────────────

pub const FILTER_TYPE_WHITELIST: u8 = 0;
pub const FILTER_TYPE_BLACKLIST: u8 = 1;

#[account]
pub struct AddressFilter {
    pub mint: Pubkey,
    pub address: Pubkey,
    pub filter_type: u8, // 0 = whitelist, 1 = blacklist
    pub added_by: Pubkey,
    pub added_at: i64,
    pub bump: u8,
}

impl AddressFilter {
    pub const SEED: &'static [u8] = b"address_filter";

    pub const ACCOUNT_SIZE: usize = 8
        + 32  // mint
        + 32  // address
        + 1   // filter_type
        + 32  // added_by
        + 8   // added_at
        + 1;  // bump

    /// Check if address is in whitelist
    pub fn is_whitelisted(mint: &Pubkey, address: &Pubkey) -> Result<bool> {
        let (key, _) = Pubkey::find_program_address(
            &[SEED, &[FILTER_TYPE_WHITELIST], mint.as_ref(), address.as_ref()],
            &crate::ID,
        );
        
        // Try to load the account
        // In practice, you'd check if the account exists
        Ok(false) // Placeholder
    }

    /// Check if address is blacklisted
    pub fn is_blacklisted(mint: &Pubkey, address: &Pubkey) -> Result<bool> {
        let (key, _) = Pubkey::find_program_address(
            &[SEED, &[FILTER_TYPE_BLACKLIST], mint.as_ref(), address.as_ref()],
            &crate::ID,
        );
        
        // Placeholder - check account existence
        Ok(false)
    }

    /// Add to whitelist
    pub fn add_to_whitelist(
        &mut self,
        mint: &Pubkey,
        address: &Pubkey,
        added_by: &Pubkey,
    ) -> Result<()> {
        self.mint = *mint;
        self.address = *address;
        self.filter_type = FILTER_TYPE_WHITELIST;
        self.added_by = *added_by;
        self.added_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Add to blacklist
    pub fn add_to_blacklist(
        &mut self,
        mint: &Pubkey,
        address: &Pubkey,
        added_by: &Pubkey,
    ) -> Result<()> {
        self.mint = *mint;
        self.address = *address;
        self.filter_type = FILTER_TYPE_BLACKLIST;
        self.added_by = *added_by;
        self.added_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}


