use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode};

const MAX_PATH_WIDTH: u16 = 80;
const CMD_CANDIDATE_ROWS: u16 = 6;
const INPUT_ROWS: u16 = 3;
const SHELL_PANEL_ROWS: u16 = 6;

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
            Constraint::Length(2),
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

    let list_title = format!(" files ({}/{}) ", app.entries.len(), app.all_entries.len());
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
            let name = truncate_to_width(&e.name, width.saturating_sub(6));
            let line = format!("{}{}{}", icon, name, size_str);
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
                .map(|e| {
                    let kind = if e.is_dir { "dir" } else { "file" };
                    let size = e
                        .size
                        .map(|s| human_size(s))
                        .unwrap_or_else(|| "-".to_string());
                    format!(" {}  {}  {}", e.name, kind, size)
                })
                .unwrap_or_default()
        };
        let status_trunc = truncate_to_width(&status, width.saturating_sub(4));
        let hint = " j/k: move  Enter: open  /: filter  : command  ! shell  q: quit ";
        let block = Block::default()
            .title(Line::from(hint))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::DarkGray));
        let para = Paragraph::new(status_trunc)
            .block(block)
            .style(Style::default().fg(Color::Gray));
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
