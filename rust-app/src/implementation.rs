use crate::interface::*;
use crate::test_parsers::*;
use crate::utils::*;
use arrayvec::ArrayVec;
use core::fmt::Write;
use ledger_crypto_helpers::common::{try_option, Address};
use ledger_crypto_helpers::eddsa::{
    ed25519_public_key_bytes, eddsa_sign, with_public_keys, Ed25519RawPubKeyAddress,
};
use ledger_crypto_helpers::hasher::{Base64Hash, Blake2b, Hasher};
use ledger_log::info;
use ledger_parser_combinators::interp_parser::{
    Action, DefaultInterp, DropInterp, InterpParser, MoveAction, ObserveBytes, ParserCommon,
    SubInterp,
};
use ledger_prompts_ui::final_accept_prompt;

use core::convert::TryFrom;
use core::ops::Deref;
use zeroize::Zeroizing;

#[allow(clippy::upper_case_acronyms)]
type PKH = Ed25519RawPubKeyAddress;

pub type GetAddressImplT = impl InterpParser<Bip32Key, Returning = ArrayVec<u8, 128>>;

pub const GET_ADDRESS_IMPL: GetAddressImplT = Action(
    SubInterp(DefaultInterp),
    mkfn(
        |path: &ArrayVec<u32, 10>, destination: &mut Option<ArrayVec<u8, 128>>| -> Option<()> {
            with_public_keys(path, |key: &_, pkh: &PKH| {
                try_option(|| -> Option<()> {
                    scroller("Provide Public Key", |w| {
                        Ok(write!(w, "For Address     {pkh}")?)
                    })?;

                    final_accept_prompt(&[])?;

                    let rv = destination.insert(ArrayVec::new());

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
        },
    ),
);

pub type SignImplT = impl InterpParser<SignParameters, Returning = ArrayVec<u8, 128>>;

pub static SIGN_IMPL: SignImplT = Action(
    (
        Action(
            // Calculate the hash of the transaction
            ObserveBytes(Hasher::new, Hasher::update, SubInterp(DropInterp)),
            // Ask the user if they accept the transaction body's hash
            mkfn(
                |(mut hasher, _): &(Blake2b, _),
                 destination: &mut Option<Zeroizing<Base64Hash<32>>>| {
                    let the_hash: Zeroizing<Base64Hash<32>> = hasher.finalize();
                    scroller("Transaction hash", |w| {
                        Ok(write!(w, "{}", &the_hash.deref())?)
                    })?;
                    *destination = Some(the_hash);
                    Some(())
                },
            ),
        ),
        MoveAction(
            SubInterp(DefaultInterp),
            // And ask the user if this is the key the meant to sign with:
            mkmvfn(
                |path: ArrayVec<u32, 10>, destination: &mut Option<ArrayVec<u32, 10>>| {
                    with_public_keys(&path, |_, pkh: &PKH| {
                        try_option(|| -> Option<()> {
                            scroller("Sign for Address", |w| Ok(write!(w, "{pkh}")?))?;
                            Some(())
                        }())
                    })
                    .ok()?;
                    *destination = Some(path);
                    Some(())
                },
            ),
        ),
    ),
    mkfn(
        |(hash, path): &(Option<Zeroizing<Base64Hash<32>>>, Option<ArrayVec<u32, 10>>),
         destination: &mut _| {
            final_accept_prompt(&["Sign Transaction?"])?;

            // By the time we get here, we've approved and just need to do the signature.
            let sig = eddsa_sign(path.as_ref()?, &hash.as_ref()?.0[..]).ok()?;
            let mut rv = ArrayVec::<u8, 128>::new();
            rv.try_extend_from_slice(&sig.0[..]).ok()?;
            *destination = Some(rv);
            Some(())
        },
    ),
);

// The global parser state enum; any parser above that'll be used as the implementation for an APDU
// must have a field here.
#[allow(clippy::large_enum_variant)]
pub enum ParsersState {
    NoState,
    GetAddressState(<GetAddressImplT as ParserCommon<Bip32Key>>::State),
    SignState(<SignImplT as ParserCommon<SignParameters>>::State),
    TestParsersState(<TestParsersImplT as ParserCommon<TestParsersSchema>>::State),
}

pub fn reset_parsers_state(state: &mut ParsersState) {
    *state = ParsersState::NoState;
}

#[inline(never)]
pub fn get_get_address_state(
    s: &mut ParsersState,
) -> &mut <GetAddressImplT as ParserCommon<Bip32Key>>::State {
    match s {
        ParsersState::GetAddressState(_) => {}
        _ => {
            info!("Non-same state found; initializing state.");
            *s = ParsersState::GetAddressState(<GetAddressImplT as ParserCommon<Bip32Key>>::init(
                &GET_ADDRESS_IMPL,
            ));
        }
    }
    match s {
        ParsersState::GetAddressState(ref mut a) => a,
        _ => {
            panic!("")
        }
    }
}

#[inline(never)]
pub fn get_sign_state(
    s: &mut ParsersState,
) -> &mut <SignImplT as ParserCommon<SignParameters>>::State {
    match s {
        ParsersState::SignState(_) => {}
        _ => {
            info!("Non-same state found; initializing state.");
            *s = ParsersState::SignState(<SignImplT as ParserCommon<SignParameters>>::init(
                &SIGN_IMPL,
            ));
        }
    }
    match s {
        ParsersState::SignState(ref mut a) => a,
        _ => {
            panic!("")
        }
    }
}

#[inline(never)]
pub fn get_test_parsers_state(
    s: &mut ParsersState,
) -> &mut <TestParsersImplT as ParserCommon<TestParsersSchema>>::State {
    match s {
        ParsersState::TestParsersState(_) => {}
        _ => {
            info!("Non-same state found; initializing state.");
            *s = ParsersState::TestParsersState(<TestParsersImplT as ParserCommon<
                TestParsersSchema,
            >>::init(&test_parsers_parser()));
        }
    }
    match s {
        ParsersState::TestParsersState(ref mut a) => a,
        _ => {
            panic!("")
        }
    }
}
