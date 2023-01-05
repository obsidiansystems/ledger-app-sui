use crate::utils::*;
use arrayvec::ArrayVec;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;
use ledger_parser_combinators::interp_parser::{
    DefaultInterp, InterpParser, MoveAction, SubInterp,
};
use core::fmt::Write;


// Try out all possible param types
pub type TestParsersSchema = ((BytesParams, U16Params), (U64Params, DArrayParams));

pub type BytesParams = (Byte, Array<Byte, 32>);
pub type U16Params = (U16<{ Endianness::Big }>, U16<{ Endianness::Little }>);
pub type U32Params = (U32<{ Endianness::Big }>, U32<{ Endianness::Little }>);
pub type U64Params = (U64<{ Endianness::Big }>, U64<{ Endianness::Little }>);
pub type DArrayParams = (DArray<Byte, Byte, 24>, DArray<Byte, U32Params, 4>);

pub type TestParsersImplT = impl InterpParser<TestParsersSchema, Returning = ArrayVec<u8, 128>>;
pub const fn test_parsers_parser() -> TestParsersImplT
{
    MoveAction(((bytes_params_parser(), u16_params_parser()), (u64_params_parser(), darray_params_parser()))
               , mkmvfn(|_:_, destination: _| {
                   let rv = ArrayVec::<u8, 128>::new();
                   *destination = Some(rv);
                   scroller("Parse done", |w| Ok(write!(w, "")?))
               }))
}

pub type BytesParamsT = impl InterpParser<BytesParams, Returning = ()>;
const fn bytes_params_parser() -> BytesParamsT
{
    MoveAction((DefaultInterp, DefaultInterp)
      , mkmvfn(|(v1, v2):(Option<u8>, Option<[u8; 32]>), destination: &mut Option<()>| {
          *destination = Some(());
          scroller("Got Bytes", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:02x?}")?))
    }))
}

pub type U16ParamsT = impl InterpParser<U16Params, Returning = ()>;
const fn u16_params_parser() -> U16ParamsT
{
    MoveAction((DefaultInterp, DefaultInterp)
               , mkmvfn(|(v1, v2):(Option<u16>, Option<u16>), destination: &mut Option<()>| {
                   *destination = Some(());
                   scroller("Got U16", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:?}")?))
               }))
}

pub type U32ParamsT = impl InterpParser<U32Params, Returning = ()>;
const fn u32_params_parser() -> U32ParamsT
{
    MoveAction((DefaultInterp, DefaultInterp)
               , mkmvfn(|(v1, v2):(Option<u32>, Option<u32>), destination: &mut Option<()>| {
                   *destination = Some(());
                   scroller("Got U32", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:?}")?))
               }))
}

pub type U64ParamsT = impl InterpParser<U64Params, Returning = ()>;
const fn u64_params_parser() -> U64ParamsT
{
    MoveAction((DefaultInterp, DefaultInterp)
               , mkmvfn(|(v1, v2):(Option<u64>, Option<u64>), destination: &mut Option<()>| {
                   *destination = Some(());
                   scroller("Got U64", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:?}")?))
               }))
}

pub type DArrayParamsT = impl InterpParser<DArrayParams, Returning = ()>;
const fn darray_params_parser() -> DArrayParamsT
{
    MoveAction((SubInterp(DefaultInterp), SubInterp(u32_params_parser()))
               , mkmvfn(|(v1, _v2):(Option<ArrayVec<u8,24>>, Option<ArrayVec<(),4>>), destination: &mut Option<()>| {
                   *destination = Some(());
                   scroller("Got Darray", |w| Ok(write!(w, "v1: {v1:02x?}")?))
               }))
}
