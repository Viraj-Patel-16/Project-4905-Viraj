use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use super::app::{App, Screen};

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
        Screen::Preview => render_preview(frame, area, app),
        Screen::Help => render_help(frame, area),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let footer = Paragraph::new(format!(
        "Keys: [1][2][3][h] jump  [Tab/Shift+Tab] focus tabs  [Enter] open tab  [Up/Down or j/k] tenants  [q/Ctrl+C] quit | Status: {}",
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

    let items: Vec<ListItem> = app
        .tenants
        .iter()
        .enumerate()
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
                    " | id={} | pattern={:?} | rate={} req/s | payload={} bytes | priority={}",
                    tenant.tenant_id,
                    tenant.traffic_pattern,
                    tenant.requests_per_second,
                    tenant.payload_size_bytes,
                    tenant.priority
                )),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Tenant Profiles"),
    );

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
            Line::from("Tip: Use Up/Down or j/k to inspect profiles quickly."),
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

fn render_preview(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.tenants.get(app.selected_tenant);

    let text = if let Some(tenant) = selected {
        vec![
            Line::from("Traffic preview will be connected to the generator in the next milestone."),
            Line::from(""),
            Line::from(format!("Selected tenant: {}", tenant.tenant_name)),
            Line::from(format!("Pattern: {:?}", tenant.traffic_pattern)),
            Line::from(format!(
                "Rate: {} requests/second",
                tenant.requests_per_second
            )),
            Line::from(format!("Payload size: {} bytes", tenant.payload_size_bytes)),
            Line::from(""),
            Line::from("Next step: generate sample TrafficEvent values from this profile."),
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
        Line::from("↑ / ↓   Move through tenant list"),
        Line::from(""),
        Line::from("Current scope: TUI framework with screen navigation."),
    ];

    let widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}
