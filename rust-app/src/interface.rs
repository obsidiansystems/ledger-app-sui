use ledger_parser_combinators::bcs::async_parser::*;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;

// Payload for a public key request
pub type Bip32Key = DArray<Byte, U32<{ Endianness::Little }>, 10>;

pub type SignParameters = (IntentMessage, Bip32Key);

// Sui Types
pub type IntentMessage = (Intent, TransactionData);

pub type TransactionData = (
    TransactionKind,
    SuiAddress, // sender
    ObjectRef,  // gas_payment
    Amount,     // gas_price
    Amount,     // gas_budget
);

pub struct SingleTransactionKind;

pub struct TransactionKind;

pub type ObjectRef = (ObjectID, SequenceNumber, ObjectDigest);

pub type Pay = (Coins, Recipients, Amounts);
pub type PayAllSui = (Coins, Recipient);
pub type PaySui = (Coins, RecipientsAndAmounts);

pub struct RecipientsAndAmounts;

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
