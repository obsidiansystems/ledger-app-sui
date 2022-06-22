use crate::interface::*;
use arrayvec::ArrayVec;
use core::fmt::Write;
use ledger_crypto_helpers::hasher::{Hash, Hasher, Blake2b};
use ledger_crypto_helpers::common::{with_public_keys, PKH, public_key_bytes};
use ledger_crypto_helpers::eddsa::eddsa_sign;
use ledger_log::{info};
use ledger_parser_combinators::interp_parser::{
    Action, DefaultInterp, DropInterp, ParserCommon, MoveAction, InterpParser, ObserveBytes, SubInterp,
};
use ledger_prompts_ui::{write_scroller, final_accept_prompt};

use core::convert::TryFrom;
use zeroize::{Zeroizing};
use core::ops::Deref;

// A couple type ascription functions to help the compiler along.
const fn mkfn<A,B,C>(q: fn(&A,&mut B)->C) -> fn(&A,&mut B)->C {
  q
}
const fn mkmvfn<A,B,C>(q: fn(A,&mut B)->Option<C>) -> fn(A,&mut B)->Option<C> {
    q
}
const fn mkvfn<A>(q: fn(&A,&mut Option<()>)->Option<()>) -> fn(&A,&mut Option<()>)->Option<()> {
    q
}

pub type GetAddressImplT = impl InterpParser<Bip32Key, Returning = ArrayVec<u8, 128>>;

pub const GET_ADDRESS_IMPL: GetAddressImplT =
    Action(SubInterp(DefaultInterp), mkfn(|path: &ArrayVec<u32, 10>, destination: &mut Option<ArrayVec<u8, 128>>| -> Option<()> {
        with_public_keys(path, |key: &_, pkh: &PKH| {

            write_scroller("Provide Public Key", |w| Ok(write!(w, "For Address     {}", pkh)?))?;

            final_accept_prompt(&[])?;

            let key_bytes = public_key_bytes(key);
            let rv = destination.insert(ArrayVec::new());
            rv.try_push(u8::try_from(key_bytes.len()).ok()?).ok()?;
            rv.try_extend_from_slice(key_bytes).ok()?;
            rv.try_push(u8::try_from(pkh.0.len()).ok()?).ok()?;
            rv.try_extend_from_slice(&pkh.0).ok()?;
            Ok(())
        }).ok()
    }));

pub type SignImplT = impl InterpParser<SignParameters, Returning = ArrayVec<u8, 128>>;

pub static SIGN_IMPL: SignImplT = Action(
    (
        Action(
            // Calculate the hash of the transaction
            ObserveBytes(Hasher::new, Hasher::update, SubInterp(DropInterp)),
            // Ask the user if they accept the transaction body's hash
            mkfn(|(mut hasher, _): &(Blake2b, _), destination: &mut Option<Zeroizing<Hash<32>>>| {
                let the_hash = hasher.finalize();
                write_scroller("Transaction hash", |w| Ok(write!(w, "{}", the_hash.deref())?))?;
                *destination=Some(the_hash);
                Some(())
            }),
        ),
        MoveAction(
            SubInterp(DefaultInterp),
            // And ask the user if this is the key the meant to sign with:
            mkmvfn(|path: ArrayVec<u32, 10>, destination: &mut Option<ArrayVec<u32, 10>>| {
                with_public_keys(&path, |_, pkh: &PKH| {
                    write_scroller("Sign for Address", |w| Ok(write!(w, "{}", pkh)?))?;
                    Ok(())
                }).ok();
                *destination = Some(path);
                Some(())
            }),
        ),
    ),
    mkfn(|(hash, path): &(Option<Zeroizing<Hash<32>>>, Option<ArrayVec<u32, 10>>), destination: &mut _| {
        final_accept_prompt(&[&"Sign Transaction?"])?;

        // By the time we get here, we've approved and just need to do the signature.
        let sig = eddsa_sign(path.as_ref()?, &hash.as_ref()?.0[..])?;
        let mut rv = ArrayVec::<u8, 128>::new();
        rv.try_extend_from_slice(&sig.0[..]).ok()?;
        *destination = Some(rv);
        Some(())
    }),
);

// The global parser state enum; any parser above that'll be used as the implementation for an APDU
// must have a field here.

pub enum ParsersState {
    NoState,
    GetAddressState(<GetAddressImplT as ParserCommon<Bip32Key>>::State),
    SignState(<SignImplT as ParserCommon<SignParameters>>::State),
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
