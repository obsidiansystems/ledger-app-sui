use crate::implementation::*;
use crate::interface::*;

use alamgu_async_block::*;

use ledger_log::{info, trace};
use ledger_prompts_ui::RootMenu;

use core::cell::RefCell;
use core::pin::Pin;
use nanos_sdk::io;
use pin_cell::*;

#[allow(dead_code)]
pub fn app_main() {
    let comm: SingleThreaded<RefCell<io::Comm>> = SingleThreaded(RefCell::new(io::Comm::new()));

    let hostio_state: SingleThreaded<RefCell<HostIOState>> =
        SingleThreaded(RefCell::new(HostIOState::new(unsafe {
            core::mem::transmute(&comm.0)
        })));
    let hostio: SingleThreaded<HostIO> =
        SingleThreaded(HostIO(unsafe { core::mem::transmute(&hostio_state.0) }));
    let states_backing: SingleThreaded<PinCell<Option<APDUsFuture>>> =
        SingleThreaded(PinCell::new(None));
    let states: SingleThreaded<Pin<&PinCell<Option<APDUsFuture>>>> =
        SingleThreaded(Pin::static_ref(unsafe {
            core::mem::transmute(&states_backing.0)
        }));

    let mut idle_menu = RootMenu::new([
        concat!("Alamgu Example ", env!("CARGO_PKG_VERSION")),
        "Exit",
    ]);
    let mut busy_menu = RootMenu::new(["Working...", "Cancel"]);

    info!("Alamgu Example {}", env!("CARGO_PKG_VERSION"));
    info!(
        "State sizes\ncomm: {}\nstates: {}",
        core::mem::size_of::<io::Comm>(),
        core::mem::size_of::<Option<APDUsFuture>>()
    );

    let // Draw some 'welcome' screen
        menu = |states : core::cell::Ref<'_, Option<APDUsFuture>>, idle : & mut RootMenu<2>, busy : & mut RootMenu<2>| {
            match states.is_none() {
                true => idle.show(),
                _ => busy.show(),
            }
        };

    menu(states.borrow(), &mut idle_menu, &mut busy_menu);
    loop {
        // Wait for either a specific button push to exit the app
        // or an APDU command
        let evt = comm.borrow_mut().next_event::<Ins>();
        match evt {
            io::Event::Command(ins) => {
                trace!("Command received");
                let poll_rv = poll_apdu_handlers(
                    PinMut::as_mut(&mut states.0.borrow_mut()),
                    ins,
                    *hostio,
                    (),
                    handle_apdu_async,
                );
                match poll_rv {
                    Ok(()) => {
                        trace!("APDU accepted; sending response");
                        comm.borrow_mut().reply_ok();
                        trace!("Replied");
                    }
                    Err(sw) => comm.borrow_mut().reply(sw),
                };
                menu(states.borrow(), &mut idle_menu, &mut busy_menu);
                trace!("Command done");
            }
            io::Event::Button(btn) => {
                trace!("Button received");
                match states.borrow().is_none() {
                    true => {
                        if let Some(1) = idle_menu.update(btn) {
                            info!("Exiting app at user direction via root menu");
                            nanos_sdk::exit_app(0)
                        }
                    }
                    _ => {
                        if let Some(1) = idle_menu.update(btn) {
                            info!("Resetting at user direction via busy menu");
                            PinMut::as_mut(&mut states.borrow_mut()).set(None);
                        }
                    }
                };
                menu(states.borrow(), &mut idle_menu, &mut busy_menu);
                trace!("Button done");
            }
            io::Event::Ticker => {
                //trace!("Ignoring ticker event");
            }
        }
    }
}

// We are single-threaded in fact, albeit with nontrivial code flow. We don't need to worry about
// full atomicity of the below globals.
struct SingleThreaded<T>(T);
unsafe impl<T> Send for SingleThreaded<T> {}
unsafe impl<T> Sync for SingleThreaded<T> {}
impl<T> core::ops::Deref for SingleThreaded<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> core::ops::DerefMut for SingleThreaded<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
