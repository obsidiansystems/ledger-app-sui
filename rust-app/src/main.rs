#![no_std]
#![no_main]

mod crypto_helpers;
mod utils;

use core::str::from_utf8;
use crypto_helpers::*;
use nanos_sdk::buttons::ButtonEvent;
use nanos_sdk::ecc::DerEncodedEcdsaSignature;
use nanos_sdk::io;
use nanos_sdk::io::SyscallError;
use nanos_ui::ui;

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

/// Display public key in two separate
/// message scrollers
fn show_pubkey() {
    let pubkey = get_pubkey();
    match pubkey {
        Ok(pk) => {
            {
                let hex0 = utils::to_hex(&pk.W[1..33]).unwrap();
                let m = from_utf8(&hex0).unwrap();
                ui::MessageScroller::new(m).event_loop();
            }
            {
                let hex1 = utils::to_hex(&pk.W[33..65]).unwrap();
                let m = from_utf8(&hex1).unwrap();
                ui::MessageScroller::new(m).event_loop();
            }
        }
        Err(_) => ui::popup("Error"),
    }
}

/// Basic nested menu. Will be subject
/// to simplifications in the future.
#[allow(clippy::needless_borrow)]
fn menu_example() {
    loop {
        match ui::Menu::new(&[&"PubKey", &"Infos", &"Back", &"Exit App"]).show() {
            0 => show_pubkey(),
            1 => loop {
                match ui::Menu::new(&[&"Copyright", &"Authors", &"Back"]).show() {
                    0 => ui::popup("2020 Ledger"),
                    1 => ui::popup("???"),
                    _ => break,
                }
            },
            2 => return,
            3 => nanos_sdk::exit_app(0),
            _ => (),
        }
    }
}

/// This is the UI flow for signing, composed of a scroller
/// to read the incoming message, a panel that requests user
/// validation, and an exit message.
fn sign_ui(message: &[u8]) -> Result<Option<DerEncodedEcdsaSignature>, SyscallError> {
    ui::popup("Message review");

    {
        let hex = utils::to_hex(message).map_err(|_| SyscallError::Overflow)?;
        let m = from_utf8(&hex).map_err(|_| SyscallError::InvalidParameter)?;

        ui::MessageScroller::new(m).event_loop();
    }

    if ui::Validator::new("Sign ?").ask() {
        let k = get_private_key()?;
        let (sig, _sig_len) = detecdsa_sign(message, &k).unwrap();
        ui::popup("Done !");
        Ok(Some(sig))
    } else {
        ui::popup("Cancelled");
        Ok(None)
    }
}

macro_rules! def_parsers {
    {$stateset_name:ident $parsers:ident $parser_tags:ident { $($name:ident = $parser:expr; )+}} =>
    {
        enum StateSet<$($name),+>{
            NoState,
            $($name($name)),+
        }
        fn state_init<$($name),+>($(_: $name),+) -> StateSet<$($name),+> { StateSet::NoState }

        type RX<'a, R> = Result<(R, &'a [u8]), (Option<OOB>, &'a [u8] )>;

        #[derive(Copy, Clone)]
        enum $parser_tags {
            Reset,
            $($name),+
        }

        use arrayvec::ArrayVec;

        fn $stateset_name() -> impl for<'a> FnMut($parser_tags, &'a [u8]) -> RX<'a, ArrayVec<u8, 260> > {
            $(#[allow(non_snake_case)] let $name = $parser;)+
            let mut state_enum = state_init($($name.init_method()),+);

            move |selector, chunk| {
                match selector {
                    $parser_tags::Reset => {
                        state_enum = StateSet::NoState;
                        Err((None, chunk))
                    }
                    $($parser_tags::$name => {
                        match state_enum {
                            StateSet::$name(_) => { }
                            _ => state_enum = StateSet::$name($name.init_method())
                        }
                        match state_enum {
                            StateSet::$name(ref mut a) => {
                                $name.parse(a, chunk)
                            }
                            _ => { panic!("Unreachable"); }
                        }
                    })+
                }
            }
        }
    }
}

// Fiddly; this one's basically just fmap rather than the more monadic-like Action.
// Relevant because it's inconvenient with current Action to return a non-copy item like ArrayVec.
mod fa {
    use ledger_parser_combinators::core_parsers::RV;
    use ledger_parser_combinators::forward_parser::{ForwardParser, OOB};
    type RX<'a, R> = Result<(R, &'a [u8]), (Option<OOB>, &'a [u8] )>;
    type RR<'a, I> = RX<'a, <I as RV>::R>;

    pub struct FinalAction<I : RV, O, F: Fn(&I::R) -> O> {
        pub sub: I,
        pub f: F
    }
    impl<I : RV, O, F: Fn(&I::R) -> O> RV for FinalAction<I,O,F> {
        type R = O;
    }
    impl<I : RV + ForwardParser, O, F: Fn(&I::R) -> O> ForwardParser for FinalAction<I, O, F> {
        type State = I::State;
        fn init() -> Self::State { I::init() }
        fn parse<'a, 'b>(&self, state: &'b mut Self::State, chunk: &'a [u8]) -> RR<'a, Self>{
            let (ret, new_chunk) = self.sub.parse(state, chunk)?;
            Ok(((self.f)(&ret), new_chunk))
        }
    }
}

