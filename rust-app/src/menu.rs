use crate::settings::*;
use include_gif::include_gif;
use ledger_prompts_ui::*;
use nanos_ui::bagls::*;
use nanos_ui::bitmaps::Glyph;

#[cfg(not(target_os = "nanos"))]
pub const APP_ICON_GLYPH: Glyph = Glyph::from_include(include_gif!("crab-small.gif"));
#[cfg(not(target_os = "nanos"))]
pub const APP_ICON: Icon = Icon::from(&APP_ICON_GLYPH).set_x(55).shift_v(-10);

// TODO: fix nanos icon
#[cfg(target_os = "nanos")]
const APP_ICON: Icon = SETTINGS_ICON;

pub struct IdleMenuWithSettings {
    pub idle_menu: IdleMenu,
    pub settings: Settings,
}

pub enum IdleMenu {
    AppMain,
    ShowVersion,
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
            AppMain => self.idle_menu = Exit,
            ShowVersion => self.idle_menu = AppMain,
            Settings(None) => self.idle_menu = ShowVersion,
            Settings(Some(Back)) => {
                if self.settings.get() == 1 {
                    self.idle_menu = Settings(Some(DisableBlindSigning))
                } else {
                    self.idle_menu = Settings(Some(EnableBlindSigning))
                }
            }
            Settings(Some(_)) => self.idle_menu = Settings(Some(Back)),
            Exit => self.idle_menu = Settings(None),
        };
    }
    fn move_right(&mut self) {
        use crate::menu::IdleMenu::*;
        use crate::menu::SettingsSubMenu::*;
        match self.idle_menu {
            AppMain => self.idle_menu = ShowVersion,
            ShowVersion => self.idle_menu = Settings(None),
            Settings(None) => self.idle_menu = Exit,
            Settings(Some(Back)) => {
                if self.settings.get() == 1 {
                    self.idle_menu = Settings(Some(DisableBlindSigning))
                } else {
                    self.idle_menu = Settings(Some(EnableBlindSigning))
                }
            }
            Settings(Some(_)) => self.idle_menu = Settings(Some(Back)),
            Exit => self.idle_menu = AppMain,
        };
    }
    #[inline(never)]
    fn handle_both(&mut self) -> Option<Self::BothResult> {
        use crate::menu::IdleMenu::*;
        use crate::menu::SettingsSubMenu::*;
        match self.idle_menu {
            AppMain => None,
            ShowVersion => None,
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
    fn label<'a>(&self) -> (MenuLabelTop<'a>, MenuLabelBottom<'a>) {
        use crate::menu::IdleMenu::*;
        use crate::menu::SettingsSubMenu::*;
        match self.idle_menu {
            AppMain => (
                MenuLabelTop::Icon(&APP_ICON),
                MenuLabelBottom {
                    text: "Alamgu Example",
                    bold: true,
                },
            ),
            ShowVersion => (
                MenuLabelTop::Text("Version"),
                MenuLabelBottom {
                    text: env!("CARGO_PKG_VERSION"),
                    bold: false,
                },
            ),
            Settings(None) => (
                MenuLabelTop::Icon(&SETTINGS_ICON),
                MenuLabelBottom {
                    text: "Settings",
                    bold: true,
                },
            ),
            Settings(Some(EnableBlindSigning)) => (
                MenuLabelTop::Text("Blind Signing"),
                MenuLabelBottom {
                    text: "Disabled",
                    bold: false,
                },
            ),
            Settings(Some(DisableBlindSigning)) => (
                MenuLabelTop::Text("Blind Signing"),
                MenuLabelBottom {
                    text: "Enabled",
                    bold: false,
                },
            ),
            Settings(Some(Back)) => (
                MenuLabelTop::Icon(&BACK_ICON),
                MenuLabelBottom {
                    text: "Back",
                    bold: true,
                },
            ),
            Exit => (
                MenuLabelTop::Icon(&DASHBOARD_ICON),
                MenuLabelBottom {
                    text: "Quit",
                    bold: true,
                },
            ),
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
    fn label<'a>(&self) -> (MenuLabelTop<'a>, MenuLabelBottom<'a>) {
        use crate::menu::BusyMenu::*;
        match self {
            Working => (
                MenuLabelTop::Text("Working..."),
                MenuLabelBottom {
                    text: "",
                    bold: false,
                },
            ),
            Cancel => (
                MenuLabelTop::Text("Cancel"),
                MenuLabelBottom {
                    text: "",
                    bold: false,
                },
            ),
        }
    }
}
