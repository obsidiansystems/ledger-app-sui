import { expect } from 'chai';
import { describe, it } from 'mocha';
import SpeculosTransport from '@ledgerhq/hw-transport-node-speculos';
import Axios from 'axios';
import Transport from "./common";
import { Common } from "hw-app-obsidian-common";
import * as ed from 'noble-ed25519';

let ignoredScreens = [ "W e l c o m e", "Cancel", "Working...", "Exit", "Rust App 0.0.1"]

let setAcceptAutomationRules = async function() {
    await Axios.post("http://localhost:5000/automation", {
      version: 1,
      rules: [
        ... ignoredScreens.map(txt => { return { "text": txt, "actions": [] } }),
        { "y": 16, "actions": [] },
        { "text": "Confirm", "actions": [ [ "button", 1, true ], [ "button", 2, true ], [ "button", 2, false ], [ "button", 1, false ] ]},
        { "actions": [ [ "button", 2, true ], [ "button", 2, false ] ]}
      ]
    });
}

let processPrompts = function(prompts: [any]) {
  let i = prompts.filter((a : any) => !ignoredScreens.includes(a["text"])).values();
  let {done, value} = i.next();
  let header = "";
  let prompt = "";
  let rv = [];
  while(!done) {
    if(value["y"] == 1) {
      if(value["text"] != header) {
        if(header || prompt) rv.push({ header, prompt });
        header = value["text"];
        prompt = "";
      }
    } else if(value["y"] == 16) {
      prompt += value["text"];
    } else {
      if(header || prompt) rv.push({ header, prompt });
      rv.push(value);
      header = "";
      prompt = "";
    }
    ({done, value} = i.next());
  }
  return rv;
}

let sendCommandAndAccept = async function(command : any, prompts : any) {
    await setAcceptAutomationRules();
    await Axios.delete("http://localhost:5000/events");

    let transport = await Transport.open("http://localhost:5000/apdu");
    let client = new Common(transport, "rust-app");
    // client.sendChunks = client.sendWithBlocks; // Use Block protocol
    
    //await new Promise(resolve => setTimeout(resolve, 100));
    
    let err = null;

    try { await command(client); } catch(e) {
      err = e;
    }
    
    //await new Promise(resolve => setTimeout(resolve, 100));

    if(err) throw(err);

    expect(processPrompts((await Axios.get("http://localhost:5000/events")).data["events"] as [any])).to.deep.equal(prompts);
    // expect(((await Axios.get("http://localhost:5000/events")).data["events"] as [any]).filter((a : any) => a["text"] != "W e l c o m e")).to.deep.equal(prompts);
}

