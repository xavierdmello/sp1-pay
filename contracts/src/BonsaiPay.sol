// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;
import "forge-std/console.sol";
import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

contract BonsaiPay {
    ISP1Verifier public immutable verifier;
    bytes32 public immutable bonsaiPayVKey;
    bytes public cert;
    address public owner;

    enum ClaimStatus {
        Pending,
        Claimed
    }

    struct Deposit {
        ClaimStatus status;
        bytes32 claimId;
        uint256 amount;
    }

   struct ProofOutputs {
        address msg_sender;
        bytes32 claim_id;
        bytes cert;
    }

    modifier onlyOwner() {
       assert(msg.sender == owner);
        _;
    }
    
    Deposit[] private deposits;
    mapping(bytes32 => uint256[]) private claimRecords;

    event Deposited(bytes32 indexed claimId, uint256 amount);
    event Claimed(address indexed recipient, bytes32 indexed claimId, uint256 amount);

    error InvalidDeposit(string message);
    error InvalidClaim(string message);
    error TransferFailed();

    constructor(ISP1Verifier _verifier, bytes32 _bonsaiPayVKey, bytes memory _cert) {
        verifier = _verifier;
        bonsaiPayVKey = _bonsaiPayVKey;
        cert = _cert;
        owner = msg.sender;
    }   

    function deposit(bytes32 claimId) public payable {
        if (claimId == bytes32(0)) revert InvalidDeposit("Empty claimId");
        if (msg.value == 0) revert InvalidDeposit("Zero deposit amount");

        deposits.push(Deposit({status: ClaimStatus.Pending, claimId: claimId, amount: msg.value}));
        claimRecords[claimId].push(deposits.length - 1);

        emit Deposited(claimId, msg.value);
    }

    function claim(bytes calldata proof, bytes calldata publicValues) public {
        ProofOutputs memory po = abi.decode(publicValues, (ProofOutputs));

        if (!(po.cert.length == cert.length && keccak256(po.cert) == keccak256(cert))) revert InvalidClaim("Invalid cert");
        if (po.msg_sender == address(0)) revert InvalidClaim("Invalid recipient address");
        if (po.claim_id == bytes32(0)) revert InvalidClaim("Empty claimId");

        verifier.verifyProof(bonsaiPayVKey, publicValues, proof);

        uint256[] storage depositIndices = claimRecords[po.claim_id];
        uint256 balance = _processDeposits(depositIndices);

        if (balance == 0) revert InvalidClaim("No claimable balance");
        (bool success,) = po.msg_sender.call{value: balance}("");

        if (!success) revert TransferFailed();

        emit Claimed(po.msg_sender, po.claim_id, balance);
    }

    function balanceOf(bytes32 claimId) public view returns (uint256) {
        if (claimId == bytes32(0)) revert InvalidClaim("Empty claimId");

        uint256[] storage depositIndices = claimRecords[claimId];
        return _calculateBalance(depositIndices);
    }

    function _processDeposits(uint256[] storage depositIndices) private returns (uint256) {
        uint256 balance = 0;

        for (uint256 i = 0; i < depositIndices.length; ++i) {
            Deposit storage dep = deposits[depositIndices[i]];
            if (dep.status == ClaimStatus.Pending) {
                dep.status = ClaimStatus.Claimed;
                balance += dep.amount;
            }
        }

        return balance;
    }

    function _calculateBalance(uint256[] storage depositIndices) private view returns (uint256) {
        uint256 balance = 0;

        for (uint256 i = 0; i < depositIndices.length; ++i) {
            Deposit storage dep = deposits[depositIndices[i]];
            if (dep.status == ClaimStatus.Pending) {
                balance += dep.amount;
            }
        }

        return balance;
    }

    // Update the JWT certificate - most providers rotate their certificates every so often
    function updateCert(bytes calldata newCert) public onlyOwner {
        cert = newCert;
    }
}
