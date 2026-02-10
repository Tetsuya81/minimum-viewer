use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::{App, Mode};

const MAX_PATH_WIDTH: u16 = 80;
const CMD_CANDIDATE_ROWS: u16 = 6;
const INPUT_ROWS: u16 = 3;
const SHELL_PANEL_ROWS: u16 = 6;
const STATUS_BAR_COLLAPSED_ROWS: u16 = 3;
const STATUS_BAR_EXPANDED_ROWS: u16 = 6;

fn truncate_to_width(s: &str, width: u16) -> String {
    let w = width as usize;
    let char_count = s.chars().count();
    if char_count <= w {
        return s.to_string();
    }
    if w == 0 {
        return String::new();
    }
    if w == 1 {
        return "…".to_string();
    }

    let tail_len = w - 1;
    let tail: String = s
        .chars()
        .rev()
        .take(tail_len)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{}", tail)
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = area.width;

    let constraints = match app.mode {
        Mode::Command => vec![
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(INPUT_ROWS),
            Constraint::Length(CMD_CANDIDATE_ROWS),
        ],
        Mode::Shell => vec![
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(INPUT_ROWS),
            Constraint::Length(SHELL_PANEL_ROWS),
        ],
        Mode::Filter => vec![
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(INPUT_ROWS),
        ],
        Mode::Browse => vec![
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(if app.status_bar_expanded {
                STATUS_BAR_EXPANDED_ROWS
            } else {
                STATUS_BAR_COLLAPSED_ROWS
            }),
        ],
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let path_width = width.saturating_sub(4).min(MAX_PATH_WIDTH);
    let path_display = truncate_to_width(
        format!("📁 {}", app.current_dir.to_string_lossy()).as_str(),
        path_width,
    );
    let path_block = Block::default()
        .title(Line::from(" current directory "))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));
    let path_para = Paragraph::new(path_display)
        .block(path_block)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
    frame.render_widget(path_para, chunks[0]);

    let visible_children = app.entries.iter().filter(|e| e.name != "..").count();
    let total_children = app.all_entries.iter().filter(|e| e.name != "..").count();
    let list_title = format!(" files ({}/{}) ", visible_children, total_children);
    let list_block = Block::default()
        .title(Line::from(list_title))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let icon = if e.is_dir { "📁 " } else { "📄 " };
            let size_str = e
                .size
                .map(|s| format!(" {:>8}", human_size(s)))
                .unwrap_or_default();
            let indent = if e.name == ".." { "" } else { "  " };
            let name = truncate_to_width(&e.name, width.saturating_sub(8));
            let line = format!("{}{}{}{}", indent, icon, name, size_str);
            let style =
                if i == app.selected_index && matches!(app.mode, Mode::Browse | Mode::Filter) {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
            ListItem::new(line).style(style)
        })
        .collect();
    let list = List::new(items)
        .block(list_block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");
    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    if app.mode == Mode::Command {
        let cmd_line = format!(":{}", app.command_input);
        let cmd_block = Block::default()
            .title(Line::from(" command (:): Enter run Esc cancel "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::Yellow));
        let cmd_para = Paragraph::new(cmd_line)
            .block(cmd_block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(cmd_para, chunks[2]);

        let cand_block = Block::default()
            .title(Line::from(" commands "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::DarkGray));
        let cand_items: Vec<ListItem> = app
            .command_candidates
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == app.command_selected {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(s.as_str()).style(style)
            })
            .collect();
        let cand_list = List::new(cand_items)
            .block(cand_block)
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
            .highlight_symbol("▸ ");
        let mut cand_state = ListState::default();
        cand_state.select(Some(app.command_selected));
        frame.render_stateful_widget(cand_list, chunks[3], &mut cand_state);
    } else if app.mode == Mode::Shell {
        let shell_line = format!("!{}", app.shell_input);
        let shell_block = Block::default()
            .title(Line::from(" shell (!): Enter run Esc cancel "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::Yellow));
        let shell_para = Paragraph::new(shell_line)
            .block(shell_block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(shell_para, chunks[2]);

        let panel_block = Block::default()
            .title(Line::from(" shell "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::DarkGray));
        let panel_body = "Enter: run shell command\nEsc: cancel";
        let panel_para = Paragraph::new(panel_body)
            .block(panel_block)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false });
        frame.render_widget(panel_para, chunks[3]);
    } else if app.mode == Mode::Filter {
        let filter_line = format!("/{}", app.filter_input);
        let filter_block = Block::default()
            .title(Line::from(" filter (/): Enter apply Esc clear "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::Yellow));
        let filter_para = Paragraph::new(filter_line)
            .block(filter_block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(filter_para, chunks[2]);
    } else {
        let status = if !app.status_message.is_empty() {
            app.status_message.clone()
        } else {
            app.selected_entry()
                .map(|e| format_status_bar(e, width.saturating_sub(4), app.status_bar_expanded))
                .unwrap_or_default()
        };
        let block = Block::default()
            .title(Line::from(" status "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::DarkGray));
        let para = Paragraph::new(status)
            .block(block)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false });
        frame.render_widget(para, chunks[2]);
    }

    if app.show_shell_popup {
        if let Some(result) = &app.shell_last_output {
            let exit_text = result
                .exit_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal/error".to_string());
            let title = format!(" shell output: {} (exit {}) ", app.shell_input, exit_text);
            let output = format!(
                "[shell]\n{}\n\n[stdout]\n{}\n\n[stderr]\n{}",
                result.ran_shell, result.stdout, result.stderr
            );
            let popup_area = centered_rect(90, 80, area);
            frame.render_widget(Clear, popup_area);
            let para = Paragraph::new(output)
                .block(
                    Block::default()
                        .title(Line::from(title))
                        .borders(Borders::ALL)
                        .border_set(symbols::border::ROUNDED)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .wrap(Wrap { trim: false });
            frame.render_widget(para, popup_area);
        }
    }
}

