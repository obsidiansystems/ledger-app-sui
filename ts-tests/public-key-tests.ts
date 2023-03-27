import { sendCommandAndAccept, BASE_URL, sendCommandExpectFail, toggleBlindSigningSettings } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import type Sui from "@mysten/ledgerjs-hw-app-sui";
import { GetPublicKeyResult, buildBip32KeyPayload} from "hw-app-alamgu";

describe('public key tests', () => {

  afterEach( async function() {
    await Axios.post(BASE_URL + "/automation", {version: 1, rules: []});
    await Axios.delete(BASE_URL + "/events");
  });

  it('provides a public key', async () => {

    await sendCommandAndAccept(async (client : Sui) => {
      const rv = await getPublicKey(client, "44'/784'/0'", false);
      expect(new Buffer(rv.publicKey).toString('hex')).to.equal("6fc6f39448ad7af0953b78b16d0f840e6fe718ba4a89384239ff20ed088da2fa");
      expect(new Buffer(rv.address).toString('hex')).to.equal("56b19e720f3bfa8caaef806afdd5dfaffd0d6ec9476323a14d1638ad734b2ba5");
      return;
    }, []);
  });

  it('can verify the address', async () => {

    await sendCommandAndAccept(async (client : any) => {
      const rv = await getPublicKey(client, "44'/784'/0'", true);
      expect(new Buffer(rv.publicKey).toString('hex')).to.equal("6fc6f39448ad7af0953b78b16d0f840e6fe718ba4a89384239ff20ed088da2fa");
      expect(new Buffer(rv.address).toString('hex')).to.equal("56b19e720f3bfa8caaef806afdd5dfaffd0d6ec9476323a14d1638ad734b2ba5");
      return;
    }, [
      {
        "header": "Provide Public Key",
        "prompt": "For Address 0x56b19e720f3bfa8caaef806afdd5dfaffd0d6ec9476323a14d1638ad734b2ba5"
      },
      {
        "text": "Confirm",
        "x": 43,
        "y": 11,
      }
    ]);
  });
});

const getPublicKey = async function(
  client: any,
  path: string,
  show_prompt: boolean,
): Promise<GetPublicKeyResult> {
  const cla = 0x00;
  const ins = 0x02;
  const p1 = 0;
  const p2 = 0;
  const payload = buildBip32KeyPayload(path);
  const verify_pubkey = Buffer.alloc(1);
  if (show_prompt) {
    verify_pubkey.writeUInt8(1);
  } else {
    verify_pubkey.writeUInt8(0);
  }
  const response = await client.sendChunks(cla, ins, p1, p2, [payload, verify_pubkey]);
  const keySize = response[0];
  const publicKey = response.slice(1, keySize+1); // slice uses end index.
  let address : Uint8Array | null = null;
  if (response.length > keySize+2) {
    const addressSize = response[keySize+1];
    address = response.slice(keySize+2, keySize+2+addressSize);
  }
  const res: GetPublicKeyResult = {
    publicKey: publicKey,
    address: address,
  };
  return res;
}
