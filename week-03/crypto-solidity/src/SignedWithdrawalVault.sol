// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract SignedWithdrawalVault {
    struct Withdrawal {
        bytes32 vaultId;
        address authorizedAccount;
        address recipient;
        uint256 amount;
        uint256 nonce;
        uint256 expiration;
        uint256 domainId;
    }

    bytes32 private constant DOMAIN_TYPEHASH =
        keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");
    bytes32 private constant NAME_HASH = keccak256("CryptoLabWithdrawal");
    bytes32 private constant VERSION_HASH = keccak256("1");
    bytes32 private constant WITHDRAWAL_TYPEHASH = keccak256(
        "Withdrawal(bytes32 vaultId,address authorizedAccount,address recipient,uint256 amount,uint256 nonce,uint256 expiration,uint256 domainId)"
    );
    uint256 private constant SECP256K1_HALF_ORDER = 0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0;

    bytes32 public immutable vaultId;
    mapping(address => uint256) public nonces;

    error WrongVault();
    error WrongDomain();
    error Expired();
    error InvalidNonce();
    error ZeroAmount();
    error InsufficientBalance();
    error InvalidSignature();
    error TransferFailed();

    event WithdrawalExecuted(
        address indexed authorizedAccount, address indexed recipient, uint256 amount, uint256 nonce
    );

    constructor(bytes32 vaultId_) {
        vaultId = vaultId_;
    }

    function domainSeparator() public view returns (bytes32) {
        return keccak256(abi.encode(DOMAIN_TYPEHASH, NAME_HASH, VERSION_HASH, block.chainid, address(this)));
    }

    function withdrawalDigest(Withdrawal calldata withdrawal) public view returns (bytes32) {
        bytes32 structHash = keccak256(
            abi.encode(
                WITHDRAWAL_TYPEHASH,
                withdrawal.vaultId,
                withdrawal.authorizedAccount,
                withdrawal.recipient,
                withdrawal.amount,
                withdrawal.nonce,
                withdrawal.expiration,
                withdrawal.domainId
            )
        );
        return keccak256(abi.encodePacked("\x19\x01", domainSeparator(), structHash));
    }

    function executeWithdrawal(Withdrawal calldata withdrawal, uint8 v, bytes32 r, bytes32 s) external {
        if (withdrawal.vaultId != vaultId) revert WrongVault();
        if (withdrawal.domainId != block.chainid) revert WrongDomain();
        if (block.timestamp >= withdrawal.expiration) revert Expired();
        if (withdrawal.nonce != nonces[withdrawal.authorizedAccount]) {
            revert InvalidNonce();
        }
        if (withdrawal.amount == 0) revert ZeroAmount();
        if (address(this).balance < withdrawal.amount) {
            revert InsufficientBalance();
        }
        if (withdrawal.authorizedAccount == address(0)) {
            revert InvalidSignature();
        }
        if (v != 27 && v != 28) revert InvalidSignature();
        if (uint256(s) > SECP256K1_HALF_ORDER) revert InvalidSignature();
        if (ecrecover(withdrawalDigest(withdrawal), v, r, s) != withdrawal.authorizedAccount) {
            revert InvalidSignature();
        }

        nonces[withdrawal.authorizedAccount] = withdrawal.nonce + 1;
        (bool success,) = payable(withdrawal.recipient).call{value: withdrawal.amount}("");
        if (!success) revert TransferFailed();

        emit WithdrawalExecuted(withdrawal.authorizedAccount, withdrawal.recipient, withdrawal.amount, withdrawal.nonce);
    }
}