describe('basic tests', () => {
  afterEach( async function() {
    await Axios.post("http://localhost:5000/automation", {version: 1, rules: []});
    await Axios.delete("http://localhost:5000/events");
  });

  it('provides a public key', async () => {

    await sendCommandAndAccept(async (client : Common) => {
      let rv = await client.getPublicKey("0");
      expect(rv.publicKey).to.equal("8118ad392b9276e348c1473649a3bbb7ec2b39380e40898d25b55e9e6ee94ca3");
      return;
    }, [
      { "header": "Provide Public Key", "prompt": "For Address     7f916b907886913c6dd7ab62681fc52140afbc84" },
      {
        "text": "Confirm",
        "x": 43,
        "y": 11,
      },
    ]);
  });
  

// function testTransaction(path: string, txn: string, prompts: any[]) {
//      return async () => {
//        let sig = await sendCommandAndAccept(
//          async (client : Common) => {

//            let pk = await client.getPublicKey(path);

//            // We don't want the prompts from getPublicKey in our result
//            await Axios.delete("http://localhost:5000/events");

//            let sig = await client.signTransaction(path, Buffer.from(txn, "utf-8").toString("hex"));

//            expect(await ed.verify(sig.signature, Buffer.from(txn, "utf-8"), pk.publicKey)).to.equal(true);
//          }, prompts);
//      }
// }

// // These tests have been extracted interacting with the testnet via the cli.

// let exampleSend = {
//   "chain_id": "testnet",
//   "entropy": "-7780543831205109370",
//   "fee": [
//     {
//       "amount": "10000",
//       "denom": "upokt"
//     }
//   ],
//   "memo": "Fourth transaction",
//   "msg": {
//     "type": "pos/Send",
//     "value": {
//       "amount": "1000000",
//       "from_address": "db987ccfa2a71b2ec9a56c88c77a7cf66d01d8ba",
//       "to_address": "db987ccfa2a71b2ec9a56c88c77a7cf66d01d8ba"
//     }
//   }
// };

// let exampleUnjail = {
//   "chain_id": "testnet",
//   "entropy": "-8051161335943327787",
//   "fee": [
//     {
//       "amount": "10000",
//       "denom": "upokt"
//     }
//   ],
//   "memo": "",
//   "msg": {
//     "type": "pos/MsgUnjail",
//     "value": {
//       "address": "db987ccfa2a71b2ec9a56c88c77a7cf66d01d8ba"
//     }
//   }
// };

// let exampleStake = {
//   "chain_id": "testnet",
//   "entropy": "2417661502575469960",
//   "fee": [
//     {
//       "amount": "10000",
//       "denom": "upokt"
//     }
//   ],
//   "memo": "",
//   "msg": {
//     "type": "pos/MsgStake",
//     "value": {
//       "chains": [
//         "0034"
//       ],
//       "public_key": {
//         "type": "crypto/ed25519_public_key",
//         "value": "6b62a590bab42ea01383d3209fa719254977fb83624fbd6755d102264ba1adc0"
//       },
//       "service_url": "https://serviceURI.com:3000",
//       "value": "1000000"
//     }
//   }
// };

// let exampleUnstake = {
//   "chain_id": "testnet",
//   "entropy": "-1105361304155186876",
//   "fee": [
//     {
//       "amount": "10000",
//       "denom": "upokt"
//     }
//   ],
//   "memo": "",
//   "msg": {
//     "type": "pos/MsgBeginUnstake",
//     "value": {
//       "validator_address": "db987ccfa2a71b2ec9a56c88c77a7cf66d01d8ba"
//     }
//   }
// };

// describe("Signing tests", function() {
//   it("can sign a simple transfer",
//      testTransaction(
//        "0/0",
//        JSON.stringify(exampleSend),
// [
//          {
//         "header": "Signing",
//         "prompt": "Transaction",
//          },
//          {
//         "header": "For Account",
//         "prompt": "678c1a7a95cdca4812036cb4a2466f033973e962"
//          },
//          {
//         "header": "Send",
//         "prompt": "Transaction",
//          },
//          {
//         "header": "Value",
//         "prompt": "1000000",
//          },
//          {
//         "header": "Transfer from",
//         "prompt": "db987ccfa2a71b2ec9a56c88c77a7cf66d01d8ba",
//          },
//          {
//         "header": "Transfer To",
//         "prompt": "db987ccfa2a71b2ec9a56c88c77a7cf66d01d8ba",
//          },
//          {
//            "text": "Confirm",
//            "x": 43,
//            "y": 11,
//          }
// ]
//      ));
//   it("can sign a simple unjail",
//      testTransaction(
//        "0/0",
//        JSON.stringify(exampleUnjail),
//        [
//         { "header": "Signing",
//           "prompt": "Transaction"
//         },
//         {
//           "header": "For Account",
//           "prompt": "678c1a7a95cdca4812036cb4a2466f033973e962"
//         },
//          {
//            "text": "Confirm",
//            "x": 43,
//            "y": 11,
//          }
//        ]
//        ));

//   it("can sign a simple stake",
//      testTransaction(
//        "0/0",
//        JSON.stringify(exampleStake),
//        [
//         { "header": "Signing",
//           "prompt": "Transaction"
//         },
//         {
//           "header": "For Account",
//           "prompt": "678c1a7a95cdca4812036cb4a2466f033973e962"
//         },
//          {
//         "header": "Stake",
//         "prompt": "Transaction",
//          },
//          {
//         "header": "Chain",
//         "prompt": "0034",
//          },
//          {
//         "header": "Public Key",
//         "prompt": "6b62a590bab42ea01383d3209fa719254977fb83624fbd6755d102264ba1adc0 (crypto/ed25519_public_key)",
//          },
//          {
//         "header": "Service URL",
//         "prompt": "https://serviceURI.com:3000",
//          },
//          {
//         "header": "Value",
//         "prompt": "1000000",
//          },
//          {
//            "text": "Confirm",
//            "x": 43,
//            "y": 11,
//          },
//        ]

//      ));

//   it("can sign a simple unstake",
//      testTransaction(
//        "0/0",
//        JSON.stringify(exampleUnstake),
//        [
//         { "header": "Signing",
//           "prompt": "Transaction"
//         },
//         {
//           "header": "For Account",
//           "prompt": "678c1a7a95cdca4812036cb4a2466f033973e962"
//         },
//         {
//           "header": "Unstake",
//           "prompt": "Transaction"
//         },
//         {
//           "header": "Transfer from",
//           "prompt": "db987ccfa2a71b2ec9a56c88c77a7cf66d01d8ba"
//         },
//         {
//           "text": "Confirm",
//           "x": 43,
//           "y": 11
//         }
//        ]
//      ));
});
