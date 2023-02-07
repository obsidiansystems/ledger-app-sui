import SpeculosTransport from '@ledgerhq/hw-transport-node-speculos';
import Axios from 'axios';
import Transport from "./http-transport";
import { Common } from "hw-app-alamgu";
import { expect } from 'chai';

let ignoredScreens = [ "W e l c o m e", "Cancel", "Working...", "Exit", "Alamgu Example 0.0.1"]

const API_PORT: number = 5005;

const BASE_URL: string = `http://127.0.0.1:${API_PORT}`;

let setAcceptAutomationRules = async function() {
    await Axios.post(BASE_URL + "/automation", {
      version: 1,
      rules: [
        ... ignoredScreens.map(txt => { return { "text": txt, "actions": [] } }),
        { "y": 16, "actions": [] },
        { "y": 31, "actions": [] },
        { "y": 46, "actions": [] },
        { "text": "Confirm", "actions": [ [ "button", 1, true ], [ "button", 2, true ], [ "button", 2, false ], [ "button", 1, false ] ]},
        { "actions": [ [ "button", 2, true ], [ "button", 2, false ] ]}
      ]
    });
}

let processPrompts = function(prompts: [any]) {
  let i = prompts.filter((a : any) => !ignoredScreens.includes(a["text"])); // .values();
  let header = "";
  let prompt = "";
  let rv = [];
  for (var ii in i) {
    let value = i[ii];
    if(value["y"] == 1) {
      if(value["text"] != header) {
        if(header || prompt) rv.push({ header, prompt });
        header = value["text"];
        prompt = "";
      }
    } else if(value["y"] == 16) {
      prompt += value["text"];
    } else if((value["y"] == 31)) {
      prompt += value["text"];
    } else if((value["y"] == 46)) {
      prompt += value["text"];
    } else {
      if(header || prompt) rv.push({ header, prompt });
      rv.push(value);
      header = "";
      prompt = "";
    }
  }
  if (header || prompt) rv.push({ header, prompt });
  return rv;
}

let fixActualPromptsForSPlus = function(prompts: any[]) {
  return prompts.map ( (value) => {
    if (value["text"]) {
      value["x"] = "<patched>";
    }
    return value;
  });
}

// HACK to workaround the OCR bug https://github.com/LedgerHQ/speculos/issues/204
let fixRefPromptsForSPlus = function(prompts: any[]) {
  return prompts.map ( (value) => {
    let fixF = (str: string) => {
      return str.replace(/S/g,"").replace(/I/g, "l");
    };
    if (value["header"]) {
      value["header"] = fixF(value["header"]);
      value["prompt"] = fixF(value["prompt"]);
    } else if (value["text"]) {
      value["text"] = fixF(value["text"]);
      value["x"] = "<patched>";
    }
    return value;
  });
}

let sendCommandAndAccept = async function(command : any, prompts : any) {
  await setAcceptAutomationRules();
  await Axios.delete(BASE_URL + "/events");

  let transport = await Transport.open(BASE_URL + "/apdu");
  let client = new Common(transport, "alamgu-example");
  // client.sendChunks = client.sendWithBlocks; // Use Block protocol
  let err = null;

  try { await command(client); } catch(e) {
    err = e;
  }
  if(err) throw(err);

  let actual_prompts = processPrompts((await Axios.get(BASE_URL + "/events")).data["events"] as [any]);
  try {
    expect(actual_prompts).to.deep.equal(prompts);
  } catch(e) {
    try {
      expect(fixActualPromptsForSPlus(actual_prompts)).to.deep.equal(fixRefPromptsForSPlus(prompts));
    } catch (_) {
      // Throw the original error if there is a mismatch as it is generally more useful
      throw(e);
    }
  }
}

export { sendCommandAndAccept, BASE_URL }
