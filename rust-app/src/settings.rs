use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;

// This is necessary to store the object in NVM and not in RAM
#[link_section = ".nvm_data"]
static mut SETTINGS: NVMData<AtomicStorage<u8>> = NVMData::new(AtomicStorage::new(&0));

#[derive(Clone, Copy)]
pub struct Settings;

impl Default for Settings {
    fn default() -> Self {
        Settings
    }
}

impl Settings {
    #[inline(never)]
    pub fn get(&self) -> u8 {
        let settings = unsafe { SETTINGS.get_mut() };
        return *settings.get_ref();
    }

    // The inline(never) is important. Otherwise weird segmentation faults happen on speculos.
    #[inline(never)]
    pub fn set(&mut self, v: &u8) {
        let settings = unsafe { SETTINGS.get_mut() };
        settings.update(v);
    }
}
