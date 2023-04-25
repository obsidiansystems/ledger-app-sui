import { VERSION, sendCommandAndAccept, BASE_URL, sendCommandExpectFail, toggleBlindSigningSettings } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import { Common } from "hw-app-alamgu";
import * as blake2b from "blake2b";
import { instantiate, Nacl } from "js-nacl";

let nacl : Nacl =null;

instantiate(n => { nacl=n; });

function testTransaction(path: string, txn0: string, prompts: any[]) {
  return async () => {
    await sendCommandAndAccept(async (client : Common) => {
      const txn = Buffer.from(txn0, "hex");
      const { publicKey } = await client.getPublicKey(path);

      // We don't want the prompts from getPublicKey in our result
      await Axios.delete(BASE_URL + "/events");

      const sig = await client.signTransaction(path, txn);
      expect(sig.signature.length).to.equal(64);
      const hash = blake2b(32).update(txn).digest();
      const pass = nacl.crypto_sign_verify_detached(sig.signature, hash, publicKey);
      expect(pass).to.equal(true);
    }, prompts);
  }
}

describe("Signing tests", function() {
  before( async function() {
    while(!nacl) await new Promise(r => setTimeout(r, 100));
  })

  it("should reject signing an unknown transaction, if blind signing is not enabled", async function () {
    let path = "44'/784'/0'";
    let txn = Buffer.from("00000000050205546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e1284af431cf032b5d85324135bf9a3073e920d7f5020000000000000020a06f410c175e828c24cee84cb3bd95cff25c33fbbdcb62c6596e8e423784ffe702d08074075c7097f361e8b443e2075a852a2292e8a08074075c7097f361e8b443e2075a852a2292e80180969800000000001643fb2578ff7191c643079a62c1cca8ec2752bc05546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e101000000000000002c01000000000000", "hex");

    await sendCommandExpectFail(async (client : Common) => {
      await client.signTransaction(path, txn);
    });
  });

  it("can blind sign a transaction", async function () {
   const path = "44'/535348'/0'";
   const txn = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const prompts = [
      {
        "header": "WARNING",
        "prompt": "Transaction not recognized"
      },
      {
        "header": "Transaction hash",
        "prompt": "yC9c_Zn3cjRXV89tJaT4WjCjXsFF4UQWn2Aq2sHjY-4",
      },
      {
        "header": "Sign for Address",
        "prompt": "19e2fea57e82293b4fee8120d934f0c5a4907198f8df29e9a153cfd7d9383488",
      },
      {
        "text": "Blind Sign Transaction?",
        "x": 4,
        "y": 11,
      },
      {
        "text": "Confirm",
        "x": 43,
        "y": 11,
      },
    ];

    await toggleBlindSigningSettings();
    await Axios.delete(BASE_URL + "/events");
    await testTransaction(path, txn, prompts)();
    await Axios.delete(BASE_URL + "/events");
    // reset back to disabled
    await toggleBlindSigningSettings();
  });
});
