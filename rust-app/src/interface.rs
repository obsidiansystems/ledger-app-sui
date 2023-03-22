use ledger_parser_combinators::bcs::async_parser::*;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;

// Payload for a public key request
pub type Bip32Key = DArray<Byte, U32<{ Endianness::Little }>, 10>;

pub type SignParameters = (IntentMessage<true>, Bip32Key);

// Sui Types
pub type IntentMessage<const PROMPT: bool> = (Intent, TransactionData<PROMPT>);

pub struct TransactionData;

pub type TransactionDataV1<const PROMPT: bool> = (
    TransactionKind<PROMPT>,
    SuiAddress, // sender
    GasData<PROMPT>, // gas_data
    TransactionExpiration, // expiration
);

pub type GasData<const PROMPT: bool> = (
    Vec<ObjectRef, { usize::MAX }>,  // payment
    SuiAddress, // owner
    Amount, // price
    Amount, // budget
);

pub struct TransactionExpiration;
pub struct TransactionExpiration_None;
pub type TransactionExpiration_Epoch = EpochId;

pub struct TransactionKind<const PROMPT: bool>;

pub type ProgrammableTransaction<const PROMPT: bool> = (
    Vec<CallArg, { usize::MAX }>,  // inputs
    Vec<Comand, { usize::MAX }>,  // commands
)

pub struct CallArg;
pub type CallArgPure = Vec<u8, { usize::MAX }>;
pub type CallArgObject = ObjectArg;

pub struct ObjectArg;

pub type ImmOrOwnedObject = ObjectRef;
pub type SharedObject = (
    ObjectId, // id
    SequenceNumber, // initial_shared_version
    bool, // mutable
)

pub type ObjectRef = (ObjectID, SequenceNumber, ObjectDigest);

pub struct RecipientsAndAmounts<const PROMPT: bool>;

pub type AccountAddress = SuiAddress;
pub type ObjectID = AccountAddress;
pub type SequenceNumber = U64<{ Endianness::Little }>;
pub type ObjectDigest = SHA3_256_HASH;

pub const SUI_ADDRESS_LENGTH: usize = 32;
pub const SUI_ADDRESS_LENGTH_OLD: usize = 20;
pub type SuiAddress = Array<Byte, SUI_ADDRESS_LENGTH_OLD>;

pub type Coins = Vec<ObjectRef, { usize::MAX }>;

pub type Recipient = SuiAddress;
pub type Recipients = Vec<Recipient, 1>;

pub type Amount = U64<{ Endianness::Little }>;
pub type Amounts = Vec<Amount, 1>;

pub type Intent = (IntentVersion, IntentScope, AppId);
pub type IntentVersion = ULEB128;
pub type IntentScope = ULEB128;
pub type AppId = ULEB128;

// TODO: confirm if 33 is indeed ok for all uses of SHA3_256_HASH
#[allow(non_camel_case_types)]
pub type SHA3_256_HASH = Array<Byte, 33>;

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

impl From<u8> for Ins {
    fn from(ins: u8) -> Ins {
        match ins {
            0 => Ins::GetVersion,
            2 => Ins::GetPubkey,
            3 => Ins::Sign,
            0x20 => Ins::TestParsers,
            0xfe => Ins::GetVersionStr,
            0xff => Ins::Exit,
            _ => panic!(),
        }
    }
}
