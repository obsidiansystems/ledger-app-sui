use crate::interface::*;
use crate::settings::*;
use crate::utils::*;
use alamgu_async_block::*;
use arrayvec::ArrayString;
use arrayvec::ArrayVec;
use core::fmt::Write;
use ledger_crypto_helpers::common::{try_option, Address, HexSlice};
use ledger_crypto_helpers::eddsa::{ed25519_public_key_bytes, eddsa_sign, with_public_keys};
use ledger_crypto_helpers::hasher::{Blake2b, Hasher, HexHash};
use ledger_log::trace;
use ledger_parser_combinators::async_parser::*;
use ledger_parser_combinators::bcs::async_parser::*;
use ledger_parser_combinators::interp::*;
use ledger_prompts_ui::{final_accept_prompt, ScrollerError};
use nanos_sdk::io::SyscallError;

use core::convert::TryFrom;
use core::future::Future;

type SuiAddressRaw = [u8; SUI_ADDRESS_LENGTH];

pub struct SuiPubKeyAddress(nanos_sdk::ecc::ECPublicKey<65, 'E'>, SuiAddressRaw);

impl Address<SuiPubKeyAddress, nanos_sdk::ecc::ECPublicKey<65, 'E'>> for SuiPubKeyAddress {
    fn get_address(key: &nanos_sdk::ecc::ECPublicKey<65, 'E'>) -> Result<Self, SyscallError> {
        let key_bytes = ed25519_public_key_bytes(key);
        let mut tmp = ArrayVec::<u8, 33>::new();
        let _ = tmp.try_push(0); // SIGNATURE_SCHEME_TO_FLAG['ED25519']
        let _ = tmp.try_extend_from_slice(key_bytes);
        let mut hasher: Blake2b = Hasher::new();
        hasher.update(&tmp);
        let hash: [u8; SUI_ADDRESS_LENGTH] = hasher.finalize();
        Ok(SuiPubKeyAddress(key.clone(), hash))
    }
    fn get_binary_address(&self) -> &[u8] {
        &self.1
    }
}

impl core::fmt::Display for SuiPubKeyAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{}", HexSlice(&self.1))
    }
}

pub type BipParserImplT =
    impl AsyncParser<Bip32Key, ByteStream> + HasOutput<Bip32Key, Output = ArrayVec<u32, 10>>;
pub const BIP_PATH_PARSER: BipParserImplT = SubInterp(DefaultInterp);

pub async fn get_address_apdu(io: HostIO) {
    let input = io.get_params::<1>().unwrap();

    let path = BIP_PATH_PARSER.parse(&mut input[0].clone()).await;

    let mut rv = ArrayVec::<u8, 220>::new();

    if with_public_keys(&path, true, |key, address: &SuiPubKeyAddress| {
        try_option(|| -> Option<()> {
            let key_bytes = ed25519_public_key_bytes(key);

            rv.try_push(u8::try_from(key_bytes.len()).ok()?).ok()?;
            rv.try_extend_from_slice(key_bytes).ok()?;

            // And we'll send the address along;
            let binary_address = address.get_binary_address();
            rv.try_push(u8::try_from(binary_address.len()).ok()?).ok()?;
            rv.try_extend_from_slice(binary_address).ok()?;
            Some(())
        }())
    })
    .ok()
    .is_none()
    {
        reject::<()>().await;
    }

    io.result_final(&rv).await;
}

pub enum CallArg {
    RecipientAddress(SuiAddressRaw),
    Amount(u64),
    OtherPure,
    ObjectArg,
}

impl HasOutput<CallArgSchema> for DefaultInterp {
    type Output = CallArg;
}

