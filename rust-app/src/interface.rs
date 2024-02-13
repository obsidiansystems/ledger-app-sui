use core::convert::TryFrom;
use ledger_device_sdk::io::{ApduHeader, StatusWords};
use ledger_parser_combinators::bcs::async_parser::*;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;
use num_enum::TryFromPrimitive;

// Payload for a public key request
pub type Bip32Key = DArray<Byte, U32<{ Endianness::Little }>, 10>;

pub type SignParameters = (IntentMessage<true>, Bip32Key);

// Sui Types
pub type IntentMessage<const PROMPT: bool> = (Intent, TransactionData<PROMPT>);

pub struct TransactionData<const PROMPT: bool>;

pub type TransactionDataV1<const PROMPT: bool> = (
    TransactionKind<PROMPT>,
    SuiAddress,            // sender
    GasData<PROMPT>,       // gas_data
    TransactionExpiration, // expiration
);

pub struct TransactionKind<const PROMPT: bool>;

pub struct ProgrammableTransaction<const PROMPT: bool>;

pub struct CommandSchema;
pub struct ArgumentSchema;
pub struct CallArgSchema;

pub type GasData<const PROMPT: bool> = (
    Vec<ObjectRef, { usize::MAX }>, // payment
    SuiAddress,                     // owner
    Amount,                         // price
    Amount,                         // budget
);

pub struct TransactionExpiration;
pub type EpochId = U64<{ Endianness::Little }>;

pub type ObjectRef = (ObjectID, SequenceNumber, ObjectDigest);

pub type SharedObject = (
    ObjectID,       // id
    SequenceNumber, // initial_shared_version
    bool,           // mutable
);

pub type AccountAddress = SuiAddress;
pub type ObjectID = AccountAddress;
pub type SequenceNumber = U64LE;
pub type ObjectDigest = SHA3_256_HASH;

pub const SUI_ADDRESS_LENGTH: usize = 32;
pub type SuiAddress = Array<Byte, SUI_ADDRESS_LENGTH>;

pub type Coins = Vec<ObjectRef, { usize::MAX }>;

pub type Recipient = SuiAddress;

pub type Amount = U64LE;

pub type U64LE = U64<{ Endianness::Little }>;
pub type U16LE = U16<{ Endianness::Little }>;

pub type Intent = (IntentVersion, IntentScope, AppId);
pub type IntentVersion = ULEB128;
pub type IntentScope = ULEB128;
pub type AppId = ULEB128;

// TODO: confirm if 33 is indeed ok for all uses of SHA3_256_HASH
#[allow(non_camel_case_types)]
pub type SHA3_256_HASH = Array<Byte, 33>;

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum Ins {
    GetVersion = 0,
    VerifyAddress = 1,
    GetPubkey = 2,
    Sign = 3,
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
