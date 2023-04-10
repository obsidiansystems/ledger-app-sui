#[cfg(target_os = "nanos")]
mod nanos {
    use nanos_sdk::nvm::*;
    use nanos_sdk::NVMData;

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
}

#[cfg(not(target_os = "nanos"))]
mod nanosplus {
    #[derive(Clone, Copy)]
    pub struct Settings(u8);

    impl Settings {
        pub fn new() -> Settings {
            Settings(0)
        }

        pub fn get(&self) -> u8 {
            self.0
        }

        pub fn set(&mut self, v: &u8) {
            self.0 = *v;
        }
    }

    impl Default for Settings {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(target_os = "nanos")]
pub use nanos::*;

#[cfg(not(target_os = "nanos"))]
pub use nanosplus::*;
