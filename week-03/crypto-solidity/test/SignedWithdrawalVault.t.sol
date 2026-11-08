// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {SignedWithdrawalVault} from "../src/SignedWithdrawalVault.sol";

contract SignedWithdrawalVaultTest is Test {
    uint256 private constant SIGNER_KEY = 0xA11CE;
    bytes32 private constant VAULT_ID = bytes32(uint256(42));

    SignedWithdrawalVault private vault;
    address private signer;
    address payable private recipient;

    function setUp() public {
        signer = vm.addr(SIGNER_KEY);
        recipient = payable(makeAddr("recipient"));
        vault = new SignedWithdrawalVault(VAULT_ID);
        vm.deal(address(vault), 10 ether);
    }

    function testExecutesSignedWithdrawalAndConsumesNonce() public {
        SignedWithdrawalVault.Withdrawal memory withdrawal = _withdrawal();
        (uint8 v, bytes32 r, bytes32 s) = _sign(withdrawal);

        vault.executeWithdrawal(withdrawal, v, r, s);

        assertEq(recipient.balance, withdrawal.amount);
        assertEq(vault.nonces(signer), 1);
    }

    function testRejectsAlteredAmountAndRecipient() public {
        SignedWithdrawalVault.Withdrawal memory withdrawal = _withdrawal();
        (uint8 v, bytes32 r, bytes32 s) = _sign(withdrawal);

        withdrawal.amount += 1;
        vm.expectRevert(SignedWithdrawalVault.InvalidSignature.selector);
        vault.executeWithdrawal(withdrawal, v, r, s);

        withdrawal = _withdrawal();
        withdrawal.recipient = payable(makeAddr("otherRecipient"));
        vm.expectRevert(SignedWithdrawalVault.InvalidSignature.selector);
        vault.executeWithdrawal(withdrawal, v, r, s);
    }

    function testRejectsExpiredAuthorization() public {
        SignedWithdrawalVault.Withdrawal memory withdrawal = _withdrawal();
        withdrawal.expiration = block.timestamp + 1;
        (uint8 v, bytes32 r, bytes32 s) = _sign(withdrawal);

        vm.warp(withdrawal.expiration);
        vm.expectRevert(SignedWithdrawalVault.Expired.selector);
        vault.executeWithdrawal(withdrawal, v, r, s);

        assertEq(vault.nonces(signer), 0);
    }

    function testRejectsReusedNonce() public {
        SignedWithdrawalVault.Withdrawal memory withdrawal = _withdrawal();
        (uint8 v, bytes32 r, bytes32 s) = _sign(withdrawal);

        vault.executeWithdrawal(withdrawal, v, r, s);
        vm.expectRevert(SignedWithdrawalVault.InvalidNonce.selector);
        vault.executeWithdrawal(withdrawal, v, r, s);
    }

    function testInsufficientFundsDoesNotConsumeNonce() public {
        SignedWithdrawalVault.Withdrawal memory withdrawal = _withdrawal();
        withdrawal.amount = 11 ether;
        (uint8 v, bytes32 r, bytes32 s) = _sign(withdrawal);

        vm.expectRevert(SignedWithdrawalVault.InsufficientBalance.selector);
        vault.executeWithdrawal(withdrawal, v, r, s);

        assertEq(vault.nonces(signer), 0);
    }

    function testTransferFailureDoesNotConsumeNonce() public {
        RejectEther rejectingRecipient = new RejectEther();
        SignedWithdrawalVault.Withdrawal memory withdrawal = _withdrawal();
        withdrawal.recipient = address(rejectingRecipient);
        (uint8 v, bytes32 r, bytes32 s) = _sign(withdrawal);

        vm.expectRevert(SignedWithdrawalVault.TransferFailed.selector);
        vault.executeWithdrawal(withdrawal, v, r, s);

        assertEq(vault.nonces(signer), 0);
    }

    function _withdrawal() private view returns (SignedWithdrawalVault.Withdrawal memory withdrawal) {
        withdrawal = SignedWithdrawalVault.Withdrawal({
            vaultId: VAULT_ID,
            authorizedAccount: signer,
            recipient: recipient,
            amount: 1 ether,
            nonce: 0,
            expiration: block.timestamp + 1 days,
            domainId: block.chainid
        });
    }

    function _sign(SignedWithdrawalVault.Withdrawal memory withdrawal)
        private
        view
        returns (uint8 v, bytes32 r, bytes32 s)
    {
        return vm.sign(SIGNER_KEY, vault.withdrawalDigest(withdrawal));
    }
}

contract RejectEther {
    receive() external payable {
        revert();
    }
}
