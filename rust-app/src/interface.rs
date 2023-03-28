use core::convert::TryFrom;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;
use nanos_sdk::io::ApduMeta;

// Payload for a public key request
pub type Bip32Key = DArray<Byte, U32<{ Endianness::Little }>, 10>;

// Payload for a signature request, content-agnostic.
pub type SignPayload = DArray<U32<{ Endianness::Little }>, Byte, { usize::MAX }>;

pub type SignParameters = (SignPayload, Bip32Key);

#[repr(u8)]
#[derive(Debug)]
pub enum Ins {
    GetVersion,
    GetPubkey,
    Sign,
    TestParsers,
    GetVersionStr,
    Exit,
}

impl TryFrom<u8> for Ins {
    type Error = ();
    fn try_from(ins: u8) -> Result<Ins, Self::Error> {
        match ins {
            0 => Ok(Ins::GetVersion),
            2 => Ok(Ins::GetPubkey),
            3 => Ok(Ins::Sign),
            0x20 => Ok(Ins::TestParsers),
            0xfe => Ok(Ins::GetVersionStr),
            0xff => Ok(Ins::Exit),
            _ => Err(()),
        }
    }
}

impl TryFrom<ApduMeta> for Ins {
    type Error = ();
    fn try_from(m: ApduMeta) -> Result<Ins, Self::Error> {
        match m {
            ApduMeta {
                cla: 0,
                ins,
                p1: 0,
                p2: 0,
            } => Self::try_from(ins),
            _ => Err(()),
        }
    }
}
