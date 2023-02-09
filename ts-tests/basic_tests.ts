import { sendCommandAndAccept, BASE_URL } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import Sui from "hw-app-sui";
import * as blake2b from "blake2b";
import { instantiate, Nacl } from "js-nacl";

describe('basic tests', () => {

  afterEach( async function() {
    await Axios.post(BASE_URL + "/automation", {version: 1, rules: []});
    await Axios.delete(BASE_URL + "/events");
  });

  it('provides a public key', async () => {

    await sendCommandAndAccept(async (client : Sui) => {
      const rv = await client.getPublicKey("44'/784'/0'");
      expect(new Buffer(rv.publicKey).toString('hex')).to.equal("3a33e8f670428a218e00c16bc6027021a45203eb0ef1fe3bb89e8c125db60eb4");
      expect(new Buffer(rv.address).toString('hex')).to.equal("1eee7846e89d1afbf57b5ad9f7bf105bd853985e");
      return;
    }, []);
  });
});

let nacl : Nacl =null;

instantiate(n => { nacl=n; });

function testTransaction(path: string, txn0: string, prompts: any[]) {
  return async () => {
    await sendCommandAndAccept(async (client : Sui) => {
      const txn = Buffer.from(txn0, "hex");
      const { publicKey } = await client.getPublicKey(path);

      // We don't want the prompts from getPublicKey in our result
      await Axios.delete(BASE_URL + "/events");

      const sig = await client.signTransaction(path, txn);
      expect(sig.signature.length).to.equal(64);
      const pass = nacl.crypto_sign_verify_detached(sig.signature, txn, publicKey);
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
           "prompt": "0x1eee7846e89d1afbf57b5ad9f7bf105bd853985e"
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

  it("can sign a transaction with multiple recipients",
     testTransaction(
       "44'/784'/0'",
       "00000000050205546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e1284af431cf032b5d85324135bf9a3073e920d7f5020000000000000020a06f410c175e828c24cee84cb3bd95cff25c33fbbdcb62c6596e8e423784ffe70fd08074075c7097f361e8b443e2075a852a229281d08074075c7097f361e8b443e2075a852a229282d08074075c7097f361e8b443e2075a852a229283d08074075c7097f361e8b443e2075a852a229284d08074075c7097f361e8b443e2075a852a229285d08074075c7097f361e8b443e2075a852a229286d08074075c7097f361e8b443e2075a852a229287d08074075c7097f361e8b443e2075a852a229288d08074075c7097f361e8b443e2075a852a229289d08074075c7097f361e8b443e2075a852a22928ad08074075c7097f361e8b443e2075a852a22928bd08074075c7097f361e8b443e2075a852a22928cd08074075c7097f361e8b443e2075a852a22928dd08074075c7097f361e8b443e2075a852a22928ed08074075c7097f361e8b443e2075a852a22928f0f0100000000000000020000000000000003000000000000000400000000000000050000000000000006000000000000000700000000000000080000000000000009000000000000000a000000000000000b000000000000000c000000000000000d000000000000000e000000000000000f000000000000001643fb2578ff7191c643079a62c1cca8ec2752bc05546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e101000000000000002c01000000000000",
       [
         {
           "header": "Transfer (1)",
           "prompt": "1 to 0xd08074075c7097f361e8b443e2075a852a229281"
         },
         {
           "header": "Transfer (2)",
           "prompt": "2 to 0xd08074075c7097f361e8b443e2075a852a229282"
         },
         {
           "header": "Transfer (3)",
           "prompt": "3 to 0xd08074075c7097f361e8b443e2075a852a229283"
         },
         {
           "header": "Transfer (4)",
           "prompt": "4 to 0xd08074075c7097f361e8b443e2075a852a229284"
         },
         {
           "header": "Transfer (5)",
           "prompt": "5 to 0xd08074075c7097f361e8b443e2075a852a229285"
         },
         {
           "header": "Transfer (6)",
           "prompt": "6 to 0xd08074075c7097f361e8b443e2075a852a229286"
         },
         {
           "header": "Transfer (7)",
           "prompt": "7 to 0xd08074075c7097f361e8b443e2075a852a229287"
         },
         {
           "header": "Transfer (8)",
           "prompt": "8 to 0xd08074075c7097f361e8b443e2075a852a229288"
         },
         {
           "header": "Transfer (9)",
           "prompt": "9 to 0xd08074075c7097f361e8b443e2075a852a229289"
         },
         {
           "header": "Transfer (10)",
           "prompt": "10 to 0xd08074075c7097f361e8b443e2075a852a22928a"
         },
         {
           "header": "Transfer (11)",
           "prompt": "11 to 0xd08074075c7097f361e8b443e2075a852a22928b"
         },
         {
           "header": "Transfer (12)",
           "prompt": "12 to 0xd08074075c7097f361e8b443e2075a852a22928c"
         },
         {
           "header": "Transfer (13)",
           "prompt": "13 to 0xd08074075c7097f361e8b443e2075a852a22928d"
         },
         {
           "header": "Transfer (14)",
           "prompt": "14 to 0xd08074075c7097f361e8b443e2075a852a22928e"
         },
         {
           "header": "Transfer (15)",
           "prompt": "15 to 0xd08074075c7097f361e8b443e2075a852a22928f"
         },
         {
           "header": "Gas",
           "prompt": "Price: 1, Budget: 300"
         },
         {
           "header": "Sign for Address",
           "prompt": "0x1eee7846e89d1afbf57b5ad9f7bf105bd853985e"
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
