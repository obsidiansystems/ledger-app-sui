use crate::crypto_helpers::{detecdsa_sign, get_pkh, get_private_key, get_pubkey, Hasher};
use crate::interface::*;
use crate::*;
use arrayvec::ArrayVec;
use core::fmt::Write;
use ledger_log::{info};
use ledger_parser_combinators::interp_parser::{
    Action, DefaultInterp, DropInterp, ParserCommon, InterpParser, ObserveBytes, SubInterp,
};
use crate::ui::write_scroller;

use core::convert::TryFrom;

// A couple type ascription functions to help the compiler along.
const fn mkfn<A,B,C>(q: fn(&A,&mut B)->C) -> fn(&A,&mut B)->C {
  q
}

pub type GetAddressImplT = impl InterpParser<Bip32Key, Returning = ArrayVec<u8, 260_usize>>;

pub static GET_ADDRESS_IMPL: GetAddressImplT =
    Action(SubInterp(DefaultInterp), mkfn(|path: &ArrayVec<u32, 10>, destination: &mut Option<ArrayVec<u8, 260>>| {
        let key = get_pubkey(path).ok()?;

        let pkh = get_pkh(key);

        // At this point we have the value to send to the host; but there's a bit more to do to
        // ask permission from the user.
        write_scroller("Provide Public Key", |w| Ok(write!(w, "{}", pkh)?))?;

        *destination=Some(ArrayVec::new());
        let p = destination.as_mut()?;
        p.try_push(u8::try_from(key.W_len).ok()?).ok()?;
        p.try_extend_from_slice(&key.W[1..key.W_len as usize]).ok()?;
        Some(())
    }));

pub type SignImplT = impl InterpParser<SignParameters, Returning = ArrayVec<u8, 260_usize>>;

pub static SIGN_IMPL: SignImplT = Action(
    (
        Action(
            // Calculate the hash of the transaction
            ObserveBytes(Hasher::new, Hasher::update, SubInterp(DropInterp)),
            // Ask the user if they accept the transaction body's hash
            mkfn(|(hash, _): &(Hasher, Option<ArrayVec<(), { usize::MAX }>>), destination: &mut _| {
                let the_hash = hash.clone().finalize();

                write_scroller("Sign Hash?", |w| Ok(write!(w, "{}", the_hash)?))?;

                *destination = Some(the_hash.0.into());
                Some(())
            }),
        ),
        Action(
            SubInterp(DefaultInterp),
            // And ask the user if this is the key the meant to sign with:
            mkfn(|path: &ArrayVec<u32, 10>, destination: &mut _| {
                let privkey = get_private_key(path).ok()?;
                let pubkey = get_pubkey(path).ok()?; // Redoing work here; fix.
                let pkh = get_pkh(pubkey);

                write_scroller("With PKH", |w| Ok(write!(w, "{}", pkh)?))?;

                *destination = Some(privkey);
                Some(())
            }),
        ),
    ),
    mkfn(|(hash, key): &(Option<[u8; 32]>, Option<_>), destination: &mut _| {
        // By the time we get here, we've approved and just need to do the signature.
        let (sig, len) = detecdsa_sign(hash.as_ref()?, key.as_ref()?)?;
        let mut rv = ArrayVec::<u8, 260>::new();
        rv.try_extend_from_slice(&sig[0..len as usize]).ok()?;
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