impl<BS: Clone + Readable> AsyncParser<CallArgSchema, BS> for DefaultInterp {
    type State<'c> = impl Future<Output = Self::Output> + 'c where BS: 'c;
    fn parse<'a: 'c, 'b: 'c, 'c>(&'b self, input: &'a mut BS) -> Self::State<'c> {
        async move {
            let enum_variant =
                <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;
            match enum_variant {
                0 => {
                    let length =
                        <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input)
                            .await;
                    trace!("CallArgSchema: Pure: length: {}", length);
                    match length {
                        8 => CallArg::Amount(
                            <DefaultInterp as AsyncParser<Amount, BS>>::parse(
                                &DefaultInterp,
                                input,
                            )
                            .await,
                        ),
                        32 => CallArg::RecipientAddress(
                            <DefaultInterp as AsyncParser<Recipient, BS>>::parse(
                                &DefaultInterp,
                                input,
                            )
                            .await,
                        ),
                        _ => {
                            for _ in 0..length {
                                let _: [u8; 1] = input.read().await;
                            }
                            CallArg::OtherPure
                        }
                    }
                }
                1 => {
                    let enum_variant =
                        <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input)
                            .await;
                    match enum_variant {
                        0 => {
                            trace!("CallArgSchema: ObjectArg: ImmOrOwnedObject");
                            object_ref_parser().parse(input).await;
                        }
                        1 => {
                            trace!("CallArgSchema: ObjectArg: SharedObject");
                            <(DefaultInterp, DefaultInterp, DefaultInterp) as AsyncParser<
                                SharedObject,
                                BS,
                            >>::parse(
                                &(DefaultInterp, DefaultInterp, DefaultInterp), input
                            )
                            .await;
                        }
                        _ => reject_on(core::file!(), core::line!()).await,
                    }
                    CallArg::ObjectArg
                }
                _ => {
                    trace!("CallArgSchema: Unknown enum: {}", enum_variant);
                    reject_on(core::file!(), core::line!()).await
                }
            }
        }
    }
}

pub const TRANSFER_OBJECT_ARRAY_LENGTH: usize = 1;
pub const SPLIT_COIN_ARRAY_LENGTH: usize = 8;

pub enum Command {
    TransferObject(ArrayVec<Argument, TRANSFER_OBJECT_ARRAY_LENGTH>, Argument),
    SplitCoins(Argument, ArrayVec<Argument, SPLIT_COIN_ARRAY_LENGTH>),
}

impl HasOutput<CommandSchema> for DefaultInterp {
    type Output = Command;
}

impl<BS: Clone + Readable> AsyncParser<CommandSchema, BS> for DefaultInterp {
    type State<'c> = impl Future<Output = Self::Output> + 'c where BS: 'c;
    fn parse<'a: 'c, 'b: 'c, 'c>(&'b self, input: &'a mut BS) -> Self::State<'c> {
        async move {
            let enum_variant =
                <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;
            match enum_variant {
                1 => {
                    trace!("CommandSchema: TransferObject");
                    let v1 = <SubInterp<DefaultInterp> as AsyncParser<
                        Vec<ArgumentSchema, TRANSFER_OBJECT_ARRAY_LENGTH>,
                        BS,
                    >>::parse(&SubInterp(DefaultInterp), input)
                    .await;
                    let v2 = <DefaultInterp as AsyncParser<ArgumentSchema, BS>>::parse(
                        &DefaultInterp,
                        input,
                    )
                    .await;
                    Command::TransferObject(v1, v2)
                }
                2 => {
                    trace!("CommandSchema: SplitCoins");
                    let v1 = <DefaultInterp as AsyncParser<ArgumentSchema, BS>>::parse(
                        &DefaultInterp,
                        input,
                    )
                    .await;
                    let v2 = <SubInterp<DefaultInterp> as AsyncParser<
                        Vec<ArgumentSchema, SPLIT_COIN_ARRAY_LENGTH>,
                        BS,
                    >>::parse(&SubInterp(DefaultInterp), input)
                    .await;
                    Command::SplitCoins(v1, v2)
                }
                _ => {
                    trace!("CommandSchema: Unknown enum: {}", enum_variant);
                    reject_on(core::file!(), core::line!()).await
                }
            }
        }
    }
}

pub enum Argument {
    GasCoin,
    Input(u16),
    Result(u16),
    NestedResult(u16, u16),
}

