use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::model::{TargetConfig, TargetSystem, TenantProfile, TrafficEvent, TrafficPattern};

#[derive(Debug, Clone)]
pub struct WorkerPreviewState {
    pub worker_id: String,
    pub current_load: u32,
    pub is_free: bool,
    pub is_busy: bool,
    pub processed_tasks: u64,
    pub submitted_tasks: u64,
    pub last_error: Option<String>,
}

impl Default for WorkerPreviewState {
    fn default() -> Self {
        Self {
            worker_id: "preview_worker".to_string(),
            current_load: 0,
            is_free: true,
            is_busy: false,
            processed_tasks: 0,
            submitted_tasks: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SendPreviewState {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Tenants,
    AddTenant,
    Preview,
    GeneratedEvents,
    Target,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetField {
    Enabled,
    System,
    Protocol,
    Endpoint,
    HttpPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddTenantField {
    TenantName,
    TrafficPattern,
    RequestsPerSecond,
    PayloadSizeBytes,
    Priority,
    DurationSeconds,
}

#[derive(Debug, Clone)]
pub struct AddTenantForm {
    pub mode: TenantFormMode,
    pub tenant_id_preview: String,
    pub tenant_name: String,
    pub traffic_pattern_index: usize,
    pub requests_per_second: String,
    pub payload_size_bytes: String,
    pub priority: String,
    pub duration_seconds: String,
    pub active_field: AddTenantField,
    pub validation_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TenantFormMode {
    Add,
    Edit { index: usize, tenant_id: String },
}

impl Default for AddTenantForm {
    fn default() -> Self {
        Self {
            mode: TenantFormMode::Add,
            tenant_id_preview: Uuid::new_v4().to_string(),
            tenant_name: String::new(),
            traffic_pattern_index: 0,
            requests_per_second: "10".to_string(),
            payload_size_bytes: "512".to_string(),
            priority: "1".to_string(),
            duration_seconds: "60".to_string(),
            active_field: AddTenantField::TenantName,
            validation_error: None,
        }
    }
}

pub struct App {
    pub current_screen: Screen,
    pub should_quit: bool,
    pub selected_tenant: usize,
    pub tenants: Vec<TenantProfile>,
    pub status_message: String,
    pub focused_screen: Screen,
    pub worker_preview: WorkerPreviewState,
    pub preview_scroll_offset: u16,
    pub tenant_scroll_offset: usize,
    pub add_tenant_form: AddTenantForm,
    pub pending_generate_export: bool,
    pub last_generated_events: usize,
    pub last_generated_output_path: String,
    pub generation_error: Option<String>,
    pub last_generated_preview: Vec<TrafficEvent>,
    pub generated_events: Vec<TrafficEvent>,
    pub generated_event_scroll_offset: usize,
    pub selected_generated_event: usize,
    pub target_config: TargetConfig,
    pub target_field: TargetField,
    pub send_preview: SendPreviewState,
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
                    TrafficPattern::Burst,
                    50,
                    1024,
                    2,
                    30,
                )
                .with_burst_config(50, 1000),
                TenantProfile::new(
                    "tenant_c",
                    "Tenant C",
                    TrafficPattern::Random,
                    20,
                    256,
                    1,
                    45,
                )
                .with_random_interval_config(50, 150),
            ],
            status_message: "Ready".to_string(),
            focused_screen: Screen::Dashboard,
            worker_preview: WorkerPreviewState::default(),
            preview_scroll_offset: 0,
            tenant_scroll_offset: 0,
            add_tenant_form: AddTenantForm::default(),
            pending_generate_export: false,
            last_generated_events: 0,
            last_generated_output_path: "results/traffic_events.jsonl".to_string(),
            generation_error: None,
            last_generated_preview: Vec::new(),
            generated_events: Vec::new(),
            generated_event_scroll_offset: 0,
            selected_generated_event: 0,
            target_config: TargetConfig::default(),
            target_field: TargetField::Enabled,
            send_preview: SendPreviewState::default(),
        }
    }
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.current_screen == Screen::AddTenant {
            self.handle_add_tenant_key(key);
            return;
        }

        if self.current_screen == Screen::Preview {
            self.handle_preview_key(key);
            return;
        }

        if self.current_screen == Screen::GeneratedEvents {
            self.handle_generated_events_key(key);
            return;
        }

        if self.current_screen == Screen::Target {
            self.handle_target_key(key);
            return;
        }

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
            KeyCode::Char('a') => {
                if self.current_screen == Screen::Tenants {
                    self.open_add_tenant_form();
                    self.status_message =
                        "Add Tenant form opened (Enter to save, Esc to cancel)".to_string();
                }
            }
            KeyCode::Char('e') => {
                if self.current_screen == Screen::Tenants {
                    self.open_edit_tenant_form();
                }
            }
            KeyCode::Char('d') => {
                if self.current_screen == Screen::Tenants {
                    self.delete_selected_tenant();
                }
            }
            KeyCode::Char('3') => {
                self.set_active_screen(Screen::Preview);
                self.status_message = "Traffic preview screen".to_string();
            }
            KeyCode::Char('4') => {
                self.set_active_screen(Screen::GeneratedEvents);
                self.status_message = "Generated events screen".to_string();
            }
            KeyCode::Char('5') => {
                self.set_active_screen(Screen::Target);
                self.status_message = "Target configuration screen".to_string();
            }
            KeyCode::Char('g') => {
                self.pending_generate_export = true;
                self.set_active_screen(Screen::Preview);
                self.status_message =
                    "Generating traffic events and exporting to results/traffic_events.jsonl..."
                        .to_string();
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
                    self.ensure_selected_visible(10);
                }
            }
            KeyCode::Down => {
                if self.current_screen == Screen::Tenants
                    && self.selected_tenant + 1 < self.tenants.len()
                {
                    self.selected_tenant += 1;
                    self.ensure_selected_visible(10);
                }
            }
            KeyCode::Char('k') => {
                if self.current_screen == Screen::Tenants && self.selected_tenant > 0 {
                    self.selected_tenant -= 1;
                    self.ensure_selected_visible(10);
                }
            }
            KeyCode::Char('j') => {
                if self.current_screen == Screen::Tenants
                    && self.selected_tenant + 1 < self.tenants.len()
                {
                    self.selected_tenant += 1;
                    self.ensure_selected_visible(10);
                }
            }
            KeyCode::PageUp => {
                if self.current_screen == Screen::Tenants {
                    let step = 8;
                    self.selected_tenant = self.selected_tenant.saturating_sub(step);
                    self.ensure_selected_visible(10);
                }
            }
            KeyCode::PageDown => {
                if self.current_screen == Screen::Tenants {
                    let step = 8;
                    let max_index = self.tenants.len().saturating_sub(1);
                    self.selected_tenant = (self.selected_tenant + step).min(max_index);
                    self.ensure_selected_visible(10);
                }
            }
            _ => {}
        }
    }

    fn set_active_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
        if screen != Screen::AddTenant {
            self.focused_screen = screen;
        }
    }

    fn next_screen(screen: Screen) -> Screen {
        match screen {
            Screen::Dashboard => Screen::Tenants,
            Screen::Tenants => Screen::Preview,
            Screen::AddTenant => Screen::Preview,
            Screen::Preview => Screen::GeneratedEvents,
            Screen::GeneratedEvents => Screen::Target,
            Screen::Target => Screen::Help,
            Screen::Help => Screen::Dashboard,
        }
    }

    fn previous_screen(screen: Screen) -> Screen {
        match screen {
            Screen::Dashboard => Screen::Help,
            Screen::Tenants => Screen::Dashboard,
            Screen::AddTenant => Screen::Tenants,
            Screen::Preview => Screen::Tenants,
            Screen::GeneratedEvents => Screen::Preview,
            Screen::Target => Screen::GeneratedEvents,
            Screen::Help => Screen::Target,
        }
    }

    pub fn focused_screen_name(&self) -> &'static str {
        match self.focused_screen {
            Screen::Dashboard => "Dashboard",
            Screen::Tenants => "Tenants",
            Screen::AddTenant => "Add Tenant",
            Screen::Preview => "Traffic Preview",
            Screen::GeneratedEvents => "Generated Events",
            Screen::Target => "Target Config",
            Screen::Help => "Help",
        }
    }

    fn open_add_tenant_form(&mut self) {
        self.current_screen = Screen::AddTenant;
        self.add_tenant_form = AddTenantForm::default();
        self.add_tenant_form.mode = TenantFormMode::Add;
    }

    fn open_edit_tenant_form(&mut self) {
        let Some(tenant) = self.tenants.get(self.selected_tenant).cloned() else {
            self.status_message = "No tenant selected to edit".to_string();
            return;
        };

        self.current_screen = Screen::AddTenant;
        self.add_tenant_form = AddTenantForm {
            mode: TenantFormMode::Edit {
                index: self.selected_tenant,
                tenant_id: tenant.tenant_id.clone(),
            },
            tenant_id_preview: tenant.tenant_id,
            tenant_name: tenant.tenant_name,
            traffic_pattern_index: Self::traffic_pattern_to_index(tenant.traffic_pattern),
            requests_per_second: tenant.requests_per_second.to_string(),
            payload_size_bytes: tenant.payload_size_bytes.to_string(),
            priority: tenant.priority.to_string(),
            duration_seconds: tenant.duration_seconds.to_string(),
            active_field: AddTenantField::TenantName,
            validation_error: None,
        };
        self.status_message = "Edit Tenant form opened (Enter to save, Esc to cancel)".to_string();
    }

    fn delete_selected_tenant(&mut self) {
        if self.tenants.is_empty() {
            self.status_message = "No tenants to delete".to_string();
            return;
        }

        let removed = self.tenants.remove(self.selected_tenant);

        if self.selected_tenant >= self.tenants.len() {
            self.selected_tenant = self.tenants.len().saturating_sub(1);
        }

        self.ensure_selected_visible(10);
        self.status_message = format!("Deleted tenant {}", removed.tenant_name);
    }

    fn handle_preview_key(&mut self, key: KeyEvent) {
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
                return;
            }
            KeyCode::Char('1') => {
                self.set_active_screen(Screen::Dashboard);
                self.status_message = "Dashboard screen".to_string();
                return;
            }
            KeyCode::Char('2') => {
                self.set_active_screen(Screen::Tenants);
                self.status_message = "Tenant profiles screen".to_string();
                return;
            }
            KeyCode::Char('3') => {
                self.set_active_screen(Screen::Preview);
                self.status_message = "Traffic preview screen".to_string();
                return;
            }
            KeyCode::Char('4') => {
                self.set_active_screen(Screen::GeneratedEvents);
                self.status_message = "Generated events screen".to_string();
                return;
            }
            KeyCode::Char('5') => {
                self.set_active_screen(Screen::Target);
                self.status_message = "Target configuration screen".to_string();
                return;
            }
            KeyCode::Char('h') => {
                self.set_active_screen(Screen::Help);
                self.status_message = "Help screen".to_string();
                return;
            }
            KeyCode::Tab | KeyCode::Right => {
                self.focused_screen = Self::next_screen(self.focused_screen);
                self.status_message = format!(
                    "Focused tab: {} (press Enter to open)",
                    self.focused_screen_name()
                );
                return;
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.focused_screen = Self::previous_screen(self.focused_screen);
                self.status_message = format!(
                    "Focused tab: {} (press Enter to open)",
                    self.focused_screen_name()
                );
                return;
            }
            KeyCode::Enter => {
                self.current_screen = self.focused_screen;
                self.status_message = format!("Opened {}", self.focused_screen_name());
                return;
            }
            _ => {}
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_sub(8);
            }
            KeyCode::PageDown => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_add(8);
            }
            KeyCode::Char('g') => {
                self.pending_generate_export = true;
                self.status_message =
                    "Generating traffic events and exporting to results/traffic_events.jsonl..."
                        .to_string();
            }
            _ => {}
        }
    }

    fn handle_generated_events_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.should_quit = true;
            self.status_message = "Quit via Ctrl+C".to_string();
            return;
        }

        // Handle navigation and global action keys FIRST
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                self.status_message = "Quit via q".to_string();
                return;
            }
            KeyCode::Char('1') => {
                self.set_active_screen(Screen::Dashboard);
                self.status_message = "Dashboard screen".to_string();
                return;
            }
            KeyCode::Char('2') => {
                self.set_active_screen(Screen::Tenants);
                self.status_message = "Tenant profiles screen".to_string();
                return;
            }
            KeyCode::Char('3') => {
                self.set_active_screen(Screen::Preview);
                self.status_message = "Traffic preview screen".to_string();
                return;
            }
            KeyCode::Char('4') => {
                self.set_active_screen(Screen::GeneratedEvents);
                self.status_message = "Generated events screen".to_string();
                return;
            }
            KeyCode::Char('5') => {
                self.set_active_screen(Screen::Target);
                self.status_message = "Target configuration screen".to_string();
                return;
            }
            KeyCode::Char('h') => {
                self.set_active_screen(Screen::Help);
                self.status_message = "Help screen".to_string();
                return;
            }
            KeyCode::Esc => {
                self.set_active_screen(Screen::Preview);
                self.status_message = "Returned to preview".to_string();
                return;
            }
            _ => {}
        }

        // Handle local navigation AFTER checking global keys
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_generated_event > 0 {
                    self.selected_generated_event -= 1;
                    self.ensure_generated_event_visible(10);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_generated_event + 1 < self.generated_events.len() {
                    self.selected_generated_event += 1;
                    self.ensure_generated_event_visible(10);
                }
            }
            KeyCode::PageUp => {
                let step = 8;
                self.selected_generated_event = self.selected_generated_event.saturating_sub(step);
                self.ensure_generated_event_visible(10);
            }
            KeyCode::PageDown => {
                let max_index = self.generated_events.len().saturating_sub(1);
                let step = 8;
                self.selected_generated_event =
                    (self.selected_generated_event + step).min(max_index);
                self.ensure_generated_event_visible(10);
            }
            KeyCode::Char('g') => {
                self.pending_generate_export = true;
                self.status_message = "Regenerating traffic events...".to_string();
            }
            _ => {}
        }
    }

    fn handle_target_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.should_quit = true;
            self.status_message = "Quit via Ctrl+C".to_string();
            return;
        }

        // Handle navigation and global action keys FIRST, before field input
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                self.status_message = "Quit via q".to_string();
                return;
            }
            KeyCode::Char('1') => {
                self.set_active_screen(Screen::Dashboard);
                self.status_message = "Dashboard screen".to_string();
                return;
            }
            KeyCode::Char('2') => {
                self.set_active_screen(Screen::Tenants);
                self.status_message = "Tenant profiles screen".to_string();
                return;
            }
            KeyCode::Char('3') => {
                self.set_active_screen(Screen::Preview);
                self.status_message = "Traffic preview screen".to_string();
                return;
            }
            KeyCode::Char('4') => {
                self.set_active_screen(Screen::GeneratedEvents);
                self.status_message = "Generated events screen".to_string();
                return;
            }
            KeyCode::Char('5') => {
                self.set_active_screen(Screen::Target);
                self.status_message = "Target configuration screen".to_string();
                return;
            }
            KeyCode::Char('g') => {
                self.pending_generate_export = true;
                self.set_active_screen(Screen::Preview);
                self.status_message =
                    "Generating and sending events using target config...".to_string();
                return;
            }
            KeyCode::Char('h') => {
                self.set_active_screen(Screen::Help);
                self.status_message = "Help screen".to_string();
                return;
            }
            KeyCode::Esc => {
                self.set_active_screen(Screen::Preview);
                self.status_message = "Returned to preview".to_string();
                return;
            }
            _ => {}
        }

        // Handle field navigation and editing AFTER checking global keys
        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                self.target_field = Self::next_target_field(self.target_field);
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.target_field = Self::previous_target_field(self.target_field);
            }
            KeyCode::Left => {
                self.adjust_target_field(false);
            }
            KeyCode::Right => {
                self.adjust_target_field(true);
            }
            KeyCode::Backspace => {
                self.backspace_target_field();
            }
            KeyCode::Enter => {
                self.adjust_target_field(true);
                self.status_message = "Target configuration updated".to_string();
            }
            KeyCode::Char(ch) => {
                self.push_target_field_char(ch);
            }
            _ => {}
        }
    }

    fn handle_add_tenant_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.should_quit = true;
            self.status_message = "Quit via Ctrl+C".to_string();
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.current_screen = Screen::Tenants;
                self.status_message = "Add Tenant canceled".to_string();
            }
            KeyCode::Tab | KeyCode::Down => {
                self.add_tenant_form.active_field =
                    Self::next_add_field(self.add_tenant_form.active_field);
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.add_tenant_form.active_field =
                    Self::previous_add_field(self.add_tenant_form.active_field);
            }
            KeyCode::Left => {
                if self.add_tenant_form.active_field == AddTenantField::TrafficPattern {
                    self.rotate_pattern_backward();
                }
            }
            KeyCode::Right => {
                if self.add_tenant_form.active_field == AddTenantField::TrafficPattern {
                    self.rotate_pattern_forward();
                }
            }
            KeyCode::Backspace => {
                self.backspace_active_field();
            }
            KeyCode::Enter => {
                self.submit_add_tenant_form();
            }
            KeyCode::Char(ch) => {
                self.push_active_field_char(ch);
            }
            _ => {}
        }
    }

    fn next_add_field(field: AddTenantField) -> AddTenantField {
        match field {
            AddTenantField::TenantName => AddTenantField::TrafficPattern,
            AddTenantField::TrafficPattern => AddTenantField::RequestsPerSecond,
            AddTenantField::RequestsPerSecond => AddTenantField::PayloadSizeBytes,
            AddTenantField::PayloadSizeBytes => AddTenantField::Priority,
            AddTenantField::Priority => AddTenantField::DurationSeconds,
            AddTenantField::DurationSeconds => AddTenantField::TenantName,
        }
    }

    fn previous_add_field(field: AddTenantField) -> AddTenantField {
        match field {
            AddTenantField::TenantName => AddTenantField::DurationSeconds,
            AddTenantField::TrafficPattern => AddTenantField::TenantName,
            AddTenantField::RequestsPerSecond => AddTenantField::TrafficPattern,
            AddTenantField::PayloadSizeBytes => AddTenantField::RequestsPerSecond,
            AddTenantField::Priority => AddTenantField::PayloadSizeBytes,
            AddTenantField::DurationSeconds => AddTenantField::Priority,
        }
    }

    fn next_target_field(field: TargetField) -> TargetField {
        match field {
            TargetField::Enabled => TargetField::System,
            TargetField::System => TargetField::Protocol,
            TargetField::Protocol => TargetField::Endpoint,
            TargetField::Endpoint => TargetField::HttpPath,
            TargetField::HttpPath => TargetField::Enabled,
        }
    }

    fn previous_target_field(field: TargetField) -> TargetField {
        match field {
            TargetField::Enabled => TargetField::HttpPath,
            TargetField::System => TargetField::Enabled,
            TargetField::Protocol => TargetField::System,
            TargetField::Endpoint => TargetField::Protocol,
            TargetField::HttpPath => TargetField::Endpoint,
        }
    }

    fn adjust_target_field(&mut self, forward: bool) {
        match self.target_field {
            TargetField::Enabled => {
                self.target_config.enabled = !self.target_config.enabled;
            }
            TargetField::System => {
                self.target_config.system = if forward {
                    self.target_config.system.next()
                } else {
                    self.target_config.system.previous()
                };
                self.apply_target_preset();
            }
            TargetField::Protocol => {
                self.target_config.protocol = if forward {
                    self.target_config.protocol.next()
                } else {
                    self.target_config.protocol.previous()
                };
                self.apply_target_preset();
            }
            TargetField::Endpoint | TargetField::HttpPath => {}
        }
    }

    fn push_target_field_char(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }

        match self.target_field {
            TargetField::Endpoint => self.target_config.endpoint.push(ch),
            TargetField::HttpPath => self.target_config.http_path.push(ch),
            TargetField::Enabled | TargetField::System | TargetField::Protocol => {}
        }
    }

    fn backspace_target_field(&mut self) {
        match self.target_field {
            TargetField::Endpoint => {
                self.target_config.endpoint.pop();
            }
            TargetField::HttpPath => {
                self.target_config.http_path.pop();
            }
            TargetField::Enabled | TargetField::System | TargetField::Protocol => {}
        }
    }

    fn apply_target_preset(&mut self) {
        let system: TargetSystem = self.target_config.system;
        let protocol = self.target_config.protocol;
        self.target_config.endpoint = system.default_endpoint(protocol).to_string();
        self.target_config.http_path = system.default_http_path().to_string();
    }

    fn rotate_pattern_forward(&mut self) {
        let len = Self::pattern_options().len();
        self.add_tenant_form.traffic_pattern_index =
            (self.add_tenant_form.traffic_pattern_index + 1) % len;
        self.add_tenant_form.validation_error = None;
    }

    fn rotate_pattern_backward(&mut self) {
        let len = Self::pattern_options().len();
        self.add_tenant_form.traffic_pattern_index =
            (self.add_tenant_form.traffic_pattern_index + len - 1) % len;
        self.add_tenant_form.validation_error = None;
    }

    fn pattern_options() -> [TrafficPattern; 3] {
        [
            TrafficPattern::Steady,
            TrafficPattern::Burst,
            TrafficPattern::Random,
        ]
    }

    fn push_active_field_char(&mut self, ch: char) {
        match self.add_tenant_form.active_field {
            AddTenantField::TenantName => {
                if !ch.is_control() {
                    self.add_tenant_form.tenant_name.push(ch);
                }
            }
            AddTenantField::TrafficPattern => {
                if matches!(ch, ' ' | 'l' | 'L') {
                    self.rotate_pattern_forward();
                } else if matches!(ch, 'h' | 'H') {
                    self.rotate_pattern_backward();
                }
            }
            AddTenantField::RequestsPerSecond => {
                Self::push_digit(&mut self.add_tenant_form.requests_per_second, ch)
            }
            AddTenantField::PayloadSizeBytes => {
                Self::push_digit(&mut self.add_tenant_form.payload_size_bytes, ch)
            }
            AddTenantField::Priority => Self::push_digit(&mut self.add_tenant_form.priority, ch),
            AddTenantField::DurationSeconds => {
                Self::push_digit(&mut self.add_tenant_form.duration_seconds, ch)
            }
        }
        self.add_tenant_form.validation_error = None;
    }

    fn push_digit(target: &mut String, ch: char) {
        if ch.is_ascii_digit() {
            target.push(ch);
        }
    }

    fn backspace_active_field(&mut self) {
        match self.add_tenant_form.active_field {
            AddTenantField::TenantName => {
                self.add_tenant_form.tenant_name.pop();
            }
            AddTenantField::RequestsPerSecond => {
                self.add_tenant_form.requests_per_second.pop();
            }
            AddTenantField::PayloadSizeBytes => {
                self.add_tenant_form.payload_size_bytes.pop();
            }
            AddTenantField::Priority => {
                self.add_tenant_form.priority.pop();
            }
            AddTenantField::DurationSeconds => {
                self.add_tenant_form.duration_seconds.pop();
            }
            AddTenantField::TrafficPattern => {}
        }
        self.add_tenant_form.validation_error = None;
    }

    fn traffic_pattern_to_index(pattern: TrafficPattern) -> usize {
        match pattern {
            TrafficPattern::Steady => 0,
            TrafficPattern::Burst => 1,
            TrafficPattern::Random => 2,
        }
    }

    pub fn ensure_generated_event_visible(&mut self, viewport_rows: usize) {
        if self.generated_events.is_empty() {
            self.generated_event_scroll_offset = 0;
            self.selected_generated_event = 0;
            return;
        }

        let rows = viewport_rows.max(1);

        if self.selected_generated_event < self.generated_event_scroll_offset {
            self.generated_event_scroll_offset = self.selected_generated_event;
            return;
        }

        if self.selected_generated_event >= self.generated_event_scroll_offset + rows {
            self.generated_event_scroll_offset = self.selected_generated_event + 1 - rows;
        }

        let max_start = self.generated_events.len().saturating_sub(rows);
        self.generated_event_scroll_offset = self.generated_event_scroll_offset.min(max_start);
    }

    fn parse_positive_u32(value: &str, label: &str) -> Result<u32, String> {
        let parsed = value
            .parse::<u32>()
            .map_err(|_| format!("{} must be a valid positive number", label))?;
        if parsed == 0 {
            return Err(format!("{} must be greater than 0", label));
        }
        Ok(parsed)
    }

    fn submit_add_tenant_form(&mut self) {
        let name = self.add_tenant_form.tenant_name.trim().to_string();
        if name.is_empty() {
            self.add_tenant_form.validation_error = Some("tenant_name cannot be empty".to_string());
            return;
        }

        let requests_per_second = match Self::parse_positive_u32(
            &self.add_tenant_form.requests_per_second,
            "requests_per_second",
        ) {
            Ok(value) => value,
            Err(error) => {
                self.add_tenant_form.validation_error = Some(error);
                return;
            }
        };

        let payload_size_bytes = match Self::parse_positive_u32(
            &self.add_tenant_form.payload_size_bytes,
            "payload_size_bytes",
        ) {
            Ok(value) => value,
            Err(error) => {
                self.add_tenant_form.validation_error = Some(error);
                return;
            }
        };

        let priority = match self.add_tenant_form.priority.parse::<u8>() {
            Ok(value) if (1..=255).contains(&value) => value,
            _ => {
                self.add_tenant_form.validation_error =
                    Some("priority must be between 1 and 255".to_string());
                return;
            }
        };

        let duration_seconds = match Self::parse_positive_u32(
            &self.add_tenant_form.duration_seconds,
            "duration_seconds",
        ) {
            Ok(value) => value,
            Err(error) => {
                self.add_tenant_form.validation_error = Some(error);
                return;
            }
        };

        let pattern = Self::pattern_options()[self.add_tenant_form.traffic_pattern_index];

        match &self.add_tenant_form.mode {
            TenantFormMode::Add => {
                let tenant_id = Uuid::new_v4().to_string();
                let new_tenant = TenantProfile::new(
                    &tenant_id,
                    &name,
                    pattern,
                    requests_per_second,
                    payload_size_bytes,
                    priority,
                    duration_seconds,
                );

                self.tenants.push(new_tenant);
                self.selected_tenant = self.tenants.len().saturating_sub(1);
                self.ensure_selected_visible(10);
                self.current_screen = Screen::Tenants;
                self.status_message = format!("Added tenant {}", name);
            }
            TenantFormMode::Edit { index, tenant_id } => {
                if *index >= self.tenants.len() {
                    self.add_tenant_form.validation_error =
                        Some("selected tenant no longer exists".to_string());
                    return;
                }

                self.tenants[*index] = TenantProfile::new(
                    tenant_id,
                    &name,
                    pattern,
                    requests_per_second,
                    payload_size_bytes,
                    priority,
                    duration_seconds,
                );

                self.selected_tenant = *index;
                self.ensure_selected_visible(10);
                self.current_screen = Screen::Tenants;
                self.status_message = format!("Updated tenant {}", name);
            }
        }
    }

    pub fn ensure_selected_visible(&mut self, viewport_rows: usize) {
        if self.tenants.is_empty() {
            self.tenant_scroll_offset = 0;
            return;
        }

        let rows = viewport_rows.max(1);

        if self.selected_tenant < self.tenant_scroll_offset {
            self.tenant_scroll_offset = self.selected_tenant;
            return;
        }

        if self.selected_tenant >= self.tenant_scroll_offset + rows {
            self.tenant_scroll_offset = self.selected_tenant + 1 - rows;
        }

        let max_start = self.tenants.len().saturating_sub(rows);
        self.tenant_scroll_offset = self.tenant_scroll_offset.min(max_start);
    }

    pub fn active_pattern_label(&self) -> &'static str {
        match Self::pattern_options()[self.add_tenant_form.traffic_pattern_index] {
            TrafficPattern::Steady => "Steady",
            TrafficPattern::Burst => "Burst",
            TrafficPattern::Random => "Random",
        }
    }

    pub fn take_generate_export_request(&mut self) -> bool {
        let requested = self.pending_generate_export;
        self.pending_generate_export = false;
        requested
    }

    pub fn set_generation_result(&mut self, events: Vec<TrafficEvent>, output_path: &str) {
        self.last_generated_events = events.len();
        self.generated_events = events.clone();
        self.last_generated_preview = events.iter().take(5).cloned().collect();
        self.last_generated_output_path = output_path.to_string();
        self.generation_error = None;
        self.selected_generated_event = 0;
        self.generated_event_scroll_offset = 0;
        self.set_active_screen(Screen::GeneratedEvents);
        self.status_message = format!(
            "Generated {} events -> {}",
            self.last_generated_events, self.last_generated_output_path
        );
    }

    pub fn set_generation_error(&mut self, message: String) {
        self.generation_error = Some(message.clone());
        self.status_message = format!("Generation failed: {}", message);
    }

    pub fn set_send_report(
        &mut self,
        attempted: usize,
        succeeded: usize,
        failed: usize,
        last_error: Option<String>,
    ) {
        self.send_preview.attempted = attempted;
        self.send_preview.succeeded = succeeded;
        self.send_preview.failed = failed;
        self.send_preview.last_error = last_error;
    }

    pub fn active_protocol_label(&self) -> &'static str {
        self.target_config.protocol.as_str()
    }

    pub fn active_target_system_label(&self) -> &'static str {
        self.target_config.system.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn generated_event_selection_scrolls_when_selection_moves_past_viewport() {
        let mut app = App::default();
        app.generated_events = (0..20)
            .map(|index| TrafficEvent::new(1000 + index, "tenant", index + 1, 512, "Generic"))
            .collect();
        app.selected_generated_event = 15;

        app.ensure_generated_event_visible(5);

        assert_eq!(app.generated_event_scroll_offset, 11);
        assert_eq!(app.selected_generated_event, 15);
    }

    #[test]
    fn preview_scrolls_with_navigation_keys() {
        let mut app = App::default();
        app.current_screen = Screen::Preview;

        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));

        assert_eq!(app.preview_scroll_offset, 8);
    }
}
