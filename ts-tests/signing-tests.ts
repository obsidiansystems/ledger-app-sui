import { sendCommandAndAccept, BASE_URL, sendCommandExpectFail, toggleBlindSigningSettings, nacl } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';
import Axios from 'axios';
import type Sui from "@mysten/ledgerjs-hw-app-sui";
import * as blake2b from "blake2b";

function testTransaction(path: string, txn0: string, prompts: any[]) {
  return async () => {
    await sendCommandAndAccept(async (client : Sui) => {
      const txn = Buffer.from(txn0, "base64");
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

  it("can sign a transaction 1",
     testTransaction(
       "44'/784'/0'",
       "AAADAAQDZm9vAAQDYmFyAAQDYmF6AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgpkZXZuZXRfbmZ0BG1pbnQAAwEAAAEBAAECAJQy9BaCPvDbdNd/IhA5ISVctUYagfS5sJ2O6wv0m602BU2EWXliIaMDu8lr0JWQLMWDbw8g/hFCIxNpH5Hx4WwWAgAAAAAAAAAgpyULGP03zT4xo+3Q38SWSHquXbkvFnyBAKLAwuDtLXl3DYFt5eIaq0rNA7WwgBpGGfKkwubvXunYYDgWgW+TFQIAAAAAAAAAILOimq+Uf7k4X7grkmX1KeTqqnTP94+4YcQdSSoWJY+O743iclf+MlPNNJuYk//wUhzs4fJwKN2HyHlYp+SYVy4CAAAAAAAAACBb5HtXx9lJ2KBbHFoLehHxjD+LmfNw1YXqsYKdRGqx+vNO6TVM+ua0NRNcyJYmOf07639Ji8gBYz/HUp1NHWqgAgAAAAAAAAAgqqqC0Fr6XSv063+AD+liVqIIFKR5t2PUnV8TMyUhUZv/LC+B/S2BfFKRgcQ+W3GW3xDEsHmeIIowK70K578ysAIAAAAAAAAAIAnFyiiVJ9GTRBBMXFNw9zolPpUBxex37Iv0ZYfNp/u4ed4cr5vC0X9bnC4kmAwhJwH9o+twrKykfriiBlyhpm4BAAAAAAAAAB0CAAAAAAAAAA==",
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
           "prompt": "0xd08074075c7097f361e8b443e2075a852a2292e8",
           "paginate": true
         },
         {
           "header": "Amount",
           "prompt": "0.01"
         },
         {
           "header": "Paying Gas (1/2)",
           "prompt": "At most 300"
         },
         {
           "header": "Paying Gas (2/2)",
           "prompt": "Price 0.000000001"
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

  it("can sign a transaction 2",
     testTransaction(
       "44'/784'/0'",
       "AAAGAAhkAAAAAAAAAAAIyAAAAAAAAAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAQDZm9vAAQDYmFyAAQDYmF6BAIAAQAAAgABAQABAgIAAAIBAAECAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgpkZXZuZXRfbmZ0BG1pbnQAAwEDAAEEAAEFAJAyPotWK7tzGHWXJU5shpzjW+OliT6ZZuCn8FO/7hNpBQ+5G/qw4hT69qWi/+ukUkAyJZQm+oloXDyVQ87ltmNcAgAAAAAAAAAgXKZJFR/QukYiNUJQMut54B2nyr+VYxe1/ALr53KBvapFOub5zQva8ICITSgsDmPGcr+ypVSbCuzcsQxemUkiqQIAAAAAAAAAIHUZTHNX/KfKPJVwaovHpti8wGgiPh0V8zHhHsVeGR9pZk9XzZy03BKicL3mT62DvIq9wz8hIMaHnBgmzpK4EwsCAAAAAAAAACAU+cAh/td0vmJmexAjFIwd/u5ExkAIYnrlcNWapAC9r3n8PaEFrIEqPOmVpi5cKm7vviERZ+P5CSheLY3S6CwIAgAAAAAAAAAgLk3PeFKpfNrMbB+fXD853zDEMq+2yETxOvZW1RhwkNh6g3TIbDnxmkK5tcclx2OBCE9oIevcAJm2N/bdw8HwvwIAAAAAAAAAIGjrwe4ppqe2UEO+0neBZUWj6wZyYHXFAiIseGflEd6916dnCc0UF3M2xa4k2P7w+rjQzx8ClyLeyYKhpmsgqnEBAAAAAAAAAEICAAAAAAAAAA==",
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
           "prompt": "0xd08074075c7097f361e8b443e2075a852a2292e8",
           "paginate": true
         },
         {
           "header": "Amount",
           "prompt": "0.01"
         },
         {
           "header": "Paying Gas (1/2)",
           "prompt": "At most 300"
         },
         {
           "header": "Paying Gas (2/2)",
           "prompt": "Price 0.000000001"
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
})
