use std::path::PathBuf;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::Widget;

use crate::state::tiles::{FileType, Tile};
use crate::ui::format::{truncate_middle, DisplaySize};

fn render_currently_selected(buf: &mut Buffer, currently_selected: &Tile, max_len: u16, y: u16) {
    let file_name = currently_selected.name.to_string_lossy();
    let size = DisplaySize(currently_selected.size as f64);
    let descendants = currently_selected.descendants;

    // Define styles for different file types
    let (style, lines) = match currently_selected.file_type {
        FileType::File => (
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            vec![
                // Arrow in Yellow, filename in White, size in Cyan
                format!("→ {} ({})", file_name, size),
                format!("→ {}", file_name),
            ],
        ),
        FileType::Folder => (
            Style::default()
                .fg(Color::Blue)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            vec![
                format!("→ {} ({} items)", file_name, descendants.unwrap_or(0)),
                format!("→ {} ({})", file_name, size),
                format!("→ {}", file_name),
            ],
        ),
    };

    // Choose the first line that fits within max_len
    for line in lines {
        if (line.chars().count() as u16) <= max_len {
            buf.set_string(1, y, &line, style);
            break;
        }
    }
}

fn render_last_read_path(buf: &mut Buffer, last_read_path: &PathBuf, max_len: u16, y: u16) {
    let last_read_path_str = last_read_path.to_string_lossy();
    let display_str = if (last_read_path_str.chars().count() as u16) <= max_len {
        last_read_path_str.to_string()
    } else {
        truncate_middle(&last_read_path_str, max_len)
    };

    buf.set_string(
        1,
        y,
        &display_str,
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::ITALIC),
    );
}

fn render_controls_legend(buf: &mut Buffer, hide_delete: bool, max_len: u16, y: u16) {
    let (long_controls_line, short_controls_line) = if hide_delete {
        (
            "Q QUIT | +/-0 ZOOM".to_string(),
            "Q QUIT | +/-0 ZOOM".to_string(),
        )
    } else {
        (
            "BACKSPACE DEL | Q QUIT | +/-0 ZOOM".to_string(),
            "BACKSPACE DEL | Q QUIT | +/-0 ZOOM".to_string(),
        )
    };
    let too_small_line = "(Controls Hidden)";

    let styled_line = if max_len >= long_controls_line.chars().count() as u16 {
        long_controls_line
    } else if max_len >= short_controls_line.chars().count() as u16 {
        short_controls_line
    } else {
        too_small_line.to_string()
    };

    buf.set_string(
        1,
        y,
        &styled_line,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
}

 

pub struct BottomLine<'a> {
    hide_delete: bool,
    hide_small_files_legend: bool,
    currently_selected: Option<&'a Tile>,
    last_read_path: Option<&'a PathBuf>,
}

impl<'a> BottomLine<'a> {
    pub fn new() -> Self {
        Self {
            hide_delete: false,
            hide_small_files_legend: false,
            currently_selected: None,
            last_read_path: None,
        }
    }
    pub fn hide_delete(mut self) -> Self {
        self.hide_delete = true;
        self
    }
    pub fn hide_small_files_legend(mut self, should_hide_small_files_legend: bool) -> Self {
        self.hide_small_files_legend = should_hide_small_files_legend;
        self
    }
    pub fn currently_selected(mut self, currently_selected: Option<&'a Tile>) -> Self {
        self.currently_selected = currently_selected;
        self
    }
    pub fn last_read_path(mut self, last_read_path: Option<&'a PathBuf>) -> Self {
        self.last_read_path = last_read_path;
        self
    }
}

impl<'a> Widget for BottomLine<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
   
        let small_files_len =  0;
        let max_status_len = area.width - small_files_len - 1;
        let max_controls_len = area.width - 1;
        let status_line_y = area.y + area.height - 2;
        let controls_line_y = status_line_y + 1;

        // Render the status line
        if let Some(currently_selected) = self.currently_selected {
            render_currently_selected(buf, currently_selected, max_status_len, status_line_y);
        } else if let Some(last_read_path) = self.last_read_path {
            render_last_read_path(buf, last_read_path, max_status_len, status_line_y);
        }

    

        // Render the controls legend
        render_controls_legend(buf, self.hide_delete, max_controls_len, controls_line_y);
    }
}
