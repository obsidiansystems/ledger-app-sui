use ledger_device_sdk::ui;
use ledger_device_sdk::{buttons::ButtonEvent, io};

use crate::interface::Ins;

#[no_mangle]
extern "C" fn sample_pending() {
    let mut comm = io::Comm::new();

    loop {
        ui::SingleMessage::new("Pending").show();
        match comm.next_event::<Ins>() {
            io::Event::Button(ButtonEvent::RightButtonRelease) => break,
            _ => (),
        }
    }
    loop {
        ui::SingleMessage::new("Ledger review").show();
        match comm.next_event::<Ins>() {
            io::Event::Button(ButtonEvent::BothButtonsRelease) => break,
            _ => (),
        }
    }
}