use nanos_sdk::ecc::{CurvesId};

use ledger_parser_combinators::core_parsers::{U32, Byte, DArray};
use ledger_parser_combinators::forward_parser::{ForwardParser, OOB};
use ledger_parser_combinators::endianness::Endianness;

// Define the APDU-handling parsers; clustered together like this to allow us to infer a type for
// the big enum of global parser states.

def_parsers!{ mk_parsers Parsers ParserTag {
    GetAddressParser = fa::FinalAction {
        sub: DArray::<_,_,10>(Byte, U32::< { Endianness::Little } >),
        f: | path | {
            let mut raw_key = [0u8; 32];
            match nanos_sdk::ecc::bip32_derive(CurvesId::Secp256k1, &path[..], &mut raw_key) {
                Ok(_) => {
                    let mut rv = ArrayVec::new();
                    rv.copy_from_slice(&raw_key);
                    rv
                }
                Err(_) => { panic!("Need to be able to reject from here; fix Action so we can"); }
            }
        }
    };
} }

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = io::Comm::new();
    // let mut states = parser_states!();
    let mut parsers = mk_parsers();

    // with_parser_state!(parsers);

    loop {
        // Draw some 'welcome' screen
        ui::SingleMessage::new("W e l c o m e").show();

        // Wait for either a specific button push to exit the app
        // or an APDU command
        match comm.next_event() {
            io::Event::Button(ButtonEvent::RightButtonRelease) => nanos_sdk::exit_app(0),
            io::Event::Command(ins) => match handle_apdu(&mut comm, ins, &mut parsers) {
                Ok(()) => comm.reply_ok(),
                Err(sw) => comm.reply(sw),
            },
            _ => (),
        }
    }
}

#[repr(u8)]
enum Ins {
    GetPubkey,
    Sign,
    Menu,
    ShowPrivateKey,
    Exit,
}

impl From<u8> for Ins {
    fn from(ins: u8) -> Ins {
        match ins {
            2 => Ins::GetPubkey,
            3 => Ins::Sign,
            4 => Ins::Menu,
            0xfe => Ins::ShowPrivateKey,
            0xff => Ins::Exit,
            _ => panic!(),
        }
    }
}

use nanos_sdk::io::Reply;



fn handle_apdu<P: for<'a> FnMut(ParserTag, &'a [u8]) -> RX<'a, ArrayVec<u8, 260> > >(comm: &mut io::Comm, ins: Ins, mut parser: P) -> Result<(), Reply> {
    if comm.rx == 0 {
        return Err(io::StatusWords::NothingReceived.into());
    }

    // Could be made standalone.
    let mut run_parser_apdu = | tag | -> Result<(), Reply> {
        let mut cursor = comm.get_data()?;

        loop {
            match parser(tag, cursor) {
                Err((Some(OOB::Prompt(_prompt)), new_cursor)) => {
                    // TODO: Actually do something UI with the prompt here.
                    cursor=new_cursor;
                }
                Err((Some(OOB::Reject), _)) => { let _ = parser(ParserTag::Reset, cursor); break Ok(()) } // Rejection; reset the parser. Possibly send error message to host?
                // Deliberately no catch-all on the Err((Some case; we'll get error messages if we
                // add to OOB's out-of-band actions and forget to implement them.
                Err((None, [])) => { break Ok(()) } // Finished the chunk with no further actions pending
                Err((None, _)) => { let _ = parser(ParserTag::Reset, cursor); break Ok(()) } // Finished the parse incorrectly; reset and error message.
                Ok((rv, [])) => {
                    comm.append(&rv[..]);
                    break Ok(())
                } // Finished the chunk and the parse.
                Ok((_, _)) => { let _ = parser(ParserTag::Reset, cursor); break Ok(()) } // Parse ended before the chunk did; reset.
            }
        }
    };

    match ins {
        Ins::GetPubkey => { run_parser_apdu(ParserTag::GetAddressParser)? }
        //{ parser(ParserTag::get_address_parser, comm.get_data()?); } // handle_pubkey_apdu(comm.get_data()?), // comm.append(&get_pubkey()?.W),
        Ins::Sign => {
            let out = sign_ui(comm.get_data()?)?;
            if let Some(o) = out {
                comm.append(&o)
            }
        }
        Ins::Menu => menu_example(),
        Ins::ShowPrivateKey => comm.append(&bip32_derive_secp256k1(&BIP32_PATH)?),
        Ins::Exit => nanos_sdk::exit_app(0),
    }
    Ok(())
}
