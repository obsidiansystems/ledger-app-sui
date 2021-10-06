use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;

// Payload for a public key request
pub type Bip32Key = DArray<Byte, U32<{ Endianness::Little }>, 10>;

// Payload for a signature request, content-agnostic.
pub type SignParameters = (
    DArray<U32<{ Endianness::Little }>, Byte, { usize::MAX }>,
    Bip32Key,
);
