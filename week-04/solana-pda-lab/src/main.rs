use solana_program::pubkey::Pubkey;

const WITHDRAWAL_SEED: &[u8] = b"withdrawal";

fn derive_withdrawal_pda(program_id: &Pubkey, owner: &Pubkey, nonce: u64) -> (Pubkey, u8) {
    let nonce_seed = nonce.to_le_bytes();
    Pubkey::find_program_address(&[WITHDRAWAL_SEED, owner.as_ref(), &nonce_seed], program_id)
}

fn main() {
    let program_id = Pubkey::new_from_array([7; 32]);
    let owner = Pubkey::new_from_array([9; 32]);
    let nonce = 42;
    let (pda, bump) = derive_withdrawal_pda(&program_id, &owner, nonce);

    println!("Program ID: {program_id}");
    println!("Owner: {owner}");
    println!("Withdrawal nonce: {nonce}");
    println!("Withdrawal PDA: {pda}");
    println!("Bump: {bump}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_program_id() -> Pubkey {
        Pubkey::new_from_array([7; 32])
    }

    fn test_owner() -> Pubkey {
        Pubkey::new_from_array([9; 32])
    }

    #[test]
    fn derivation_is_deterministic_and_recreates_with_bump() {
        let program_id = test_program_id();
        let owner = test_owner();
        let (pda, bump) = derive_withdrawal_pda(&program_id, &owner, 42);
        let (same_pda, same_bump) = derive_withdrawal_pda(&program_id, &owner, 42);
        let nonce_seed = 42u64.to_le_bytes();
        let recreated = Pubkey::create_program_address(
            &[WITHDRAWAL_SEED, owner.as_ref(), &nonce_seed, &[bump]],
            &program_id,
        )
        .unwrap();

        assert_eq!(pda, same_pda);
        assert_eq!(bump, same_bump);
        assert_eq!(pda, recreated);
    }

    #[test]
    fn owner_and_nonce_produce_distinct_pdas() {
        let program_id = test_program_id();
        let owner = test_owner();
        let other_owner = Pubkey::new_from_array([10; 32]);
        let (original, _) = derive_withdrawal_pda(&program_id, &owner, 42);
        let (different_owner, _) = derive_withdrawal_pda(&program_id, &other_owner, 42);
        let (different_nonce, _) = derive_withdrawal_pda(&program_id, &owner, 43);

        assert_ne!(original, different_owner);
        assert_ne!(original, different_nonce);
    }

    #[test]
    fn program_id_is_part_of_pda_derivation() {
        let owner = test_owner();
        let (original, _) = derive_withdrawal_pda(&test_program_id(), &owner, 42);
        let other_program_id = Pubkey::new_from_array([8; 32]);
        let (different_program, _) = derive_withdrawal_pda(&other_program_id, &owner, 42);

        assert_ne!(original, different_program);
    }
}
