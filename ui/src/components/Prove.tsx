import React, { useState, useCallback, useEffect } from "react";
import Account from "./Account";
import { useBonsaiPayClaimedEvent, useBonsaiPayBalanceOf } from "../generated";
import { sha256 } from "@noble/hashes/sha256";
import { toHex } from "viem";
import { useContractWrite, usePrepareContractWrite } from "wagmi";
import SP1PayABI from "../abi/SP1Pay";

interface ProveProps {
  disabled: boolean;
  email: string | null;
}

let bonsaiPayAddress = import.meta.env.VITE_CUSTODY_ADDRESS;

const Prove: React.FC<ProveProps> = ({ disabled, email }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [isClaimed, setIsClaimed] = useState(false);
  const [isNonZeroBalance, setIsNonZeroBalance] = useState(false);
  const [proof, setProof] = useState<string | null>(null);
  const [publicValues, setPublicValues] = useState<string | null>(null);
  const [proofId, setProofId] = useState<string | null>(null);
  const [proofExplorerLink, setProofExplorerLink] = useState<string | null>(
    null
  );

  const { config } = usePrepareContractWrite({
    address: bonsaiPayAddress,
    abi: SP1PayABI,
    functionName: "claim",
    args: [proof ? `${proof}` : null, publicValues ? `${publicValues}` : null],
  });

  const {
    data,
    isLoading: isTxLoading,
    isSuccess,
    write,
  } = useContractWrite(config);

  useEffect(() => {
    console.log("Proof: ", proof);
    console.log("Public Values: ", publicValues);
    if (proof && publicValues) {
      write?.();
    }
  }, [proof, publicValues, write]);

  useBonsaiPayClaimedEvent({
    listener: () => {
      setIsClaimed(true);
    },
  });

  const { data: balance } = useBonsaiPayBalanceOf({
    args: [toHex(sha256(email ?? ""))],
  });

  useEffect(() => {
    setIsNonZeroBalance(balance !== 0n);
  }, [balance]);

  const { VITE_API_HOST } = import.meta.env;

  const handleClick = useCallback(async () => {
    setIsLoading(true);

    const jwtCookie = document.cookie
      .split("; ")
      .find((row) => row.startsWith("jwt="));
    const jwt = jwtCookie?.split("=")[1];

    if (!jwt) {
      console.error("JWT not found");
      setIsLoading(false);
      return;
    }

    try {
      // Request the proof
      const requestResponse = await fetch(`${VITE_API_HOST}/api/requestProof`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Auth-Token": jwt,
        },
        body: JSON.stringify({ jwt }),
      });

      if (requestResponse.ok) {
        const { proofId } = await requestResponse.json();
        setProofId(proofId);
        setProofExplorerLink(`https://explorer.succinct.xyz/proof/${proofId}`);

        // Wait for the proof
        const waitResponse = await fetch(`${VITE_API_HOST}/api/waitProof`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ proof_id: proofId }),
        });

        if (waitResponse.ok) {
          const data = await waitResponse.json();
          setProof(data.proof);
          setPublicValues(data.publicValues);
        } else {
          throw new Error("Wait proof response not OK");
        }
      } else {
        throw new Error("Request proof response not OK");
      }
    } catch (error) {
      console.error("Error fetching data:", error);
    } finally {
      setIsLoading(false);
    }
  }, [VITE_API_HOST]);

  return (
    <>
      <Account email={email} disabled={disabled} hideClaim={true} />
      <button
        onClick={handleClick}
        disabled={isLoading || disabled || isClaimed || !isNonZeroBalance}
      >
        {isClaimed ? "Claimed" : isLoading ? "Proving..." : "Prove with SP1"}
      </button>
      {proofExplorerLink && (
        <p>
          <a href={proofExplorerLink} target="_blank" rel="noopener noreferrer">
            View In-Progress Proof on Explorer
          </a>
        </p>
      )}
      {isLoading && <p>This will take ~5 minutes...</p>}
    </>
  );
};

export default Prove;
