use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
#[cfg(unix)]
use std::os::raw::{c_int, c_long};
#[cfg(unix)]
use std::ptr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::{App, Mode};
use crate::command;

const ICON_FOLDER: &str = "\u{f07b}";
const ICON_FILE: &str = "\u{f15b}";
const ICON_COLOR_FOLDER: Color = Color::Yellow;
const ICON_COLOR_FILE: Color = Color::White;
const ICON_COLOR_SELECTED: Color = Color::Black;

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

    let head: String = s.chars().take(w - 1).collect();
    format!("{}…", head)
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let width = area.width;

    let constraints = match app.mode {
        Mode::Command => vec![
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(CMD_CANDIDATE_ROWS),
            Constraint::Length(INPUT_ROWS),
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
        Mode::Browse | Mode::Create | Mode::Help => vec![
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
        format!("{} {}", ICON_FOLDER, app.current_dir.to_string_lossy()).as_str(),
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
    let items: Vec<ListItem> = if app.mode == Mode::Create {
        let mut result = Vec::new();
        for (i, e) in app.entries.iter().enumerate() {
            let icon_str = if e.is_dir { format!("{} ", ICON_FOLDER) } else { format!("{} ", ICON_FILE) };
            let icon_color = if e.is_dir { ICON_COLOR_FOLDER } else { ICON_COLOR_FILE };
            let indent = if e.name == ".." { "" } else { "  " };
            let link_suffix = e.link_target.as_ref().map(|t| format!("  >>  {}", t)).unwrap_or_default();
            let display_text = format!("{}{}", e.name, link_suffix);
            let name = truncate_to_width(&display_text, width.saturating_sub(8));
            let line = Line::from(vec![
                Span::styled(indent.to_string(), Style::default().fg(Color::Gray)),
                Span::styled(icon_str, Style::default().fg(icon_color)),
                Span::styled(name, Style::default().fg(Color::Gray)),
            ]);
            result.push(ListItem::new(line));
            if i == app.selected_index {
                let create_icon = if app.create_input.trim_start().starts_with('/')
                    || app.create_input.contains('/')
                {
                    format!("{} ", ICON_FOLDER)
                } else {
                    format!("{} ", ICON_FILE)
                };
                let display_name = app.create_input.as_str();
                let hint = " // `/`[Folder name] or [File name]";
                let prefix = format!("  {} ", create_icon);
                let line_width = (width.saturating_sub(12)) as usize;
                let hint_len = hint.chars().count();
                let prefix_len = prefix.chars().count();
                let available_for_input = line_width.saturating_sub(prefix_len).saturating_sub(hint_len).saturating_sub(2);
                let input_part = truncate_to_width(display_name, available_for_input as u16);
                let padding_len = line_width
                    .saturating_sub(prefix_len)
                    .saturating_sub(input_part.chars().count())
                    .saturating_sub(hint_len);
                let padding = " ".repeat(padding_len);
                let create_line = format!("{}{}{}{}", prefix, input_part, padding, hint);
                result.push(
                    ListItem::new(create_line)
                        .style(Style::default().fg(Color::Black).bg(Color::Yellow)),
                );
            }
        }
        result
    } else {
        app.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let icon_str = if e.is_dir { format!("{} ", ICON_FOLDER) } else { format!("{} ", ICON_FILE) };
                let indent = if e.name == ".." { "" } else { "  " };
                let link_suffix = e.link_target.as_ref().map(|t| format!("  >>  {}", t)).unwrap_or_default();
                let display_text = format!("{}{}", e.name, link_suffix);
                let name = truncate_to_width(&display_text, width.saturating_sub(8));

                let is_selected = i == app.selected_index && matches!(app.mode, Mode::Browse | Mode::Filter);

                let icon_style = if is_selected {
                    Style::default()
                        .fg(ICON_COLOR_SELECTED)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    let icon_color = if e.is_dir { ICON_COLOR_FOLDER } else { ICON_COLOR_FILE };
                    Style::default().fg(icon_color)
                };

                let text_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let line = Line::from(vec![
                    Span::styled(indent.to_string(), text_style),
                    Span::styled(icon_str, icon_style),
                    Span::styled(name, text_style),
                ]);
                ListItem::new(line)
            })
            .collect()
    };
    let list = List::new(items)
        .block(list_block)
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");
    let mut list_state = ListState::default();
    list_state.select(Some(if app.mode == Mode::Create {
        app.selected_index + 1
    } else {
        app.selected_index
    }));
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    if app.mode == Mode::Command {
        let cand_block = Block::default()
            .title(Line::from(" commands (Tab next / Shift+Tab prev) "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::DarkGray));
        let cand_items: Vec<ListItem> = app
            .command_candidates
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if app.command_selected == Some(i) {
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
        cand_state.select(app.command_selected);
        frame.render_stateful_widget(cand_list, chunks[2], &mut cand_state);

        let cmd_line = format!(":{}", app.command_input);
        let cmd_block = Block::default()
            .title(Line::from(
                " command (:): Enter run Esc cancel Tab select Shift+Tab reverse ",
            ))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::Yellow));
        let cmd_para = Paragraph::new(cmd_line)
            .block(cmd_block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(cmd_para, chunks[3]);

        let desired_cursor_x = chunks[3]
            .x
            .saturating_add(1)
            .saturating_add(1)
            .saturating_add(app.command_input.chars().count() as u16);
        let max_cursor_x = chunks[3]
            .x
            .saturating_add(chunks[3].width.saturating_sub(2))
            .saturating_sub(1);
        let cursor_x = desired_cursor_x.min(max_cursor_x);
        let cursor_y = chunks[3].y.saturating_add(1);
        frame.set_cursor_position((cursor_x, cursor_y));
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
    } else if app.mode == Mode::Create {
        let status = if !app.status_message.is_empty() {
            format!("{}\n\nEnter: create  Esc: cancel", app.status_message)
        } else {
            "Enter: create  Esc: cancel".to_string()
        };
        let block = Block::default()
            .title(Line::from(" create (n) "))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::DarkGray));
        let para = Paragraph::new(status)
            .block(block)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false });
        frame.render_widget(para, chunks[2]);

        let create_row_index = app.selected_index + 1;
        let visible_row = create_row_index.saturating_sub(list_state.offset());
        let list_inner_y = chunks[1].y + 1 + visible_row as u16;
        let prefix_len = 7; // "▸ " (2) + "  " (2) + "icon " (2) + offset fix
        let cursor_x = chunks[1]
            .x
            .saturating_add(1)
            .saturating_add(prefix_len)
            .saturating_add(app.create_input.chars().count() as u16);
        let max_cursor_x = chunks[1]
            .x
            .saturating_add(chunks[1].width)
            .saturating_sub(2);
        frame.set_cursor_position((cursor_x.min(max_cursor_x), list_inner_y));
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
            .title_bottom(
                Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).right_aligned(),
            )
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

    if app.mode == Mode::Help {
        draw_help_screen(frame, app, area);
    }

    if app.show_delete_confirm {
        let pending = app.pending_delete.as_ref();
        let target = pending
            .map(|p| p.path.to_string_lossy().to_string())
            .unwrap_or_else(|| "(unknown)".to_string());
        let is_dir = pending.map(|p| p.is_dir).unwrap_or(false);
        let question = if is_dir {
            "Delete this directory recursively?"
        } else {
            "Delete this file?"
        };
        let body = format!(
            "{}\n\n{}\n\nPress y to confirm, n (or Esc/Enter) to cancel.",
            question, target
        );
        let popup_area = centered_rect(70, 40, area);
        frame.render_widget(Clear, popup_area);
        let para = Paragraph::new(body)
            .block(
                Block::default()
                    .title(Line::from(" delete confirmation "))
                    .borders(Borders::ALL)
                    .border_set(symbols::border::ROUNDED)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false });
        frame.render_widget(para, popup_area);
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
    if entry.is_dangling {
        let target = entry
            .link_target
            .as_deref()
            .unwrap_or("(unknown)");
        return format!("Broken symlink: {} >> {}", entry.name, target);
    }

    let size = entry
        .size
        .map(human_size)
        .unwrap_or_else(|| "-".to_string());
    let modified = format_modified(entry.modified);
    let perm = entry.permissions.clone().unwrap_or_else(|| "-".to_string());
    let owner = entry.owner.clone().unwrap_or_else(|| "-".to_string());
    let group = entry.group.clone().unwrap_or_else(|| "-".to_string());

    if !expanded {
        return format_status_rows(&[("Modified", modified), ("Size", size)], content_width);
    }
    format_status_rows(
        &[
            ("Modified", modified),
            ("Size", size),
            ("Perm", perm),
            ("Owner", owner),
            ("Group", group),
        ],
        content_width,
    )
}

