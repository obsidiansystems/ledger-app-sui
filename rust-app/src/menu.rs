use crate::settings::*;
use ledger_prompts_ui::Menu;

pub struct IdleMenuWithSettings {
    pub idle_menu: IdleMenu,
    pub settings: Settings,
}

pub enum IdleMenu {
    AppMain,
    Settings(Option<SettingsSubMenu>),
    Exit,
}

pub enum SettingsSubMenu {
    EnableBlindSigning,
    DisableBlindSigning,
    Back,
}

pub enum BusyMenu {
    Working,
    Cancel,
}

pub struct DoExitApp;

impl Menu for IdleMenuWithSettings {
    type BothResult = DoExitApp;
    fn move_left(&mut self) {
        use crate::menu::IdleMenu::*;
        use crate::menu::SettingsSubMenu::*;
        match self.idle_menu {
            Settings(None) => self.idle_menu = AppMain,
            Settings(Some(Back)) => {
                if self.settings.get() == 1 {
                    self.idle_menu = Settings(Some(DisableBlindSigning))
                } else {
                    self.idle_menu = Settings(Some(EnableBlindSigning))
                }
            }
            Exit => self.idle_menu = Settings(None),
            _ => {}
        };
    }
    fn move_right(&mut self) {
        use crate::menu::IdleMenu::*;
        use crate::menu::SettingsSubMenu::*;
        match self.idle_menu {
            AppMain => self.idle_menu = Settings(None),
            Settings(None) => self.idle_menu = Exit,
            Settings(Some(_)) => self.idle_menu = Settings(Some(Back)),
            _ => {}
        };
    }
    #[inline(never)]
    fn handle_both(&mut self) -> Option<Self::BothResult> {
        use crate::menu::IdleMenu::*;
        use crate::menu::SettingsSubMenu::*;
        match self.idle_menu {
            AppMain => None,
            Settings(None) => {
                if self.settings.get() == 1 {
                    self.idle_menu = Settings(Some(DisableBlindSigning))
                } else {
                    self.idle_menu = Settings(Some(EnableBlindSigning))
                };
                None
            }
            Settings(Some(EnableBlindSigning)) => {
                self.settings.set(&1);
                self.idle_menu = Settings(Some(DisableBlindSigning));
                None
            }
            Settings(Some(DisableBlindSigning)) => {
                self.settings.set(&0);
                self.idle_menu = Settings(Some(EnableBlindSigning));
                None
            }
            Settings(Some(Back)) => {
                self.idle_menu = Settings(None);
                None
            }
            Exit => Some(DoExitApp),
        }
    }
    #[inline(never)]
    fn label(&self) -> &str {
        use crate::menu::IdleMenu::*;
        use crate::menu::SettingsSubMenu::*;
        match self.idle_menu {
            AppMain => concat!("Sui ", env!("CARGO_PKG_VERSION")),
            Settings(None) => "Settings",
            Settings(Some(EnableBlindSigning)) => "Enable Blind Signing",
            Settings(Some(DisableBlindSigning)) => "Disable Blind Signing",
            Settings(Some(Back)) => "Back",
            Exit => "Exit",
        }
    }
}

pub struct DoCancel;

impl Menu for BusyMenu {
    type BothResult = DoCancel;
    fn move_left(&mut self) {
        *self = BusyMenu::Working;
    }
    fn move_right(&mut self) {
        *self = BusyMenu::Cancel;
    }
    #[inline(never)]
    fn handle_both(&mut self) -> Option<Self::BothResult> {
        use crate::menu::BusyMenu::*;
        match self {
            Working => None,
            Cancel => Some(DoCancel),
        }
    }
    #[inline(never)]
    fn label(&self) -> &str {
        use crate::menu::BusyMenu::*;
        match self {
            Working => "Working...",
            Cancel => "Cancel",
        }
    }
}
