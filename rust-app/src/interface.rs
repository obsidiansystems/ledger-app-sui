use core::convert::TryFrom;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;
use ledger_device_sdk::io::{ApduHeader, StatusWords};
use num_enum::TryFromPrimitive;

// Payload for a public key request
pub type Bip32Key = DArray<Byte, U32<{ Endianness::Little }>, 10>;

// Payload for a signature request, content-agnostic.
pub type SignPayload = DArray<U32<{ Endianness::Little }>, Byte, { usize::MAX }>;

pub type SignParameters = (SignPayload, Bip32Key);

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum Ins {
    GetVersion = 0,
    VerifyAddress = 1,
    GetPubkey = 2,
    Sign = 3,
    TestParsers = 0x20,
    GetVersionStr = 0xfe,
    Exit = 0xff,
}

impl TryFrom<ApduHeader> for Ins {
    type Error = StatusWords;
    fn try_from(m: ApduHeader) -> Result<Ins, Self::Error> {
        match m {
            ApduHeader {
                cla: 0,
                ins,
                p1: 0,
                p2: 0,
            } => Self::try_from(ins).map_err(|_| StatusWords::BadIns),
            _ => Err(StatusWords::BadIns),
        }
    }
}
