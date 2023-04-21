import { VERSION, sendCommandAndAccept, BASE_URL, sendCommandExpectFail, toggleBlindSigningSettings } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import { Common } from "hw-app-alamgu";
import * as blake2b from "blake2b";
import { instantiate, Nacl } from "js-nacl";

describe('basic tests', () => {

  afterEach( async function() {
    await Axios.post(BASE_URL + "/automation", {version: 1, rules: []});
    await Axios.delete(BASE_URL + "/events");
  });

  it('provides a public key', async () => {

    await sendCommandAndAccept(async (client : Common) => {
      const rv = await client.getPublicKey("44'/535348'/0'");
      expect(new Buffer(rv.publicKey).toString('hex')).to.equal("19e2fea57e82293b4fee8120d934f0c5a4907198f8df29e9a153cfd7d9383488");
      return;
    }, []);
  });

  it('does address verification', async () => {

    await sendCommandAndAccept(async (client : Common) => {
      const rv = await client.verifyAddress("44'/535348'/0'");
      expect(new Buffer(rv.publicKey).toString('hex')).to.equal("19e2fea57e82293b4fee8120d934f0c5a4907198f8df29e9a153cfd7d9383488");
      expect(new Buffer(rv.address).toString('hex')).to.equal("19e2fea57e82293b4fee8120d934f0c5a4907198f8df29e9a153cfd7d9383488");
      return;
    }, [
      {
        "header": "Provide Public Key",
        "prompt": "",
      },
      {
        "header": "Address",
        "prompt": "19e2fea57e82293b4fee8120d934f0c5a4907198f8df29e9a153cfd7d9383488",
        "paginate": true,
      },
      {
        "text": "Confirm",
        "x": "<patched>",
        "y": "<patched>",
      },
    ]);
  });
});

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

  it("can sign a transaction",
     testTransaction(
       "44'/535348'/0'",
       "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
       [
         {
           "header": "Transaction hash",
           "prompt": "yC9c_Zn3cjRXV89tJaT4WjCjXsFF4UQWn2Aq2sHjY-4",
         },
         {
           "header": "Sign for Address",
           "prompt": "19e2fea57e82293b4fee8120d934f0c5a4907198f8df29e9a153cfd7d9383488"
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
      expect(rv.major).to.equal(VERSION.major);
      expect(rv.minor).to.equal(VERSION.minor);
      expect(rv.patch).to.equal(VERSION.patch);
      }, []);
    });
});
