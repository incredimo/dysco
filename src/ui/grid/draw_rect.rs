use ::tui::buffer::Buffer;
use ::tui::layout::Rect;
use ::tui::style::{Color, Modifier, Style};
use ::unicode_width::UnicodeWidthStr;

use crate::state::tiles::{FileType, Tile};
use crate::ui::format::{truncate_middle, DisplaySize, DisplaySizeRounded};
use crate::ui::grid::{boundaries, draw_next_symbol};

fn get_color_at_pos(x: u16, y: u16, width: u16, height: u16, is_file: bool) -> Color {
    let x_ratio = x as f64 / width as f64;
    let y_ratio = y as f64 / height as f64;
    let position = (x_ratio + (1.0 - y_ratio)) / 2.0;

    if is_file {
        match (position * 1000.0) as u16 {
            0..=200 => Color::Rgb(255, 121, 198),    // Pink
            201..=400 => Color::Rgb(241, 134, 210),  // Pink-Purple
            401..=600 => Color::Rgb(189, 147, 249),  // Purple
            601..=800 => Color::Rgb(164, 190, 251),  // Purple-Cyan
            _ => Color::Rgb(139, 233, 253),          // Cyan
        }
    } else {
        match (position * 1000.0) as u16 {
            0..=200 => Color::Rgb(139, 233, 253),    // Cyan
            201..=400 => Color::Rgb(109, 241, 188),  // Cyan-Green
            401..=600 => Color::Rgb(80, 250, 123),   // Green
            601..=800 => Color::Rgb(167, 217, 115),  // Green-Orange
            _ => Color::Rgb(255, 184, 108),          // Orange
        }
    }
}

fn get_shade_at_pos(x: u16, y: u16, width: u16, height: u16) -> &'static str {
    let x_ratio = x as f64 / width as f64;
    let y_ratio = y as f64 / height as f64;
    let position = (x_ratio + (1.0 - y_ratio)) / 2.0;
    
    match (position * 1000.0) as u16 {
        0..=200 => "░",
        201..=400 => "▒",
        401..=600 => "▒",
        601..=800 => "▓",
        _ => "█",
    }
}

fn tile_first_line(tile: &Tile) -> String {
    let max_text_length = if tile.width > 2 { tile.width - 2 } else { 0 };
    let name = &tile.name.to_string_lossy();
    let descendant_count = &tile.descendants;
    let filename_text = match tile.file_type {
        FileType::File => format!("{}", name),
        FileType::Folder => format!("{}/", name),
    };
    match tile.file_type {
        FileType::File => truncate_middle(&filename_text, max_text_length),
        FileType::Folder => {
            let descendant_count = descendant_count.expect("folder should have descendants");
            let short_descendants_indication = format!("(+{})", descendant_count);
            let long_descendants_indication = format!("(+{} descendants)", descendant_count);
            if filename_text.len() + long_descendants_indication.len() <= max_text_length as usize {
                format!("{} {}", filename_text, long_descendants_indication)
            } else if filename_text.len() + short_descendants_indication.len()
                <= max_text_length as usize
            {
                format!("{} {}", filename_text, short_descendants_indication)
            } else {
                truncate_middle(&filename_text, max_text_length)
            }
        }
    }
}

fn tile_second_line(tile: &Tile) -> String {
    let max_text_length = if tile.width > 2 { tile.width - 2 } else { 0 };
    let percentage = &tile.percentage;
    let display_size = DisplaySize(tile.size as f64);
    let display_size_rounded = DisplaySizeRounded(tile.size as f64);
    let display_size = format!("{}", display_size);
    let display_size_rounded = format!("{}", display_size_rounded);
    if max_text_length >= display_size.len() as u16 + 7 {
        format!("{} ({:.0}%)", display_size, percentage * 100.0)
    } else if max_text_length > display_size.len() as u16 {
        display_size
    } else if max_text_length > display_size_rounded.len() as u16 {
        display_size_rounded
    } else if max_text_length > 6 {
        format!("({:.0}%)", (percentage * 100.0).round())
    } else if max_text_length >= 4 {
        format!("{:.0}%", (percentage * 100.0).round())
    } else {
        unreachable!("trying to render a rect of less than minimum size")
    }
}

