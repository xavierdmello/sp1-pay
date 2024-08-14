// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/BonsaiPay.sol";
import "@sp1-contracts/SP1MockVerifier.sol";
import "forge-std/console.sol";

contract BonsaiPayTest is Test {
    BonsaiPay public bonsaiPay;
    SP1MockVerifier public mockVerifier;
    address public constant ALICE = address(0x1);
    uint256 public constant DEPOSIT_AMOUNT = 1 ether;

    struct Fixture {
        address msgSender;
        bytes32 claimId;
        bytes32 vkey;
        bytes publicValues;
        bytes proof;
        bytes cert;
    }

    Fixture public fixture;

    function setUp() public {
        // Deploy mock verifier
        mockVerifier = new SP1MockVerifier();

        // Load fixture
        string memory fixturePath = "src/fixtures/fixture.json";
        string memory fixtureJson = vm.readFile(fixturePath);

        // Parse and decode individual fields
        fixture.msgSender = abi.decode(vm.parseJson(fixtureJson, ".msgSender"), (address));
        fixture.claimId = abi.decode(vm.parseJson(fixtureJson, ".claimId"), (bytes32));
        fixture.vkey = abi.decode(vm.parseJson(fixtureJson, ".vkey"), (bytes32));
        fixture.publicValues = abi.decode(vm.parseJson(fixtureJson, ".publicValues"), (bytes));
        fixture.proof = abi.decode(vm.parseJson(fixtureJson, ".proof"), (bytes));

        // Deploy BonsaiPay
        bonsaiPay = new BonsaiPay(ISP1Verifier(address(mockVerifier)), fixture.vkey, fixture.cert);

        // Fund Alice
        vm.deal(ALICE, 10 ether);
    }

    function testDeposit() public {
        vm.prank(ALICE);
        bonsaiPay.deposit{value: DEPOSIT_AMOUNT}(fixture.claimId);

        assertEq(address(bonsaiPay).balance, DEPOSIT_AMOUNT);
    }

    function testClaim() public {
        // First, make a deposit
        vm.prank(ALICE);
        bonsaiPay.deposit{value: DEPOSIT_AMOUNT}(fixture.claimId);

        // Prepare for claim
        address recipient = fixture.msgSender;
        uint256 initialBalance = recipient.balance;

        // Perform claim
        bonsaiPay.claim(fixture.proof, fixture.publicValues);

        // Check if the recipient received the funds
        assertEq(recipient.balance, initialBalance + DEPOSIT_AMOUNT);
    }

    function testInvalidClaim() public {
        // Attempt to claim without a deposit
        vm.expectRevert(abi.encodeWithSignature("InvalidClaim(string)", "No claimable balance"));
        bonsaiPay.claim(fixture.proof, fixture.publicValues);
    }
}