import { ccc, Script } from "@ckb-ccc/core";
import { cccClient } from "./ccc-client";

type Account = {
  lockScript: Script;
  address: string;
  pubKey: string;
};

export const generateAccountFromPrivateKey = async (
  privKey: string,
): Promise<Account> => {
  // Signer object :{
  // client:CccClient,
  // privateKey: Byte32,
  // publicKey: Byte32,
  //
  //}
  const signer = new ccc.SignerCkbPrivateKey(cccClient, privKey);
  const lock = await signer.getAddressObjSecp256k1();

  return {
    lockScript: lock.script,
    address: lock.toString(),
    pubKey: signer.publicKey,
  };
};

export async function capacityOf(address: string): Promise<bigint> {
  const addr = await ccc.Address.fromString(address, cccClient);

  // Get the balance of the address
  let balance = await cccClient.getBalance([addr.script]);
  return balance;
}

export async function transfer(
  toAddress: string,
  amountInCKB: string,
  signerPrivateKey: string,
): Promise<string> {
  // Validate the sender
  const signer = new ccc.SignerCkbPrivateKey(cccClient, signerPrivateKey);
  // Decode the recipient address
  // this should give us the lock script - that only the recipient can unlock
  const { script: toLock } = await ccc.Address.fromString(toAddress, cccClient);

  // Build the full transaction
  const tx = ccc.Transaction.from({
    outputs: [{ lock: toLock }],
    outputsData: [],
  });

  // CCC transactions are easy to be edited
  tx.outputs.forEach((output, i) => {
    if (output.capacity > ccc.fixedPointFrom(amountInCKB)) {
      alert(`Insufficient capacity at output ${i} to store data`);
      return;
    }
    output.capacity = ccc.fixedPointFrom(amountInCKB);
  });

  // Complete missing parts for transaction
  await tx.completeInputsByCapacity(signer);
  //  calculates the fee and if needed adds another input cell/ adjust an output to account for it await tx.completeFeeBy(signer, 1000);
  await tx.completeFeeBy(signer, 1000);
  const txHash = await signer.sendTransaction(tx);
  console.log(
    `Go to explorer to check the sent transaction https://pudge.explorer.nervos.org/transaction/${txHash}`,
  );

  return txHash;
}

export async function wait(seconds: number) {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
}

export function shannonToCKB(amount: bigint) {
  return amount / 100000000n;
}