fn format_status_rows(items: &[(&str, String)], width: u16) -> String {
    if items.is_empty() {
        return String::new();
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let max = width as usize;

    for (label, value) in items {
        let segment = format!("{}: {}", label, value);
        if current.is_empty() {
            current = segment;
            continue;
        }
        let candidate = format!("{} | {}", current, segment);
        if max > 0 && candidate.chars().count() <= max {
            current = candidate;
        } else {
            lines.push(current);
            current = segment;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }

    lines.join("\n")
}

fn format_modified(modified: Option<SystemTime>) -> String {
    let Some(time) = modified else {
        return "-".to_string();
    };
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "-".to_string();
    };
    let secs = duration.as_secs() as i64;
    format_local_with_offset(secs)
}

#[cfg(unix)]
fn format_local_with_offset(secs: i64) -> String {
    let mut raw_time = secs as TimeT;
    let mut tm = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ptr::null(),
    };
    unsafe {
        if localtime_r(&mut raw_time as *mut TimeT as *const TimeT, &mut tm).is_null() {
            return format!("{} +00:00", format_unix_utc(secs));
        }
    }

    let year = tm.tm_year as i64 + 1900;
    let month = tm.tm_mon as i64 + 1;
    let day = tm.tm_mday as i64;
    let hour = tm.tm_hour as i64;
    let minute = tm.tm_min as i64;
    let offset = tm.tm_gmtoff;
    let sign = if offset >= 0 { '+' } else { '-' };
    let abs = offset.abs();
    let off_hour = abs / 3600;
    let off_min = (abs % 3600) / 60;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02} {}{:02}:{:02}",
        year, month, day, hour, minute, sign, off_hour, off_min
    )
}

