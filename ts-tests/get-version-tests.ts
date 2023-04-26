import { VERSION, sendCommandAndAccept } from "./common";
import { expect } from 'chai';
import { describe, it } from 'mocha';

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
