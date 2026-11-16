use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::sprites::Character;

pub enum TrayAction {
    TogglePause,
    SetCharacter(Character),
    Quit,
}

pub struct Tray {
    icon: TrayIcon,
    pause: CheckMenuItem,
    characters: Vec<(Character, CheckMenuItem)>,
    quit: MenuItem,
}

impl Tray {
    pub fn new(current: Character, rgba: Vec<u8>, w: u32, h: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();
        let pause = CheckMenuItem::new("Pause (hide)", true, false, None);
        menu.append(&pause)?;

        let submenu = Submenu::new("Character", true);
        let mut characters = Vec::new();
        for c in Character::ALL {
            let item = CheckMenuItem::new(c.label(), true, c == current, None);
            submenu.append(&item)?;
            characters.push((c, item));
        }
        menu.append(&submenu)?;

        let quit = MenuItem::new("Quit oneko", true, None);
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit)?;

        let builder = TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip("oneko");
        let icon = Self::make_icon(rgba, w, h)?;
        // macOS menu bar icons are templates (alpha only); Windows wants the full-colour sprite.
        #[cfg(target_os = "macos")]
        let builder = builder.with_icon_templated(icon);
        #[cfg(not(target_os = "macos"))]
        let builder = builder.with_icon(icon);
        let icon = builder.build()?;
        Ok(Self { icon, pause, characters, quit })
    }

    fn make_icon(rgba: Vec<u8>, w: u32, h: u32) -> Result<Icon, tray_icon::BadIcon> {
        Icon::from_rgba(rgba, w, h)
    }

    pub fn set_icon(&self, rgba: Vec<u8>, w: u32, h: u32) {
        if let Ok(icon) = Self::make_icon(rgba, w, h) {
            // Keep the template flag on macOS, otherwise the icon turns solid black on dark menu bars.
            #[cfg(target_os = "macos")]
            let _ = self.icon.set_icon_templated(Some(icon));
            #[cfg(not(target_os = "macos"))]
            let _ = self.icon.set_icon(Some(icon));
        }
    }

    pub fn poll(&self) -> Option<TrayAction> {
        let ev = MenuEvent::receiver().try_recv().ok()?;
        if ev.id == *self.pause.id() {
            return Some(TrayAction::TogglePause);
        }
        if ev.id == *self.quit.id() {
            return Some(TrayAction::Quit);
        }
        for (c, item) in &self.characters {
            if ev.id == *item.id() {
                // behave like a radio group
                for (other, it) in &self.characters {
                    it.set_checked(other == c);
                }
                return Some(TrayAction::SetCharacter(*c));
            }
        }
        None
    }
}