fn human_size(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if n >= GB {
        format!("{:.1}G", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.1}M", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1}K", n as f64 / KB as f64)
    } else {
        format!("{}B", n)
    }
}

fn format_status_bar(entry: &crate::app::DirEntry, content_width: u16, expanded: bool) -> String {
    let size = entry
        .size
        .map(human_size)
        .unwrap_or_else(|| "-".to_string());
    let modified = format_modified(entry.modified);
    let perm = entry
        .permissions
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let owner = entry.owner.clone().unwrap_or_else(|| "-".to_string());
    let group = entry.group.clone().unwrap_or_else(|| "-".to_string());

    if !expanded {
        return format_status_row(
            &[("Size", size), ("Modified", modified)],
            content_width,
        );
    }

    if content_width >= 90 {
        return format_status_row(
            &[
                ("Size", size),
                ("Modified", modified),
                ("Perm", perm),
                ("Owner", owner),
                ("Group", group),
            ],
            content_width,
        );
    }

    if content_width >= 50 {
        return [
            format_status_row(&[("Size", size), ("Modified", modified)], content_width),
            format_status_row(
                &[("Perm", perm), ("Owner", owner), ("Group", group)],
                content_width,
            ),
        ]
        .join("\n");
    }

    [
        format_status_line("Size", &size, content_width),
        format_status_line("Modified", &modified, content_width),
        format_status_line("Perm", &perm, content_width),
        format_status_line("Owner", &owner, content_width),
        format_status_line("Group", &group, content_width),
    ]
    .join("\n")
}

fn format_status_row(items: &[(&str, String)], width: u16) -> String {
    if items.is_empty() {
        return String::new();
    }
    let sep = " | ";
    let sep_chars = sep.chars().count();
    let separator_width = sep_chars.saturating_mul(items.len().saturating_sub(1));
    let available = (width as usize).saturating_sub(separator_width);
    let each = if items.is_empty() {
        0
    } else {
        available / items.len()
    } as u16;
    items
        .iter()
        .map(|(label, value)| format_status_line(label, value, each))
        .collect::<Vec<_>>()
        .join(sep)
}

fn format_status_line(label: &str, value: &str, width: u16) -> String {
    let prefix = format!("{}: ", label);
    let prefix_len = prefix.chars().count();
    let width_len = width as usize;
    if width_len <= prefix_len {
        return truncate_to_width(&prefix, width);
    }
    let max_value_width = (width_len - prefix_len) as u16;
    format!("{}{}", prefix, truncate_to_width(value, max_value_width))
}

fn format_modified(modified: Option<SystemTime>) -> String {
    let Some(time) = modified else {
        return "-".to_string();
    };
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "-".to_string();
    };
    let secs = duration.as_secs() as i64;
    format_unix_utc(secs)
}

fn format_unix_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let day_seconds = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year, month, day, hour, minute
    )
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m, d)
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn unix_epoch_formats_as_expected() {
        assert_eq!(format_unix_utc(0), "1970-01-01 00:00");
    }

    #[test]
    fn status_bar_collapsed_contains_only_size_and_modified() {
        let entry = crate::app::DirEntry {
            name: "example.txt".to_string(),
            path: PathBuf::from("example.txt"),
            is_dir: false,
            size: Some(1234),
            modified: Some(UNIX_EPOCH),
            permissions: Some("rw-r--r--".to_string()),
            owner: Some("alice".to_string()),
            group: Some("staff".to_string()),
        };

        let status = format_status_bar(&entry, 80, false);
        assert!(status.contains("Size: 1.2K"));
        assert!(status.contains("Modified: 1970-01-01 00:00"));
        assert!(!status.contains("Perm:"));
        assert!(!status.contains("Owner:"));
        assert!(!status.contains("Group:"));
    }

    #[test]
    fn status_bar_expanded_contains_permission_owner_group() {
        let entry = crate::app::DirEntry {
            name: "example.txt".to_string(),
            path: PathBuf::from("example.txt"),
            is_dir: false,
            size: Some(1234),
            modified: Some(UNIX_EPOCH),
            permissions: Some("rw-r--r--".to_string()),
            owner: Some("alice".to_string()),
            group: Some("staff".to_string()),
        };

        let status = format_status_bar(&entry, 80, true);

        assert!(status.contains("Size: 1.2K"));
        assert!(status.contains("Modified: 1970-01-01 00:00"));
        assert!(status.contains("Perm: rw-r--r--"));
        assert!(status.contains("Owner: alice"));
        assert!(status.contains("Group: staff"));
    }

    #[test]
    fn status_bar_expanded_small_width_breaks_into_multiple_lines() {
        let entry = crate::app::DirEntry {
            name: "example.txt".to_string(),
            path: PathBuf::from("example.txt"),
            is_dir: false,
            size: Some(1234),
            modified: Some(UNIX_EPOCH),
            permissions: Some("rw-r--r--".to_string()),
            owner: Some("alice".to_string()),
            group: Some("staff".to_string()),
        };

        let status = format_status_bar(&entry, 30, true);
        let lines: Vec<&str> = status.lines().collect();
        assert_eq!(lines.len(), 5);
    }
}
