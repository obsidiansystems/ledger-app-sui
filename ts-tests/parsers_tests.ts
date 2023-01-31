import { sendCommandAndAccept } from "./common";

interface TestParsersSchema {
  bytes_params: BytesParams,
  u16_params: U16Params,
  u32_params: U32Params,
  u64_params: U64Params,
  darray_params: DArrayParams,
}

interface BytesParams {
  v1: number,
  v2: string,
}

interface U16Params {
  v1: number,
  v2: number,
}

interface U32Params {
  v1: number,
  v2: number,
}

interface U64Params {
  v1: bigint,
  v2: bigint,
}

interface DArrayParams {
  v1: string,
  v2: number[],
}

function buildPayload(obj: TestParsersSchema): Buffer {
  const bytes_v1 = Buffer.alloc(1);
  bytes_v1.writeUInt8(obj.bytes_params.v1);
  const bytes_v2 = Buffer.alloc(32);
  bytes_v2.write(obj.bytes_params.v2, 0, "hex");
  const u16_v1 = Buffer.alloc(2);
  u16_v1.writeUInt16BE(obj.u16_params.v1);
  const u16_v2 = Buffer.alloc(2);
  u16_v2.writeUInt16LE(obj.u16_params.v2);
  const u64_v1 = Buffer.alloc(8);
  u64_v1.writeBigUInt64BE(obj.u64_params.v1);
  const u64_v2 = Buffer.alloc(8);
  u64_v2.writeBigUInt64LE(obj.u64_params.v2);

  const darrayparams_v1 = Buffer.from(obj.darray_params.v1, "hex");
  const len_v1 = Buffer.alloc(1);
  len_v1.writeUInt8(darrayparams_v1.length);

  const darrayparams_v2 = Buffer.alloc(4 * 2 * obj.darray_params.v2.length);
  for (var i = 0; i < obj.darray_params.v2.length; i++) {
    // Write the same number in LE and BE
    const v = obj.darray_params.v2[i];
    const u32_v1 = Buffer.alloc(4);
    darrayparams_v2.writeUInt32BE(v, i * 4 * 2);
    darrayparams_v2.writeUInt32LE(v, i * 4 * 2 + 4);
  }
  const len_v2 = Buffer.alloc(1);
  len_v2.writeUInt8(obj.darray_params.v2.length);

  return Buffer.concat(
    [ bytes_v1, bytes_v2,
      u16_v1, u16_v2,
      u64_v1, u64_v2,
      len_v1, darrayparams_v1,
      len_v2, darrayparams_v2
    ]);
}

let doTestParsersAPDU = async function(
  client: any,
  obj: TestParsersSchema,
): Promise<void> {
  const cla = 0x00;
  const ins = 0x20;
  const p1 = 0;
  const p2 = 0;

  const payload = buildPayload(obj);

  // there is no return value
  await client.sendChunks(cla, ins, p1, p2, [payload]);
  return;
}

describe('parsers tests', () => {

  // afterEach( async function() {
  //   await Axios.post(BASE_URL + "/automation", {version: 1, rules: []});
  //   await Axios.delete(BASE_URL + "/events");
  // });

  it('can parse a bunch of data', async () => {

    await sendCommandAndAccept(async (client : any) => {
      let obj = {

        bytes_params: {
          v1: 255,
          v2: "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF",
        },
        u16_params:  {
          v1: 345,
          v2: 567,
        },
        u32_params:  {
          v1: 78901234,
          v2: 90123456,
        },
        u64_params:  {
          v1: BigInt("9007199254740992"),
          v2: BigInt("18014398509481982"),
        },
        darray_params:  {
          v1: "12345678",
          v2: [9, 8, 7, 6],
        },
      };
      await doTestParsersAPDU(client, obj);
      return;
    }, [
      {
        "header": "Got Bytes",
        "prompt": "v1: 255, v2: [12, 34, 56, 78, 90, ab, cd, ef, 12, 34, 56, 78, 90, ab, cd, ef, 12, 34, 56, 78, 90, ab, cd, ef, 12, 34, 56, 78, 90, ab, cd, ef]",
      },
      {
        "header": "Got U16",
        "prompt": "v1: 345, v2: 567",
      },
      {
        "header": "Got U64",
        "prompt": "v1: 9007199254740992, v2: 18014398509481982",
      },
      {
        "header": "Got U32",
        "prompt": "v1: 9, v2: 9v1: 8, v2: 8v1: 7, v2: 7v1: 6, v2: 6",
      },
      {
        "header": "Got Darray",
        "prompt": "v1: [12, 34, 56, 78]",
      },
      {
        "header": "Parse done",
        "prompt": "",
      },
    ]);
  });
});
