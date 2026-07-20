use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::model::{TenantProfile, TrafficPattern};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Tenants,
    AddTenant,
    Preview,
    Help,
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
    pub tenant_scroll_offset: usize,
    pub add_tenant_form: AddTenantForm,
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
            worker_preview: WorkerPreviewState::default(),
            tenant_scroll_offset: 0,
            add_tenant_form: AddTenantForm::default(),
        }
    }
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.current_screen == Screen::AddTenant {
            self.handle_add_tenant_key(key);
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
            Screen::Preview => Screen::Help,
            Screen::Help => Screen::Dashboard,
        }
    }

    fn previous_screen(screen: Screen) -> Screen {
        match screen {
            Screen::Dashboard => Screen::Help,
            Screen::Tenants => Screen::Dashboard,
            Screen::AddTenant => Screen::Tenants,
            Screen::Preview => Screen::Tenants,
            Screen::Help => Screen::Preview,
        }
    }

    pub fn focused_screen_name(&self) -> &'static str {
        match self.focused_screen {
            Screen::Dashboard => "Dashboard",
            Screen::Tenants => "Tenants",
            Screen::AddTenant => "Add Tenant",
            Screen::Preview => "Traffic Preview",
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
            TrafficPattern::Bursty,
            TrafficPattern::Heavy,
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
            TrafficPattern::Bursty => 1,
            TrafficPattern::Heavy => 2,
        }
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
            TrafficPattern::Bursty => "Bursty",
            TrafficPattern::Heavy => "Heavy",
        }
    }
}
