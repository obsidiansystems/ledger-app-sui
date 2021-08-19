#![cfg_attr(target_os="nanos", no_std)]
#![cfg_attr(target_os="nanos", no_main)]

#[cfg(not(target_os="nanos"))]
fn main() {
}

#[cfg(target_os="nanos")]
mod crypto_helpers;
#[cfg(target_os="nanos")]
use crypto_helpers::*;
#[cfg(target_os="nanos")]
mod utils;

use core::str::from_utf8;
#[cfg(target_os="nanos")]
use nanos_sdk::buttons::ButtonEvent;
#[cfg(target_os="nanos")]
use nanos_sdk::ecc::DerEncodedEcdsaSignature;
#[cfg(target_os="nanos")]
use nanos_sdk::io;
#[cfg(target_os="nanos")]
use nanos_sdk::io::SyscallError;
#[cfg(target_os="nanos")]
use nanos_ui::ui;

#[cfg(target_os="nanos")]
nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

/// Display public key in two separate
/// message scrollers
#[cfg(target_os="nanos")]
fn show_pubkey() {
    let pubkey = get_pubkey(&BIP32_PATH);
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
#[cfg(target_os="nanos")]
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
            3 => nanos_sdk::exit_app(2),
            _ => (),
        }
    }
}

/// This is the UI flow for signing, composed of a scroller
/// to read the incoming message, a panel that requests user
/// validation, and an exit message.
#[cfg(target_os="nanos")]
fn sign_ui(message: &[u8]) -> Result<Option<DerEncodedEcdsaSignature>, SyscallError> {
    ui::popup("Message review");

    {
        let hex = utils::to_hex(message).map_err(|_| SyscallError::Overflow)?;
        let m = from_utf8(&hex).map_err(|_| SyscallError::InvalidParameter)?;

        ui::MessageScroller::new(m).event_loop();
    }

    if ui::Validator::new("Sign ?").ask() {
        let k = get_private_key(&BIP32_PATH)?;
        let (sig, _sig_len) = detecdsa_sign(message, &k).unwrap();
        ui::popup("Done !");
        Ok(Some(sig))
    } else {
        ui::popup("Cancelled");
        Ok(None)
    }
}

#[cfg(target_os="nanos")]
use rust_app::{mk_parsers, ParserTag, RX};
use ledger_parser_combinators::interp_parser::OOB;

#[cfg(target_os="nanos")]
#[cfg(not(test))]
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

#[cfg(target_os="nanos")]
use nanos_sdk::io::Reply;
// use nanos_sdk::debug_print;
use arrayvec::ArrayVec;

#[cfg(target_os="nanos")]
fn handle_apdu<P: for<'a> FnMut(ParserTag, &'a [u8]) -> RX<'a, ArrayVec<u8, 260> > >(comm: &mut io::Comm, ins: Ins, mut parser: P) -> Result<(), Reply> {
    if comm.rx == 0 {
        return Err(io::StatusWords::NothingReceived.into());
    }

    // Could be made standalone.
    let mut run_parser_apdu = | tag | -> Result<(), Reply> {
        let mut cursor = comm.get_data()?;

        loop {
            //debug_print("Entering the parser\n");
            let parse_rv = parser(tag, cursor);
            //debug_print("Passed the parser\n");
            match parse_rv {
                Err((Some(OOB::Prompt(prompt)), new_cursor)) => {
                    // TODO: Actually do something UI with the prompt here.
                    let mut pmpt = ArrayVec::<&str, 2>::new();
                    pmpt.extend(prompt.into_iter().map(|a| a.as_str()));
                    ui::MessageValidator::new(&pmpt, &[], &[]).ask();
                    cursor=new_cursor;
                }
                Err((Some(OOB::Reject), _)) => { let _ = parser(ParserTag::Reset, cursor); break Ok(()) } // Rejection; reset the parser. Possibly send error message to host?
                // Deliberately no catch-all on the Err((Some case; we'll get error messages if we
                // add to OOB's out-of-band actions and forget to implement them.
                Err((None, [])) => { break Ok(()) } // Finished the chunk with no further actions pending
                Err((None, _)) => { let _ = parser(ParserTag::Reset, cursor); break Ok(()) } // Finished the parse incorrectly; reset and error message.
                Ok((rv, [])) => {
                    comm.append(&rv[..]);
                    // Parse finished; reset.
                    let _ = parser(ParserTag::Reset, b"");
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
