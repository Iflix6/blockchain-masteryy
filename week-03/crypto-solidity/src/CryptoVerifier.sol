// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract CryptoVerifier {
    function hashMessage(string calldata message) external pure returns (bytes32) {
        return keccak256(bytes(message));
    }

    function hashAuthorization(address user, uint256 amount, uint256 nonce) external view returns (bytes32) {
        return keccak256(abi.encode(block.chainid, address(this), user, amount, nonce));
    }
}
