use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::model::{TenantProfile, TrafficPattern};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Tenants,
    Preview,
    Help,
}

pub struct App {
    pub current_screen: Screen,
    pub should_quit: bool,
    pub selected_tenant: usize,
    pub tenants: Vec<TenantProfile>,
    pub status_message: String,
    pub focused_screen: Screen,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_screen: Screen::Dashboard,
            should_quit: false,
            selected_tenant: 0,
            tenants: vec![
                TenantProfile::new(
                    "tenant_a",
                    "Tenant A",
                    TrafficPattern::Steady,
                    10,
                    512,
                    1,
                    60,
                ),
                TenantProfile::new(
                    "tenant_b",
                    "Tenant B",
                    TrafficPattern::Bursty,
                    50,
                    1024,
                    2,
                    30,
                ),
                TenantProfile::new(
                    "tenant_c",
                    "Tenant C",
                    TrafficPattern::Heavy,
                    20,
                    256,
                    1,
                    45,
                ),
            ],
            status_message: "Ready".to_string(),
            focused_screen: Screen::Dashboard,
        }
    }
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.should_quit = true;
            self.status_message = "Quit via Ctrl+C".to_string();
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                self.status_message = "Quit via q".to_string();
            }
            KeyCode::Char('1') => {
                self.set_active_screen(Screen::Dashboard);
                self.status_message = "Dashboard screen".to_string();
            }
            KeyCode::Char('2') => {
                self.set_active_screen(Screen::Tenants);
                self.status_message = "Tenant profiles screen".to_string();
            }
            KeyCode::Char('3') => {
                self.set_active_screen(Screen::Preview);
                self.status_message = "Traffic preview screen".to_string();
            }
            KeyCode::Char('h') => {
                self.set_active_screen(Screen::Help);
                self.status_message = "Help screen".to_string();
            }
            KeyCode::Tab | KeyCode::Right => {
                self.focused_screen = Self::next_screen(self.focused_screen);
                self.status_message = format!(
                    "Focused tab: {} (press Enter to open)",
                    self.focused_screen_name()
                );
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.focused_screen = Self::previous_screen(self.focused_screen);
                self.status_message = format!(
                    "Focused tab: {} (press Enter to open)",
                    self.focused_screen_name()
                );
            }
            KeyCode::Enter => {
                self.current_screen = self.focused_screen;
                self.status_message = format!("Opened {}", self.focused_screen_name());
            }
            KeyCode::Up => {
                if self.current_screen == Screen::Tenants && self.selected_tenant > 0 {
                    self.selected_tenant -= 1;
                }
            }
            KeyCode::Down => {
                if self.current_screen == Screen::Tenants
                    && self.selected_tenant + 1 < self.tenants.len()
                {
                    self.selected_tenant += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.current_screen == Screen::Tenants && self.selected_tenant > 0 {
                    self.selected_tenant -= 1;
                }
            }
            KeyCode::Char('j') => {
                if self.current_screen == Screen::Tenants
                    && self.selected_tenant + 1 < self.tenants.len()
                {
                    self.selected_tenant += 1;
                }
            }
            _ => {}
        }
    }

    fn set_active_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
        self.focused_screen = screen;
    }

    fn next_screen(screen: Screen) -> Screen {
        match screen {
            Screen::Dashboard => Screen::Tenants,
            Screen::Tenants => Screen::Preview,
            Screen::Preview => Screen::Help,
            Screen::Help => Screen::Dashboard,
        }
    }

    fn previous_screen(screen: Screen) -> Screen {
        match screen {
            Screen::Dashboard => Screen::Help,
            Screen::Tenants => Screen::Dashboard,
            Screen::Preview => Screen::Tenants,
            Screen::Help => Screen::Preview,
        }
    }

    pub fn focused_screen_name(&self) -> &'static str {
        match self.focused_screen {
            Screen::Dashboard => "Dashboard",
            Screen::Tenants => "Tenants",
            Screen::Preview => "Traffic Preview",
            Screen::Help => "Help",
        }
    }
}
