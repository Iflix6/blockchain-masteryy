// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {CryptoVerifier} from "../src/CryptoVerifier.sol";

contract CryptoVerifierTest is Test {
    CryptoVerifier verifier;

    function setUp() public {
        verifier = new CryptoVerifier();
    }

    function testDeterministicHash() public view {
        bytes32 a = verifier.hashMessage("hello");
        bytes32 b = verifier.hashMessage("hello");

        assertEq(a, b);
    }

    function testDifferentMessages() public view {
        bytes32 a = verifier.hashMessage("hello");
        bytes32 b = verifier.hashMessage("Hello");

        assertTrue(a != b);
    }

    function testNonceChangesAuthorizationHash() public view {
        bytes32 a = verifier.hashAuthorization(address(0x1234), 100, 1);
        bytes32 b = verifier.hashAuthorization(address(0x1234), 100, 2);

        assertTrue(a != b);
    }
}