impl HasOutput<ArgumentSchema> for DefaultInterp {
    type Output = Argument;
}

impl<BS: Clone + Readable> AsyncParser<ArgumentSchema, BS> for DefaultInterp {
    type State<'c> = impl Future<Output = Self::Output> + 'c where BS: 'c;
    fn parse<'a: 'c, 'b: 'c, 'c>(&'b self, input: &'a mut BS) -> Self::State<'c> {
        async move {
            let enum_variant =
                <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;
            match enum_variant {
                0 => {
                    trace!("ArgumentSchema: GasCoin");
                    Argument::GasCoin
                }
                1 => {
                    trace!("ArgumentSchema: Input");
                    Argument::Input(
                        <DefaultInterp as AsyncParser<U16LE, BS>>::parse(&DefaultInterp, input)
                            .await,
                    )
                }
                2 => {
                    trace!("ArgumentSchema: Result");
                    Argument::Result(
                        <DefaultInterp as AsyncParser<U16LE, BS>>::parse(&DefaultInterp, input)
                            .await,
                    )
                }
                3 => {
                    trace!("ArgumentSchema: NestedResult");
                    Argument::NestedResult(
                        <DefaultInterp as AsyncParser<U16LE, BS>>::parse(&DefaultInterp, input)
                            .await,
                        <DefaultInterp as AsyncParser<U16LE, BS>>::parse(&DefaultInterp, input)
                            .await,
                    )
                }
                _ => reject_on(core::file!(), core::line!()).await,
            }
        }
    }
}

impl<const PROMPT: bool> HasOutput<ProgrammableTransaction<PROMPT>>
    for ProgrammableTransaction<PROMPT>
{
    type Output = ();
}

