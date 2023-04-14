import { sendCommandAndAccept, BASE_URL, sendCommandExpectFail, toggleBlindSigningSettings } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import type Sui from "@mysten/ledgerjs-hw-app-sui";

describe('public key tests', () => {

  afterEach( async function() {
    await Axios.post(BASE_URL + "/automation", {version: 1, rules: []});
    await Axios.delete(BASE_URL + "/events");
  });

  it('provides a public key', async () => {

    await sendCommandAndAccept(async (client : Sui) => {
      const rv = await client.getPublicKey("44'/784'/0'");
      expect(new Buffer(rv.publicKey).toString('hex')).to.equal("6fc6f39448ad7af0953b78b16d0f840e6fe718ba4a89384239ff20ed088da2fa");
      expect(new Buffer(rv.address).toString('hex')).to.equal("56b19e720f3bfa8caaef806afdd5dfaffd0d6ec9476323a14d1638ad734b2ba5");
      return;
    }, [
      {
        "header": "Provide Public Key",
        "prompt": "For Address 0x56b19e720f3bfa8caaef806afdd5dfaffd0d6ec9476323a14d1638ad734b2ba5",
      },
      {
        "text": "Confirm",
        "x": 43,
        "y": 11,
      },
    ]);
  });
});
