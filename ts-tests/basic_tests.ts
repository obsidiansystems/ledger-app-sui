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
       let sig = await sendCommandAndAccept(
         async (client : Common) => {

           let pubkey = (await client.getPublicKey(path)).publicKey;

           // We don't want the prompts from getPublicKey in our result
           await Axios.delete(BASE_URL + "/events");

           let sig = await client.signTransaction(path, Buffer.from(txn, "utf-8").toString("hex"));
           expect(sig.signature.length).to.equal(128);
           let hash = blake2b(32).update(Buffer.from(txn, "utf-8")).digest();
           let pass = nacl.crypto_sign_verify_detached(Buffer.from(sig.signature, 'hex'), hash, Buffer.from(pubkey, 'hex'));
           expect(pass).to.equal(true);
         }, prompts);
     }
}

// describe("Signing tests", function() {
//   before( async function() {
//     while(!nacl) await new Promise(r => setTimeout(r, 100));
//   })

//   it("can sign a transaction",
//      testTransaction(
//        "0",
//        JSON.stringify({"testapp":true}),
//        [
//          {
//            "header": "Transaction hash",
//            "prompt": "a5dQl_ZMC3Onv0ldlZ9C-Nl75FXraTHpoipEGTdNzrQ",
//          },
//          {
//            "header": "Sign for Address",
//            "prompt": "7f916b907886913c6dd7ab62681fc52140afbc84"
//          },
//          {
//            "text": "Sign Transaction?",
//            "x": 19,
//            "y": 11
//          },
//          {
//            "text": "Confirm",
//            "x": 43,
//            "y": 11,
//          }
//        ]
//      ));
// });
