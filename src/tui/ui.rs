use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use super::app::{AddTenantField, App, Screen, TenantFormMode};

pub fn render(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, layout[0], app);
    render_body(frame, layout[1], app);
    render_footer(frame, layout[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let screen_title = match app.current_screen {
        Screen::Dashboard => "Dashboard",
        Screen::Tenants => "Tenants",
        Screen::AddTenant => "Add Tenant",
        Screen::Preview => "Traffic Preview",
        Screen::Help => "Help",
    };

    let tabs = [
        (Screen::Dashboard, "Dashboard"),
        (Screen::Tenants, "Tenants"),
        (Screen::Preview, "Preview"),
        (Screen::Help, "Help"),
    ]
    .into_iter()
    .map(|(screen, label)| {
        let mut style = Style::default();
        if screen == app.current_screen {
            style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
        } else if screen == app.focused_screen {
            style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
        }
        Span::styled(format!("[{}]", label), style)
    })
    .collect::<Vec<_>>();

    let mut header_line = vec![Span::styled(
        format!("Rust TUI Traffic Producer - {}  ", screen_title),
        Style::default().add_modifier(Modifier::BOLD),
    )];

    for (idx, tab) in tabs.into_iter().enumerate() {
        header_line.push(tab);
        if idx < 3 {
            header_line.push(Span::raw(" "));
        }
    }

    let header = Paragraph::new(Line::from(header_line))
        .block(Block::default().borders(Borders::ALL).title("COMP 4905"));

    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    match app.current_screen {
        Screen::Dashboard => render_dashboard(frame, area, app),
        Screen::Tenants => render_tenants(frame, area, app),
        Screen::AddTenant => render_add_tenant(frame, area, app),
        Screen::Preview => render_preview(frame, area, app),
        Screen::Help => render_help(frame, area),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let footer = Paragraph::new(format!(
        "Keys: [1][2][3][h] jump  [a] add tenant  [e] edit tenant  [d] delete tenant  [Up/Down, j/k, PgUp/PgDn] list nav  [q/Ctrl+C] quit | Status: {}",
        app.status_message
    ))
    .block(Block::default().borders(Borders::ALL).title("Controls"));

    frame.render_widget(footer, area);
}

fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let text = vec![
        Line::from("Project: Rust TUI Traffic Producer"),
        Line::from(""),
        Line::from(format!("Configured tenants: {}", app.tenants.len())),
        Line::from(""),
        Line::from("Purpose: define tenant profiles and generate traffic events."),
        Line::from("Use screen [2] to view tenant profiles."),
        Line::from("Use screen [3] to preview generated traffic later."),
    ];

    let widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Overview"))
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}

fn render_tenants(frame: &mut Frame, area: Rect, app: &App) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    let visible_rows = sections[0].height.saturating_sub(2) as usize;
    let max_start = app.tenants.len().saturating_sub(visible_rows.max(1));
    let start = app.tenant_scroll_offset.min(max_start);
    let end = (start + visible_rows.max(1)).min(app.tenants.len());

    let items: Vec<ListItem> = app
        .tenants
        .iter()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
        .map(|(index, tenant)| {
            let marker = if index == app.selected_tenant {
                ">"
            } else {
                " "
            };

            let line = Line::from(vec![
                Span::raw(format!("{} ", marker)),
                Span::styled(
                    tenant.tenant_name.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    " | pattern={:?} | rate={} req/s",
                    tenant.traffic_pattern, tenant.requests_per_second
                )),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
        "Tenant Profiles ({}/{})",
        app.selected_tenant.saturating_add(1),
        app.tenants.len()
    )));

    frame.render_widget(list, sections[0]);

    let details = if let Some(tenant) = app.tenants.get(app.selected_tenant) {
        vec![
            Line::from(Span::styled(
                tenant.tenant_name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Tenant ID: {}", tenant.tenant_id)),
            Line::from(format!("Pattern: {:?}", tenant.traffic_pattern)),
            Line::from(format!("Rate: {} req/s", tenant.requests_per_second)),
            Line::from(format!("Payload: {} bytes", tenant.payload_size_bytes)),
            Line::from(format!("Priority: {}", tenant.priority)),
            Line::from(format!("Duration: {} s", tenant.duration_seconds)),
            Line::from(""),
            Line::from("Tip: [a] add tenant  [d] delete selected tenant"),
            Line::from("Tip: Use PgUp/PgDn for faster scrolling."),
        ]
    } else {
        vec![Line::from("No tenant selected.")]
    };

    let details_widget = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Selected Tenant"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(details_widget, sections[1]);
}

fn render_add_tenant(frame: &mut Frame, area: Rect, app: &App) {
    let form = &app.add_tenant_form;
    let (mode_title, action_label) = match form.mode {
        TenantFormMode::Add => ("Create Tenant Profile", "save tenant"),
        TenantFormMode::Edit { .. } => ("Edit Tenant Profile", "save changes"),
    };

    let field = |label: &str, value: String, is_active: bool| {
        if is_active {
            Line::from(vec![
                Span::styled("-> ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}: {}", label, value),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(format!("   {}: {}", label, value))
        }
    };

    let text = vec![
        Line::from(Span::styled(
            mode_title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "tenant_id (auto-generated): {}",
            form.tenant_id_preview
        )),
        Line::from(""),
        field(
            "tenant_name",
            form.tenant_name.clone(),
            form.active_field == AddTenantField::TenantName,
        ),
        field(
            "traffic_pattern",
            app.active_pattern_label().to_string(),
            form.active_field == AddTenantField::TrafficPattern,
        ),
        field(
            "requests_per_second",
            form.requests_per_second.clone(),
            form.active_field == AddTenantField::RequestsPerSecond,
        ),
        field(
            "payload_size_bytes",
            form.payload_size_bytes.clone(),
            form.active_field == AddTenantField::PayloadSizeBytes,
        ),
        field(
            "priority (1-255)",
            form.priority.clone(),
            form.active_field == AddTenantField::Priority,
        ),
        field(
            "duration_seconds",
            form.duration_seconds.clone(),
            form.active_field == AddTenantField::DurationSeconds,
        ),
        Line::from(""),
        Line::from(match &form.validation_error {
            Some(error) => format!("Validation: {}", error),
            None => "Validation: ready".to_string(),
        }),
        Line::from(""),
        Line::from("Controls: Tab/Shift+Tab move fields | Type to edit | Left/Right on pattern"),
        Line::from(format!("Controls: Enter {} | Esc cancel", action_label)),
    ];

    let widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match form.mode {
                    TenantFormMode::Add => "Add Tenant",
                    TenantFormMode::Edit { .. } => "Edit Tenant",
                }),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}

fn render_preview(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.tenants.get(app.selected_tenant);

    let text = if let Some(tenant) = selected {
        vec![
            Line::from("Live preview mode submits synthetic tasks to a background worker."),
            Line::from(""),
            Line::from(format!("Selected tenant: {}", tenant.tenant_name)),
            Line::from(format!("Pattern: {:?}", tenant.traffic_pattern)),
            Line::from(format!(
                "Rate: {} requests/second",
                tenant.requests_per_second
            )),
            Line::from(format!("Payload size: {} bytes", tenant.payload_size_bytes)),
            Line::from(""),
            Line::from(Span::styled(
                "Worker Runtime",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("Worker ID: {}", app.worker_preview.worker_id)),
            Line::from(format!("Current load: {}", app.worker_preview.current_load)),
            Line::from(format!("is_free: {}", app.worker_preview.is_free)),
            Line::from(format!("is_busy: {}", app.worker_preview.is_busy)),
            Line::from(format!(
                "Submitted tasks: {}",
                app.worker_preview.submitted_tasks
            )),
            Line::from(format!(
                "Processed tasks: {}",
                app.worker_preview.processed_tasks
            )),
            Line::from(match &app.worker_preview.last_error {
                Some(error) => format!("Last submit error: {}", error),
                None => "Last submit error: none".to_string(),
            }),
            Line::from(""),
            Line::from(
                "Tip: switch tenants in [Tenants] screen, then return here to observe load.",
            ),
        ]
    } else {
        vec![Line::from("No tenant selected.")]
    };

    let widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Traffic Preview"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from("Navigation"),
        Line::from(""),
        Line::from("1       Go to Dashboard"),
        Line::from("2       Go to Tenants"),
        Line::from("3       Go to Traffic Preview"),
        Line::from("h       Go to Help"),
        Line::from("q       Quit application"),
        Line::from("Ctrl+C  Quit application"),
        Line::from("↑ / ↓   Move through tenant list"),
        Line::from("a       Open Add Tenant form"),
        Line::from("e       Open Edit Tenant form"),
        Line::from("d       Delete selected tenant"),
        Line::from("PgUp/PgDn  Scroll tenant list faster"),
        Line::from("Tab     Focus next screen tab"),
        Line::from("Enter   Open focused tab"),
        Line::from(""),
        Line::from("Traffic Preview scope: live worker telemetry and task submission."),
    ];

    let widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}