#[cfg(not(unix))]
fn format_local_with_offset(secs: i64) -> String {
    format!("{} +00:00", format_unix_utc(secs))
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

#[cfg(unix)]
type TimeT = i64;

#[cfg(unix)]
#[repr(C)]
struct Tm {
    tm_sec: c_int,
    tm_min: c_int,
    tm_hour: c_int,
    tm_mday: c_int,
    tm_mon: c_int,
    tm_year: c_int,
    tm_wday: c_int,
    tm_yday: c_int,
    tm_isdst: c_int,
    tm_gmtoff: c_long,
    tm_zone: *const i8,
}

#[cfg(unix)]
unsafe extern "C" {
    fn localtime_r(timep: *const TimeT, result: *mut Tm) -> *mut Tm;
}

fn draw_help_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let items = command::help_items();
    let popup_area = centered_rect(80, 85, area);
    frame.render_widget(Clear, popup_area);

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| {
            let keys = item.keys_display.unwrap_or("");
            let line = Line::from(vec![
                Span::styled(
                    format!("{:>20}  ", keys),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("{:<10}  ", item.command_name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(item.description),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(Line::from(
                    " help  j/k: navigate  Enter: execute  Esc: close ",
                ))
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{f0da} ");

    frame.render_stateful_widget(list, popup_area, &mut app.help_list_state);
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
            uid: None,
            gid: None,
            owner: Some("alice".to_string()),
            group: Some("staff".to_string()),
            link_target: None,
            is_dangling: false,
        };

        let status = format_status_bar(&entry, 80, false);
        let modified_pos = status.find("Modified: ").expect("Modified must exist");
        let size_pos = status.find("Size: 1.2K").expect("Size must exist");
        assert!(modified_pos < size_pos);
        assert!(status.contains(" +"));
        assert!(!status.contains("…"));
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
            uid: None,
            gid: None,
            owner: Some("alice".to_string()),
            group: Some("staff".to_string()),
            link_target: None,
            is_dangling: false,
        };

        let status = format_status_bar(&entry, 80, true);

        let modified_pos = status.find("Modified: ").expect("Modified must exist");
        let size_pos = status.find("Size: 1.2K").expect("Size must exist");
        let perm_pos = status.find("Perm: rw-r--r--").expect("Perm must exist");
        let owner_pos = status.find("Owner: alice").expect("Owner must exist");
        let group_pos = status.find("Group: staff").expect("Group must exist");
        assert!(modified_pos < size_pos);
        assert!(size_pos < perm_pos);
        assert!(perm_pos < owner_pos);
        assert!(owner_pos < group_pos);
        assert!(status.contains("Perm: rw-r--r--"));
        assert!(status.contains("Owner: alice"));
        assert!(status.contains("Group: staff"));
        assert!(!status.contains("…"));
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
            uid: None,
            gid: None,
            owner: Some("alice".to_string()),
            group: Some("staff".to_string()),
            link_target: None,
            is_dangling: false,
        };

        let status = format_status_bar(&entry, 30, true);
        let lines: Vec<&str> = status.lines().collect();
        assert!(lines.len() >= 3);
    }

    #[test]
    fn modified_has_timezone_offset_format() {
        let rendered = format_modified(Some(UNIX_EPOCH));
        let bytes = rendered.as_bytes();
        assert_eq!(bytes.len(), 23);
        assert_eq!(bytes[4], b'-');
        assert_eq!(bytes[7], b'-');
        assert_eq!(bytes[10], b' ');
        assert_eq!(bytes[13], b':');
        assert_eq!(bytes[16], b' ');
        assert!(bytes[17] == b'+' || bytes[17] == b'-');
        assert_eq!(bytes[20], b':');
    }
}
