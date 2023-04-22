import { VERSION, sendCommandAndAccept, BASE_URL, sendCommandExpectFail, toggleBlindSigningSettings } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import type Sui from "./Sui";
import * as blake2b from "blake2b";
import { instantiate, Nacl } from "js-nacl";

let nacl : Nacl =null;

instantiate(n => { nacl=n; });

function testTransaction(path: string, txn: Buffer, prompts: any[]) {
  return async () => {
    await sendCommandAndAccept(async (client : Sui) => {

      const { publicKey } = await client.getPublicKey(path);

      // We don't want the prompts from getPublicKey in our result
      await Axios.delete(BASE_URL + "/events");

      const sig = await client.signTransaction(path, txn);
      expect(sig.signature.length).to.equal(64);
      const pass = nacl.crypto_sign_verify_detached(
          sig.signature,
          blake2b(32).update(txn).digest(),
          publicKey,
      );
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
       Buffer.from("000000000002000840420f000000000000204f2370b2a4810ad6c8e1cfd92cc8c8818fef8f59e3a80cea17871f78d850ba4b0202000101000001010200000101006fb21feead027da4873295affd6c4f3618fe176fa2fbf3e7b5ef1d9463b31e210112a6d0c44edc630d2724b1f57fea4f93308b1d22164402c65778bd99379c4733070000000000000020f2fd3c87b227f1015182fe4348ed680d7ed32bcd3269704252c03e1d0b13d30d6fb21feead027da4873295affd6c4f3618fe176fa2fbf3e7b5ef1d9463b31e2101000000000000000c0400000000000000", "hex"),
       [
         {
           "header": "Transfer",
           "prompt": "SUI"
         },
         {
           "header": "From",
           "prompt": "0x56b19e720f3bfa8caaef806afdd5dfaffd0d6ec9476323a14d1638ad734b2ba5",
           "paginate": true
         },
         {
           "header": "To",
           "prompt": "0x4f2370b2a4810ad6c8e1cfd92cc8c8818fef8f59e3a80cea17871f78d850ba4b",
           "paginate": true
         },
         {
           "header": "Amount",
           "prompt": "SUI 0.001"
         },
         {
           "header": "Max Gas",
           "prompt": "SUI 0.000001036"
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

  it("can blind sign an unknown transaction", async function () {
    let path = "44'/784'/0'";
    let txn = Buffer.from("00000000050205546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e1284af431cf032b5d85324135bf9a3073e920d7f5020000000000000020a06f410c175e828c24cee84cb3bd95cff25c33fbbdcb62c6596e8e423784ffe702d08074075c7097f361e8b443e2075a852a2292e8a08074075c7097f361e8b443e2075a852a2292e80180969800000000001643fb2578ff7191c643079a62c1cca8ec2752bc05546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e101000000000000002c01000000000000", "hex");
    let prompts =
       [
         {
           "header": "WARNING",
           "prompt": "Transaction not recognized"
         },
         {
           "header": "Transaction Hash",
           "prompt": "0xfc2bce70e1cb980a6d49a32ff770a782ee13dabdecee085b82e0fdad5e92fcdd"
         },
         {
           "text": "Blind Sign Transaction?",
           "x": 4,
           "y": 11
         },
         {
           "text": "Confirm",
           "x": 43,
           "y": 11,
         }
       ];

    await toggleBlindSigningSettings();
    await Axios.delete(BASE_URL + "/events");
    await testTransaction(path, txn, prompts)();
    await Axios.delete(BASE_URL + "/events");
    // reset back to disabled
    await toggleBlindSigningSettings();
  });

  it("should reject signing a non-SUI coin transaction, if blind signing is not enabled", async function () {
    let path = "44'/784'/0'";
    let txn = Buffer.from("AAAAAAADAQAe2uv1Mds+xCVK5Jv/Dv5cgEl/9DthDcpbjWcsmFpzbs6BNQAAAAAAIKPD8GQqgBpJZRV+nFDRE7rqR0Za8x0pyfLusVdpPPVRAAgADl+jHAAAAAAg5y3MHATlk+Ik5cPIdEz5iPANs1jcXZHVGjh4Mb16lwkCAgEAAAEBAQABAQIAAAECAF/sd27xyQe/W+gY4WRtPlQro1siWQu79s0pxbbCSRafAfnjaU5yJSFFDJznsAaBqbkiR9CB8DJqWki8fn8AUZeQz4E1AAAAAAAgTRU/MsawTJirpVwjDF8gyiEbaT0+7J0V8ifUEGGBkcVf7Hdu8ckHv1voGOFkbT5UK6NbIlkLu/bNKcW2wkkWn+gDAAAAAAAA8NdGAAAAAAAA", "base64");

    await sendCommandExpectFail(async (client : Sui) => {
      await client.signTransaction(path, txn);
    });
  });

  it("should reject signing an unknown transaction, if blind signing is not enabled", async function () {
    let path = "44'/784'/0'";
    let txn = Buffer.from("00000000050205546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e1284af431cf032b5d85324135bf9a3073e920d7f5020000000000000020a06f410c175e828c24cee84cb3bd95cff25c33fbbdcb62c6596e8e423784ffe702d08074075c7097f361e8b443e2075a852a2292e8a08074075c7097f361e8b443e2075a852a2292e80180969800000000001643fb2578ff7191c643079a62c1cca8ec2752bc05546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e101000000000000002c01000000000000", "hex");

    await sendCommandExpectFail(async (client : Sui) => {
      await client.signTransaction(path, txn);
    });
  });

  it("Rejects a blind sign with mismatching lengths", async function () {
    let path = "44'/784'/0'";
    let txn = Buffer.from("00000000050205546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e1284af431cf032b5d85324135bf9a3073e920d7f5020000000000000020a06f410c175e828c24cee84cb3bd95cff25c33fbbdcb62c6596e8e423784ffe702d08074075c7097f361e8b443e2075a852a2292e8a08074075c7097f361e8b443e2075a852a2292e80180969800000000001643fb2578ff7191c643079a62c1cca8ec2752bc05546e7f126d2f40331a543b9608439b582fd0d103000000000000002080fdabcc90498e7eb8413b140c4334871eeafa5a86203fd9cfdb032f604f49e101000000000000002c01000000000000", "hex");

    await toggleBlindSigningSettings();
    await Axios.delete(BASE_URL + "/events");
    await sendCommandExpectFail(async (client : any) => {
      client.oldSendChunks = client.sendChunks;
      client.sendChunks = (cla, ins, p1, p2, payload) => {
        payload[0][3]=payload[0][3]+20; // Add 20*2^24 to the transaction length, so we'll run out of input.
        let rv = client.oldSendChunks(cla, ins, p1, p2, payload);
        return rv;
      }
      await client.signTransaction(path, txn);
    });
    // Check that the app is still running and has not crashed.
    await sendCommandAndAccept(
	  async client => { let { publicKey } = await client.getPublicKey(path); expect(publicKey.length>0).to.equal(true); },
	  [
        {
          "header": "Provide Public Key",
          "prompt": "For Address 0x56b19e720f3bfa8caaef806afdd5dfaffd0d6ec9476323a14d1638ad734b2ba5",
        },
        {
          "text": "Confirm",
          "x": "<patched>",
          "y": "<patched>",
        },
      ]);
    await Axios.delete(BASE_URL + "/events");
    // reset back to disabled
    await toggleBlindSigningSettings();
  });
});

describe("get version tests", function() {
  it("can get app version", async () => {
    await sendCommandAndAccept(async (client : any) => {
      var rv = await client.getVersion();
      expect(rv.major).to.equal(VERSION.major);
      expect(rv.minor).to.equal(VERSION.minor);
      expect(rv.patch).to.equal(VERSION.patch);
      }, []);
    });
});
