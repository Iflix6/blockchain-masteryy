use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct VaultState {
    pub owner: [u8; 32],
    pub balance: u64,
    pub nonce: u64,
    pub active: bool,
}

pub fn encode(state: &VaultState) -> Result<Vec<u8>, std::io::Error> {
    borsh::to_vec(state)
}

pub fn decode(data: &[u8]) -> Result<VaultState, std::io::Error> {
    VaultState::try_from_slice(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_round_trip() {
        let original = VaultState {
            owner: [7u8; 32],
            balance: 1000,
            nonce: 2,
            active: true,
        };

        let encoded = encode(&original).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(encoded.len(), 49);
    }

    #[test]
    fn rejects_truncated_state() {
        let state = VaultState {
            owner: [1u8; 32],
            balance: 100,
            nonce: 0,
            active: true,
        };

        let mut bytes = encode(&state).unwrap();
        bytes.pop();

        assert!(decode(&bytes).is_err());
    }
}
