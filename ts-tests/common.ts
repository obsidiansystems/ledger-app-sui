import SpeculosTransport from '@ledgerhq/hw-transport-node-speculos';
import Axios from 'axios';
import Transport from "./http-transport";
import Sui from "@mysten/ledgerjs-hw-app-sui";
import { expect } from 'chai';

export const VERSION = {
  major: 0,
  minor: 1,
  patch: 0,
};

const ignoredScreens = [ "Cancel", "Working...", "Quit", "Version"

                         /* App name and version */
                         , "Sui", "ui", `${VERSION.major}.${VERSION.minor}.${VERSION.patch}`

                         , "Settings", "Blind Signing", "Enabled", "Disabled", "Back"
                         /* The next ones are specifically for S+ in which OCR is broken */
                         , "ettings", "Blind igning"
                       ];

const API_PORT: number = 5005;

const BASE_URL: string = `http://127.0.0.1:${API_PORT}`;

const setAcceptAutomationRules = async function() {
  await Axios.post(BASE_URL + "/automation", {
    version: 1,
    rules: [
      ... ignoredScreens.map(txt => { return { "text": txt, "actions": [] } }),
      { "y": 16, "actions": [] },
      { "y": 31, "actions": [] },
      { "y": 46, "actions": [] },
      {
        "text": "Confirm",
        "actions": [
          [ "button", 1, true ],
          [ "button", 2, true ],
          [ "button", 2, false ],
          [ "button", 1, false ],
        ],
      },
      {
        "actions": [
          [ "button", 2, true ],
          [ "button", 2, false ],
        ],
      }
    ]
  });
}

const processPrompts = function(prompts: any[]) {
  const i = prompts.filter((a : any) => !ignoredScreens.includes(a["text"])); // .values();
  let header = "";
  let prompt = "";
  let rv = [];
  for (var ii in i) {
    const value = i[ii];
    if(value["y"] == 0 || value["y"] == 1) { // S is 1, S+ is somehow 0
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

const fixActualPromptsForSPlus = function(prompts: any[]) {
  return prompts.map ( (value) => {
    if (value["text"]) {
      value["x"] = "<patched>";
      value["y"] = "<patched>";
    }
    return value;
  });
}

// HACK to workaround the OCR bug https://github.com/LedgerHQ/speculos/issues/204
const fixRefPromptsForSPlus = function(prompts: any[]) {
  return prompts.map ( (value) => {
    const fixF = (str: string) => {
      return str.replace(/S/g,"").replace(/I/g, "l");
    };
    if (value["header"]) {
      value["header"] = fixF(value["header"]);
      value["prompt"] = fixF(value["prompt"]);
    } else if (value["text"]) {
      value["text"] = fixF(value["text"]);
      value["x"] = "<patched>";
      value["y"] = "<patched>";
    }
    return value;
  });
}

const paginate_prompts = function(page_length: number, prompts: any[]) {
  let rv = [];
  for (var ii in prompts) {
    const value = prompts[ii];
    if (value["paginate"]) {
      const header = value["header"];
      const prompt = value["prompt"];
      let prompt_chunks = prompt.match(new RegExp('.{1,' + page_length + '}', 'g'));
      if (prompt_chunks.length == 1) {
        rv.push({header, prompt});
      } else {
        for (var j in prompt_chunks) {
          const chunk = prompt_chunks[j];
          let header_j = header + " (" + (Number(j) + 1).toString() + "/" + prompt_chunks.length.toString() + ")";
          rv.push({"header": header_j, "prompt": chunk});
        }
      }
    } else {
      rv.push(value);
    }
  }
  return rv;
}

const sendCommandAndAccept = async function(command : any, prompts : any[]) {
  await setAcceptAutomationRules();
  await Axios.delete(BASE_URL + "/events");

  const transport = await Transport.open(BASE_URL + "/apdu");
  const client = new Sui(transport);
  let err = null;

  try { await command(client); } catch(e) {
    err = e;
  }
  if(err) throw(err);

  const actual_prompts = processPrompts((await Axios.get(BASE_URL + "/events")).data["events"] as any[]);
  try {
    expect(actual_prompts).to.deep.equal(paginate_prompts(16, prompts));
  } catch(e) {
    try {
      expect(fixActualPromptsForSPlus(actual_prompts)).to.deep.equal(fixRefPromptsForSPlus(paginate_prompts(48, prompts)));
    } catch (_) {
      // Throw the original error if there is a mismatch as it is generally more useful
      throw(e);
    }
  }
}

const sendCommandExpectFail = async function(command : any) {
  await setAcceptAutomationRules();
  await Axios.delete(BASE_URL + "/events");

  const transport = await Transport.open(BASE_URL + "/apdu");
  const client = new Sui(transport);
  // client.sendChunks = client.sendWithBlocks; // Use Block protocol

  try { await command(client); } catch(e) {
    return;
  }
  expect.fail("Command should have failed");
}

let toggleBlindSigningSettings = async function() {
  await Axios.post(BASE_URL + "/button/right", {"action":"press-and-release"});
  await Axios.post(BASE_URL + "/button/right", {"action":"press-and-release"});
  await Axios.post(BASE_URL + "/button/both", {"action":"press-and-release"});
  await Axios.post(BASE_URL + "/button/both", {"action":"press-and-release"});
  await Axios.post(BASE_URL + "/button/right", {"action":"press-and-release"});
  await Axios.post(BASE_URL + "/button/both", {"action":"press-and-release"});
  await Axios.post(BASE_URL + "/button/left", {"action":"press-and-release"});
  await Axios.post(BASE_URL + "/button/left", {"action":"press-and-release"});
}

export { sendCommandAndAccept, BASE_URL, sendCommandExpectFail, toggleBlindSigningSettings }
