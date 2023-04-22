/********************************************************************************
 *   Ledger Node JS API
 *   (c) 2016-2017 Ledger
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 ********************************************************************************/
import type Transport from "@ledgerhq/hw-transport";
import { Common } from "hw-app-alamgu";
import type { SignTransactionResult, GetVersionResult } from "hw-app-alamgu";

export type { SignTransactionResult, GetVersionResult };

export type GetPublicKeyResult = {
  publicKey: Uint8Array;
  address: Uint8Array;
};

/**
 * Sui API
 *
 * @example
 * import Sui from "hw-app-sui";
 * const sui = new Sui(transport)
 */

export default class Sui extends Common {

  constructor(transport: Transport) {
    super(transport, "SUI");
    this.sendChunks = this.sendWithBlocks;
  }

  /**
    * Retrieves the public key associated with a particular BIP32 path from the ledger
app.
    *
    * @param path - the path to retrieve.
    */
  override async getPublicKey(
    path: string,
  ): Promise<GetPublicKeyResult> {
    const { publicKey, address } = await super.getPublicKey(path);
    if (address == null) {
      throw "should never happen, app always returns address";
    }
    return { publicKey, address };
  }
}

