use crate::utils::*;
use alamgu_async_block::*;
use arrayvec::ArrayVec;
use core::fmt::Write;
use ledger_parser_combinators::async_parser::*;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::endianness::*;
use ledger_parser_combinators::interp::*;

// Try out all possible param types
pub type TestParsersSchema = ((BytesParams, U16Params), (U64Params, DArrayParams));

pub type BytesParams = (Byte, Array<Byte, 32>);
pub type U16Params = (U16<{ Endianness::Big }>, U16<{ Endianness::Little }>);
pub type U32Params = (U32<{ Endianness::Big }>, U32<{ Endianness::Little }>);
pub type U64Params = (U64<{ Endianness::Big }>, U64<{ Endianness::Little }>);
pub type DArrayParams = (DArray<Byte, Byte, 24>, DArray<Byte, U32Params, 4>);

pub type TestParsersImplT<BS: Readable> =
    impl AsyncParser<TestParsersSchema, BS> + HasOutput<TestParsersSchema, Output = ()>;
pub const fn test_parsers_parser<BS: Readable>() -> TestParsersImplT<BS> {
    Action(
        (
            (bytes_params_parser(), u16_params_parser()),
            (u64_params_parser(), darray_params_parser()),
        ),
        |_| scroller("Parse done", |w| Ok(write!(w, "")?)),
    )
}

pub type BytesParamsT<BS: Readable> =
    impl AsyncParser<BytesParams, BS> + HasOutput<BytesParams, Output = ()>;
const fn bytes_params_parser<BS: Readable>() -> BytesParamsT<BS> {
    Action(
        (DefaultInterp, DefaultInterp),
        |(v1, v2): (Option<u8>, Option<[u8; 32]>)| {
            scroller("Got Bytes", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:02x?}")?))
        },
    )
}

pub type U16ParamsT<BS: Readable> =
    impl AsyncParser<U16Params, BS> + HasOutput<U16Params, Output = ()>;
const fn u16_params_parser<BS: Readable>() -> U16ParamsT<BS> {
    Action(
        (DefaultInterp, DefaultInterp),
        |(v1, v2): (Option<u16>, Option<u16>)| {
            scroller("Got U16", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:?}")?))
        },
    )
}

pub type U32ParamsT<BS: Readable> =
    impl AsyncParser<U32Params, BS> + HasOutput<U32Params, Output = ()>;
const fn u32_params_parser<BS: Readable>() -> U32ParamsT<BS> {
    Action(
        (DefaultInterp, DefaultInterp),
        |(v1, v2): (Option<u32>, Option<u32>)| {
            scroller("Got U32", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:?}")?))
        },
    )
}

pub type U64ParamsT<BS: Readable> =
    impl AsyncParser<U64Params, BS> + HasOutput<U64Params, Output = ()>;
const fn u64_params_parser<BS: Readable>() -> U64ParamsT<BS> {
    Action(
        (DefaultInterp, DefaultInterp),
        |(v1, v2): (Option<u64>, Option<u64>)| {
            scroller("Got U64", |w| Ok(write!(w, "v1: {v1:?}, v2: {v2:?}")?))
        },
    )
}

pub type DArrayParamsT<BS: Readable> =
    impl AsyncParser<DArrayParams, BS> + HasOutput<DArrayParams, Output = ()>;
const fn darray_params_parser<BS: Readable>() -> DArrayParamsT<BS> {
    Action(
        (SubInterp(DefaultInterp), SubInterp(u32_params_parser())),
        |(v1, _v2): (Option<ArrayVec<u8, 24>>, Option<ArrayVec<(), 4>>)| {
            scroller("Got Darray", |w| Ok(write!(w, "v1: {v1:02x?}")?))
        },
    )
}

pub async fn test_parsers(io: HostIO) {
    let input = io.get_params::<1>().unwrap();
    test_parsers_parser().parse(&mut input[0].clone()).await;
    io.result_final(&[]).await;
}