impl<BS: Clone + Readable, const PROMPT: bool> AsyncParser<ProgrammableTransaction<PROMPT>, BS>
    for ProgrammableTransaction<PROMPT>
{
    type State<'c> = impl Future<Output = Self::Output> + 'c where BS: 'c;
    fn parse<'a: 'c, 'b: 'c, 'c>(&'b self, input: &'a mut BS) -> Self::State<'c> {
        async move {
            let mut recipient = None;
            let mut recipient_index = None;
            let mut amounts: ArrayVec<(u64, u32), SPLIT_COIN_ARRAY_LENGTH> = ArrayVec::new();

            // Handle inputs
            {
                let length =
                    <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;

                trace!("ProgrammableTransaction: Inputs: {}", length);
                for i in 0..length {
                    let arg = <DefaultInterp as AsyncParser<CallArgSchema, BS>>::parse(
                        &DefaultInterp,
                        input,
                    )
                    .await;
                    match arg {
                        CallArg::RecipientAddress(addr) => match recipient {
                            None => {
                                recipient = Some(addr);
                                recipient_index = Some(i);
                            }
                            // Reject on multiple RecipientAddress(s)
                            _ => reject_on(core::file!(), core::line!()).await,
                        },
                        CallArg::Amount(amt) =>
                        {
                            #[allow(clippy::single_match)]
                            match amounts.try_push((amt, i)) {
                                Err(_) => reject_on(core::file!(), core::line!()).await,
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }

            if recipient_index == None || amounts.is_empty() {
                reject_on::<()>(core::file!(), core::line!()).await;
            }

            let mut verified_recipient = false;
            let mut total_amount: u64 = 0;
            // Handle commands
            {
                let length =
                    <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;
                trace!("ProgrammableTransaction: Commands: {}", length);
                for _ in 0..length {
                    let c = <DefaultInterp as AsyncParser<CommandSchema, BS>>::parse(
                        &DefaultInterp,
                        input,
                    )
                    .await;
                    match c {
                        Command::TransferObject(_nested_results, recipient_input) => {
                            if verified_recipient {
                                // Reject more than one TransferObject(s)
                                reject_on::<()>(core::file!(), core::line!()).await;
                            }
                            match recipient_input {
                                Argument::Input(inp_index) => {
                                    if Some(inp_index as u32) != recipient_index {
                                        trace!("TransferObject recipient mismatch");
                                        reject_on::<()>(core::file!(), core::line!()).await;
                                    }
                                    verified_recipient = true;
                                }
                                _ => reject_on(core::file!(), core::line!()).await,
                            }
                        }
                        Command::SplitCoins(coin, input_indices) => {
                            match coin {
                                Argument::GasCoin => {}
                                _ => reject_on(core::file!(), core::line!()).await,
                            }
                            for arg in &input_indices {
                                match arg {
                                    Argument::Input(inp_index) => {
                                        for (amt, ix) in &amounts {
                                            if *ix == (*inp_index as u32) {
                                                match total_amount.checked_add(*amt) {
                                                    Some(t) => total_amount = t,
                                                    None => {
                                                        reject_on(core::file!(), core::line!())
                                                            .await
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => reject_on(core::file!(), core::line!()).await,
                                }
                            }
                        }
                    }
                }
            }

            if !verified_recipient {
                reject_on::<()>(core::file!(), core::line!()).await;
            }

            if PROMPT
                && Option::<()>::is_none(
                    &try {
                        scroller_paginated("To", |w| {
                            Ok(write!(
                                w,
                                "0x{}",
                                HexSlice(&recipient.ok_or(ScrollerError)?)
                            )?)
                        })?;

                        let (quotient, remainder_str) = get_amount_in_decimals(total_amount);
                        scroller_paginated("Amount", |w| {
                            Ok(write!(w, "SUI {quotient}.{}", remainder_str.as_str())?)
                        })?;
                    },
                )
            {
                reject::<()>().await;
            }
        }
    }
}

impl<const PROMPT: bool> HasOutput<TransactionKind<PROMPT>> for TransactionKind<PROMPT> {
    type Output = ();
}

impl<BS: Clone + Readable, const PROMPT: bool> AsyncParser<TransactionKind<PROMPT>, BS>
    for TransactionKind<PROMPT>
{
    type State<'c> = impl Future<Output = Self::Output> + 'c where BS: 'c;
    fn parse<'a: 'c, 'b: 'c, 'c>(&'b self, input: &'a mut BS) -> Self::State<'c> {
        async move {
            let enum_variant =
                <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;
            match enum_variant {
                0 => {
                    trace!("TransactionKind: ProgrammableTransaction");
                    <ProgrammableTransaction<PROMPT> as AsyncParser<
                        ProgrammableTransaction<PROMPT>,
                        BS,
                    >>::parse(&ProgrammableTransaction::<PROMPT>, input)
                    .await;
                }
                _ => {
                    trace!("TransactionKind: {}", enum_variant);
                    reject_on(core::file!(), core::line!()).await
                }
            }
        }
    }
}

fn get_amount_in_decimals(amount: u64) -> (u64, ArrayString<12>) {
    let factor_pow = 9;
    let factor = u64::pow(10, factor_pow);
    let quotient = amount / factor;
    let remainder = amount % factor;
    let mut remainder_str: ArrayString<12> = ArrayString::new();
    {
        // Make a string for the remainder, containing at lease one zero
        // So 1 SUI will be displayed as "1.0"
        let mut rem = remainder;
        for i in 0..factor_pow {
            let f = u64::pow(10, factor_pow - i - 1);
            let r = rem / f;
            let _ = remainder_str.try_push(char::from(b'0' + r as u8));
            rem %= f;
            if rem == 0 {
                break;
            }
        }
    }
    (quotient, remainder_str)
}

impl HasOutput<TransactionExpiration> for DefaultInterp {
    type Output = ();
}

impl<BS: Clone + Readable> AsyncParser<TransactionExpiration, BS> for DefaultInterp {
    type State<'c> = impl Future<Output = Self::Output> + 'c where BS: 'c;
    fn parse<'a: 'c, 'b: 'c, 'c>(&'b self, input: &'a mut BS) -> Self::State<'c> {
        async move {
            let enum_variant =
                <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;
            match enum_variant {
                0 => {
                    trace!("TransactionExpiration: None");
                }
                1 => {
                    trace!("TransactionExpiration: Epoch");
                    <DefaultInterp as AsyncParser<EpochId, BS>>::parse(&DefaultInterp, input).await;
                }
                _ => reject_on(core::file!(), core::line!()).await,
            }
        }
    }
}

const fn gas_data_parser<BS: Clone + Readable, const PROMPT: bool>(
) -> impl AsyncParser<GasData<PROMPT>, BS> + HasOutput<GasData<PROMPT>, Output = ()> {
    Action(
        (
            SubInterp(object_ref_parser()),
            DefaultInterp,
            DefaultInterp,
            DefaultInterp,
        ),
        |(_, _sender, _gas_price, gas_budget): (_, _, u64, u64)| {
            // Gas price is per gas amount. Gas budget is total, reflecting the amount of gas *
            // gas price. We only care about the total, not the price or amount in isolation , so we
            // just ignore that field.
            //
            // C.F. https://github.com/MystenLabs/sui/pull/8676
            if PROMPT {
                let (quotient, remainder_str) = get_amount_in_decimals(gas_budget);
                scroller("Max Gas", |w| {
                    Ok(write!(w, "SUI {}.{}", quotient, remainder_str.as_str())?)
                })?
            }
            Some(())
        },
    )
}

const fn object_ref_parser<BS: Readable>(
) -> impl AsyncParser<ObjectRef, BS> + HasOutput<ObjectRef, Output = ()> {
    Action((DefaultInterp, DefaultInterp, DefaultInterp), |_| Some(()))
}

const fn intent_parser<BS: Readable>(
) -> impl AsyncParser<Intent, BS> + HasOutput<Intent, Output = ()> {
    Action((DefaultInterp, DefaultInterp, DefaultInterp), |_| {
        trace!("Intent Ok");
        Some(())
    })
}

const fn transaction_data_v1_parser<BS: Clone + Readable, const PROMPT: bool>(
) -> impl AsyncParser<TransactionDataV1<PROMPT>, BS> + HasOutput<TransactionDataV1<PROMPT>, Output = ()>
{
    Action(
        (
            TransactionKind::<PROMPT>,
            DefaultInterp,
            gas_data_parser::<_, PROMPT>(),
            DefaultInterp,
        ),
        |_| Some(()),
    )
}

impl<const PROMPT: bool> HasOutput<TransactionData<PROMPT>> for TransactionData<PROMPT> {
    type Output = ();
}

impl<BS: Clone + Readable, const PROMPT: bool> AsyncParser<TransactionData<PROMPT>, BS>
    for TransactionData<PROMPT>
{
    type State<'c> = impl Future<Output = Self::Output> + 'c where BS: 'c;
    fn parse<'a: 'c, 'b: 'c, 'c>(&'b self, input: &'a mut BS) -> Self::State<'c> {
        async move {
            let enum_variant =
                <DefaultInterp as AsyncParser<ULEB128, BS>>::parse(&DefaultInterp, input).await;
            match enum_variant {
                0 => {
                    trace!("TransactionData: V1");
                    transaction_data_v1_parser::<_, PROMPT>().parse(input).await;
                }
                _ => reject_on(core::file!(), core::line!()).await,
            }
        }
    }
}

const fn tx_parser<BS: Clone + Readable, const PROMPT: bool>(
) -> impl AsyncParser<IntentMessage<PROMPT>, BS> + HasOutput<IntentMessage<PROMPT>, Output = ()> {
    Action((intent_parser(), TransactionData::<PROMPT>), |_| Some(()))
}

pub async fn sign_apdu(io: HostIO, settings: Settings) {
    let mut input = io.get_params::<2>().unwrap();

    // Read length, and move input[0] by one byte
    let length = usize::from_le_bytes(input[0].read().await);

    let known_txn = {
        let mut txn = input[0].clone();
        NoinlineFut(async move {
            trace!("Beginning check parse");
            TryFuture(tx_parser::<_, false>().parse(&mut txn))
                .await
                .is_some()
        })
        .await
    };

    if known_txn {
        if scroller("Transfer", |w| Ok(write!(w, "SUI")?)).is_none() {
            reject::<()>().await;
        };
        {
            let mut bs = input[1].clone();
            NoinlineFut(async move {
                let path = BIP_PATH_PARSER.parse(&mut bs).await;
                if with_public_keys(&path, true, |_, address: &SuiPubKeyAddress| {
                    try_option(|| -> Option<()> {
                        scroller_paginated("From", |w| Ok(write!(w, "{address}")?))?;
                        Some(())
                    }())
                })
                .ok()
                .is_none()
                {
                    reject::<()>().await;
                }
            })
            .await
        };

        {
            let mut txn = input[0].clone();
            NoinlineFut(async move {
                trace!("Beginning parse");
                tx_parser::<_, true>().parse(&mut txn).await;
            })
            .await
        };

        if final_accept_prompt(&["Sign Transaction?"]).is_none() {
            reject::<()>().await;
        };
    } else if settings.get() == 0 {
        scroller("WARNING", |w| {
            Ok(write!(
                w,
                "Transaction not recognized, enable blind signing to sign unknown transactions"
            )?)
        });
        reject::<()>().await;
    } else if scroller("WARNING", |w| Ok(write!(w, "Transaction not recognized")?)).is_none() {
        reject::<()>().await;
    }

    // By the time we get here, we've approved and just need to do the signature.
    NoinlineFut((|input: ArrayVec<ByteStream, 2>| async move {
        let mut hasher: Blake2b = Hasher::new();
        {
            let mut txn = input[0].clone();
            const CHUNK_SIZE: usize = 128;
            let (chunks, rem) = (length / CHUNK_SIZE, length % CHUNK_SIZE);
            for _ in 0..chunks {
                let b: [u8; CHUNK_SIZE] = txn.read().await;
                hasher.update(&b);
            }
            for _ in 0..rem {
                let b: [u8; 1] = txn.read().await;
                hasher.update(&b);
            }
        }
        let hash: HexHash<32> = hasher.finalize();
        if !known_txn {
            if scroller("Transaction Hash", |w| Ok(write!(w, "0x{hash}")?)).is_none() {
                reject::<()>().await;
            };
            if final_accept_prompt(&["Blind Sign Transaction?"]).is_none() {
                reject::<()>().await;
            };
        }
        let path = BIP_PATH_PARSER.parse(&mut input[1].clone()).await;
        if let Some(sig) = { eddsa_sign(&path, true, &hash.0).ok() } {
            io.result_final(&sig.0[0..]).await;
        } else {
            reject::<()>().await;
        }
    })(input))
    .await
}

pub type APDUsFuture = impl Future<Output = ()>;

#[inline(never)]
pub fn handle_apdu_async(io: HostIO, ins: Ins, settings: Settings) -> APDUsFuture {
    trace!("Constructing future");
    async move {
        trace!("Dispatching");
        match ins {
            Ins::GetVersion => {
                const APP_NAME: &str = "sui";
                let mut rv = ArrayVec::<u8, 220>::new();
                let _ = rv.try_push(env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap());
                let _ = rv.try_push(env!("CARGO_PKG_VERSION_MINOR").parse().unwrap());
                let _ = rv.try_push(env!("CARGO_PKG_VERSION_PATCH").parse().unwrap());
                let _ = rv.try_extend_from_slice(APP_NAME.as_bytes());
                io.result_final(&rv).await;
            }
            Ins::GetPubkey => {
                NoinlineFut(get_address_apdu(io)).await;
            }
            Ins::Sign => {
                trace!("Handling sign");
                NoinlineFut(sign_apdu(io, settings)).await;
            }
            Ins::GetVersionStr => {}
            Ins::Exit => nanos_sdk::exit_app(0),
        }
    }
}
