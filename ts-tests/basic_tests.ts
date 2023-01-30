import { sendCommandAndAccept, BASE_URL } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import { Common } from "hw-app-obsidian-common";
import * as blake2b from "blake2b";
import { instantiate, Nacl } from "js-nacl";

describe('basic tests', () => {

  afterEach( async function() {
    await Axios.post(BASE_URL + "/automation", {version: 1, rules: []});
    await Axios.delete(BASE_URL + "/events");
  });

  it('provides a public key', async () => {

    await sendCommandAndAccept(async (client : Common) => {
      let rv = await client.getPublicKey("0");
      expect(rv.publicKey).to.equal("8118ad392b9276e348c1473649a3bbb7ec2b39380e40898d25b55e9e6ee94ca3");
      return;
    }, [
      { "header": "Provide Public Key", "prompt": "For Address     8118ad392b9276e348c1473649a3bbb7ec2b39380e40898d25b55e9e6ee94ca3" },
      {
        "text": "Confirm",
        "x": 43,
        "y": 11,
      },
    ]);
  });
});

let nacl : Nacl =null;

instantiate(n => { nacl=n; });

function testTransaction(path: string, txn: string, prompts: any[]) {
     return async () => {
       await sendCommandAndAccept(
         async (client : Common) => {

           let pubkey = (await client.getPublicKey(path)).publicKey;

           // We don't want the prompts from getPublicKey in our result
           await Axios.delete(BASE_URL + "/events");

           let sig = await client.signTransaction(path, txn);
           expect(sig.signature.length).to.equal(128);
           let pass = nacl.crypto_sign_verify_detached(Buffer.from(sig.signature, 'hex'), Buffer.from(txn, "hex"), Buffer.from(pubkey, 'hex'));
           expect(pass).to.equal(true);
         }, prompts);
     }
}

describe("Signing tests", function() {
  before( async function() {
    while(!nacl) await new Promise(r => setTimeout(r, 100));
  })

  it("can sign a transaction",
     testTransaction(
       "44'/784'/0'",
       "00000000050205546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e1284af431cf032b5d85324135bf9a3073e920d7f5020000000000000020a06f410c175e828c24cee84cb3bd95cff25c33fbbdcb62c6596e8e423784ffe701d08074075c7097f361e8b443e2075a852a2292e80180969800000000001643fb2578ff7191c643079a62c1cca8ec2752bc05546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e101000000000000002c01000000000000",
       [
         {
           "header": "Transfer",
           "prompt": "10000000 to 0xd08074075c7097f361e8b443e2075a852a2292e8"
         },
         {
           "header": "Gas",
           "prompt": "Price: 1, Budget: 300"
         },
         {
           "header": "Sign for Address",
           "prompt": "3a33e8f670428a218e00c16bc6027021a45203eb0ef1fe3bb89e8c125db60eb4"
         },
         {
           "text": "Sign Transaction?",
           "x": 19,
           "y": 11
         },
         {
           "text": "Confirm",
           "x": 43,
           "y": 11,
         }
       ]
     ));
});

describe("get version tests", function() {
  it("can get app version", async () => {
    await sendCommandAndAccept(async (client : any) => {
      var rv = await client.getVersion();
      expect(rv.major).to.equal(0);
      expect(rv.minor).to.equal(0);
      expect(rv.patch).to.equal(1);
      }, []);
    });
});
