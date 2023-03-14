use crate::interface::*;
use crate::test_parsers::*;
use crate::utils::*;
use alamgu_async_block::*;
use arrayvec::ArrayVec;
use core::fmt::Write;
use ledger_crypto_helpers::common::{try_option, Address};
use ledger_crypto_helpers::eddsa::{
    ed25519_public_key_bytes, eddsa_sign, with_public_keys, Ed25519RawPubKeyAddress,
};
use ledger_crypto_helpers::hasher::{Base64Hash, Blake2b, Hasher};
use ledger_log::trace;
use ledger_parser_combinators::async_parser::*;
use ledger_parser_combinators::core_parsers::*;
use ledger_parser_combinators::interp::*;
use ledger_prompts_ui::final_accept_prompt;

use core::convert::TryFrom;
use core::future::Future;
use core::ops::Deref;
use zeroize::Zeroizing;

#[allow(clippy::upper_case_acronyms)]
type PKH = Ed25519RawPubKeyAddress;

pub type BipParserImplT =
    impl AsyncParser<Bip32Key, ByteStream> + HasOutput<Bip32Key, Output = ArrayVec<u32, 10>>;
pub const BIP_PATH_PARSER: BipParserImplT = SubInterp(DefaultInterp);

pub async fn get_address_apdu(io: HostIO) {
    let input = io.get_params::<1>().unwrap();

    let path = BIP_PATH_PARSER.parse(&mut input[0].clone()).await;

    let mut rv = ArrayVec::<u8, 220>::new();

    if with_public_keys(&path, |key, pkh: &PKH| {
        try_option(|| -> Option<()> {
            // Should return the format that the chain customarily uses for public keys; for
            // ed25519 that's usually r | s with no prefix, which isn't quite our internal
            // representation.
            let key_bytes = ed25519_public_key_bytes(key);

            rv.try_push(u8::try_from(key_bytes.len()).ok()?).ok()?;
            rv.try_extend_from_slice(key_bytes).ok()?;

            // And we'll send the address along; in our case it happens to be the same as the
            // public key, but in general it's something computed from the public key.
            let binary_address = pkh.get_binary_address();
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

const fn hasher_parser(
) -> impl LengthDelimitedParser<Byte, ByteStream> + HasOutput<Byte, Output = (Blake2b, Option<()>)>
{
    ObserveBytes(Hasher::new, Hasher::update, DropInterp)
}

pub async fn sign_apdu(io: HostIO) {
    let mut input = io.get_params::<2>().unwrap();

    let length = usize::from_le_bytes(input[0].read().await);
    let mut txn = input[0].clone();

    let hash: Zeroizing<Base64Hash<32>> =
        hasher_parser().parse(&mut txn, length).await.0.finalize();

    if scroller("Transaction hash", |w| Ok(write!(w, "{}", hash.deref())?)).is_none() {
        reject::<()>().await;
    }

    let path = BIP_PATH_PARSER.parse(&mut input[1].clone()).await;

    if with_public_keys(&path, |_, pkh: &PKH| {
        try_option(|| -> Option<()> {
            scroller("Sign for Address", |w| Ok(write!(w, "{pkh}")?))?;
            final_accept_prompt(&["Sign Transaction?"])?;
            Some(())
        }())
    })
    .ok()
    .is_none()
    {
        reject::<()>().await;
    }

    // By the time we get here, we've approved and just need to do the signature.
    if let Some(sig) = { eddsa_sign(&path, &hash.deref().0).ok() } {
        io.result_final(&sig.0[0..]).await;
    } else {
        reject::<()>().await;
    }
}

pub type APDUsFuture = impl Future<Output = ()>;

#[inline(never)]
pub fn handle_apdu_async(io: HostIO, ins: Ins) -> APDUsFuture {
    trace!("Constructing future");
    async move {
        trace!("Dispatching");
        match ins {
            Ins::GetVersion => {
                const APP_NAME: &str = "alamgu example";
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
                NoinlineFut(sign_apdu(io)).await;
            }
            Ins::TestParsers => {
                NoinlineFut(test_parsers(io)).await;
            }
            Ins::GetVersionStr => {}
            Ins::Exit => nanos_sdk::exit_app(0),
        }
    }
}