pub fn tile_style(tile: &Tile, selected: bool) -> (Option<Style>, Style, Style) {
    let (background_style, first_line_style, second_line_style) = match (selected, &tile.file_type) {
        (true, FileType::File) => (
            Some(Style::default().fg(Color::Gray).bg(Color::Gray)),
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        (false, FileType::File) => (None, Style::default(), Style::default()),
        (true, FileType::Folder) => (
            Some(Style::default().fg(Color::Blue).bg(Color::Blue)),
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(Color::Black).bg(Color::Blue),
        ),
        (false, FileType::Folder) => (
            None,
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            Style::default(),
        ),
    };
    (background_style, first_line_style, second_line_style)
}

pub fn draw_rect_on_grid(buf: &mut Buffer, coords: (u16, u16), dimensions: (u16, u16)) {
    let (x, y) = coords;
    let (width, height) = dimensions;
    if width < 1 || height < 1 {
        return;
    }

    for i in x..(x + width + 1) {
        if i == x {
            draw_next_symbol(buf, i, y, "╭");
            draw_next_symbol(buf, i, y + height, "╰");
        } else if i == x + width {
            draw_next_symbol(buf, i, y, "╮");
            draw_next_symbol(buf, i, y + height, "╯");
        } else {
            draw_next_symbol(buf, i, y, "─");
            draw_next_symbol(buf, i, y + height, "─");
        }
    }

    for j in (y + 1)..(y + height) {
        draw_next_symbol(buf, x, j, "│");
        draw_next_symbol(buf, x + width, j, "│");
    }
}

pub fn draw_filled_rect(buf: &mut Buffer, fill_style: Style, rect: &Rect) {
    let is_file = fill_style.fg == Some(Color::Gray);

    // Fill with granular gradient
    for y in rect.y + 1..rect.y + rect.height {
        for x in rect.x + 1..rect.x + rect.width {
            let color = get_color_at_pos(
                x - rect.x,
                y - rect.y,
                rect.width,
                rect.height,
                is_file
            );
            let shade = get_shade_at_pos(
                x - rect.x,
                y - rect.y,
                rect.width,
                rect.height
            );
            
            buf.get_mut(x, y)
                .set_symbol(shade)
                .set_style(Style::default().fg(color));
        }
    }

    // Draw borders with matching gradient
    let corners = [
        (rect.x, rect.y, "╭"),
        (rect.x + rect.width, rect.y, "╮"),
        (rect.x, rect.y + rect.height, "╰"),
        (rect.x + rect.width, rect.y + rect.height, "╯")
    ];

    for (x, y, symbol) in corners {
        let color = get_color_at_pos(
            x - rect.x,
            y - rect.y,
            rect.width,
            rect.height,
            is_file
        );
        buf.get_mut(x, y)
            .set_symbol(symbol)
            .set_style(Style::default().fg(color));
    }

    for x in rect.x + 1..rect.x + rect.width {
        let top_color = get_color_at_pos(x - rect.x, 0, rect.width, rect.height, is_file);
        let bottom_color = get_color_at_pos(x - rect.x, rect.height - 1, rect.width, rect.height, is_file);
        
        buf.get_mut(x, rect.y)
            .set_symbol("─")
            .set_style(Style::default().fg(top_color));
        buf.get_mut(x, rect.y + rect.height)
            .set_symbol("─")
            .set_style(Style::default().fg(bottom_color));
    }

    for y in rect.y + 1..rect.y + rect.height {
        let left_color = get_color_at_pos(0, y - rect.y, rect.width, rect.height, is_file);
        let right_color = get_color_at_pos(rect.width - 1, y - rect.y, rect.width, rect.height, is_file);
        
        buf.get_mut(rect.x, y)
            .set_symbol("│")
            .set_style(Style::default().fg(left_color));
        buf.get_mut(rect.x + rect.width, y)
            .set_symbol("│")
            .set_style(Style::default().fg(right_color));
    }
}

pub fn draw_tile_text_on_grid(buf: &mut Buffer, tile: &Tile, selected: bool) {
    let first_line = tile_first_line(tile);
    let second_line = tile_second_line(tile);
    let (background_style, first_line_style, second_line_style) = tile_style(tile, selected);

    // Draw the gradient background for selected tiles
    if let Some(_) = background_style {
        let is_file = matches!(tile.file_type, FileType::File);
        for y in tile.y + 1..tile.y + tile.height {
            for x in tile.x + 1..tile.x + tile.width {
                let color = get_color_at_pos(
                    x - tile.x,
                    y - tile.y,
                    tile.width,
                    tile.height,
                    is_file
                );
                let shade = get_shade_at_pos(
                    x - tile.x,
                    y - tile.y,
                    tile.width,
                    tile.height
                );
                
                buf.get_mut(x, y)
                    .set_symbol(shade)
                    .set_style(Style::default().fg(color));
            }
        }
    }

    // Position text in top-left with padding
    let padding = 2;
    let max_width = tile.width.saturating_sub(padding * 2);
    
    // Handle text placement based on available space
    if tile.height > 3 {
        buf.set_string(
            tile.x + padding,
            tile.y + padding,
            truncate_middle(&first_line, max_width),
            first_line_style,
        );
        buf.set_string(
            tile.x + padding,
            tile.y + padding + 1,
            truncate_middle(&second_line, max_width),
            second_line_style,
        );
    } else if tile.height > 2 {
        buf.set_string(
            tile.x + padding,
            tile.y + padding,
            truncate_middle(&first_line, max_width),
            first_line_style,
        );
    }
}