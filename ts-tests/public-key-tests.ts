import { sendCommandAndAccept, BASE_URL, } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import { Common } from "hw-app-alamgu";

describe('public key tests', () => {

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
      },
      {
        "text": "Confirm",
        "x": "<patched>",
        "y": "<patched>",
      },
    ]);
  });
});
