use arrayvec::{ArrayVec, ArrayString};
use core::fmt::Write;
use crate::crypto_helpers::{get_pubkey, get_private_key, detecdsa_sign, get_pkh, Hasher};
use crate::interface::*;
use ledger_parser_combinators::interp_parser::{InterpParser, DefaultInterp, SubInterp, ObserveBytes, DropInterp, Action};
use nanos_ui::ui;

pub type GetAddressImplT = Action<SubInterp<DefaultInterp>, fn(&ArrayVec<u32, 10>) -> Option<ArrayVec<u8, 260>>>;

pub const GET_ADDRESS_IMPL : GetAddressImplT
  = Action(
      SubInterp(DefaultInterp),
      | path : &ArrayVec<u32, 10> | {

          let key = get_pubkey(path).ok()?;
          let mut rv = ArrayVec::<u8, 260>::new();
          rv.try_extend_from_slice(&key.W[..]).ok()?;
          
          // At this point we have the value to send to the host; but there's a bit more to do to
          // ask permission from the user.

          let pkh = get_pkh(key);

          let mut pmpt = ArrayString::<128>::new();
          write!(pmpt, "{}", pkh).ok()?;

          if ! ui::MessageValidator::new(&["Provide Public Key", &pmpt], &[], &[]).ask() {
              None
          } else {
              Some(rv)
          }
      }
  );

pub type SignImplT = Action<
      (  Action<
           ObserveBytes<Hasher, fn(&mut Hasher, &[u8]), SubInterp<DropInterp>>,
           fn(&(Hasher, ArrayVec<(), { usize::MAX }>)) -> Option<[u8; 32]> >
      ,  Action<SubInterp<DefaultInterp>,
            fn(&ArrayVec<u32,10>) -> Option<nanos_sdk::bindings::cx_ecfp_private_key_t> >),
       fn(&([u8; 32], nanos_sdk::bindings::cx_ecfp_private_key_t)) -> Option<ArrayVec<u8, 260>>>;

pub const SIGN_IMPL : SignImplT = Action(
            (
            Action(
                // Calculate the hash of the transaction
                ObserveBytes( Hasher::new, Hasher::update, SubInterp(DropInterp)),

                // Ask the user if they accept the transaction body's hash
                | ( hash, _ ) : &( Hasher, ArrayVec<(), { usize::MAX } > ) | {

                    let the_hash = hash.clone().finalize();

                    let mut pmpt = ArrayString::<128>::new();
                    write!(pmpt, "{}", the_hash).ok()?;

                    if ! ui::MessageValidator::new(&["Sign Hash?", &pmpt], &[], &[]).ask() {
                        None
                    } else {
                        Some(the_hash.0.into())
                    }
                }
            ),
            Action(
                SubInterp(DefaultInterp),
                // And ask the user if this is the key the meant to sign with:
                | path : &ArrayVec<u32, 10> | {
                    let privkey = get_private_key(path).ok()?;
                    let pubkey = get_pubkey(path).ok()?; // Redoing work here; fix.
                    let pkh = get_pkh(pubkey);

                    let mut pmpt = ArrayString::<128>::new();
                    write!(pmpt, "{}", pkh).ok()?;

                    if ! ui::MessageValidator::new(&["With PKH", &pmpt], &[], &[]).ask() {
                        None
                    } else {
                        Some(privkey)
                    }
                }
            )),
            | (hash, key) : &([u8; 32], _) | {
                // By the time we get here, we've approved and just need to do the signature.
                let (sig, len) = detecdsa_sign(hash, key)?;
                let mut rv = ArrayVec::<u8, 260>::new();
                rv.try_extend_from_slice(&sig[0 .. len as usize]).ok()?;
                Some(rv)
            });



// The global parser state enum; any parser above that'll be used as the implementation for an APDU
// must have a field here.

pub enum ParsersState {
    NoState,
    GetAddressState(<GetAddressImplT as InterpParser<Bip32Key>>::State),
    SignState(<SignImplT as InterpParser<SignParameters>>::State)
}

pub fn get_get_address_state(s : &mut ParsersState) -> &mut <GetAddressImplT as InterpParser<Bip32Key>>::State {
    match s {
        ParsersState::GetAddressState(_) => { }
        _ => { *s = ParsersState::GetAddressState(<GetAddressImplT as InterpParser<Bip32Key>>::init(&GET_ADDRESS_IMPL)); }
    }
    match s {
        ParsersState::GetAddressState(ref mut a) => { a }
        _ => { panic!("") }
    }
}

pub fn get_sign_state(s : &mut ParsersState) -> &mut <SignImplT as InterpParser<SignParameters>>::State {
    match s {
        ParsersState::SignState(_) => { }
        _ => { *s = ParsersState::SignState(<SignImplT as InterpParser<SignParameters>>::init(&SIGN_IMPL)); }
    }
    match s {
        ParsersState::SignState(ref mut a) => { a }
        _ => { panic!("") }
    }
}
