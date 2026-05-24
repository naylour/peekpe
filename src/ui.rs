use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, Paragraph};

use crate::app::{App, EntryKind, Screen};
use crate::pe::{self, PeInfo};

/// Сколько элементов экспорта/импорта показываем в карточке до «показать полностью».
const PREVIEW_EXPORTS: usize = 8;
const PREVIEW_DLLS: usize = 4;
const PREVIEW_FUNCS: usize = 4;

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Explorer => render_explorer(frame, app),
        Screen::Detail => render_detail(frame, app),
        Screen::FullList => render_full_list(frame, app),
    }
}

fn render_explorer(frame: &mut Frame, app: &mut App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let header_line = Line::from(vec![
        Span::styled(" peekpe ", Style::new().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw(" "),
        Span::styled(app.cwd.display().to_string(), Style::new().fg(Color::Cyan)),
    ]);
    frame.render_widget(Paragraph::new(header_line), header);

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|e| {
            let (text, style) = match e.kind {
                EntryKind::Parent => ("..".to_string(), Style::new().fg(Color::Yellow)),
                EntryKind::Dir => (format!("{}/", e.name), Style::new().fg(Color::Blue).bold()),
                EntryKind::PeFile => (e.name.clone(), Style::new().fg(Color::Green)),
            };
            ListItem::new(Line::styled(text, style))
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title(" Файлы "))
        .highlight_symbol("▶ ")
        .highlight_style(Style::new().bg(Color::DarkGray).bold());
    frame.render_stateful_widget(list, body, &mut app.list_state);

    let footer_line = match &app.error {
        Some(err) => Line::styled(err.clone(), Style::new().fg(Color::Red)),
        None => Line::styled(
            "↑/↓ выбор   ↵ открыть   ← вверх   q выход",
            Style::new().fg(Color::DarkGray),
        ),
    };
    frame.render_widget(Paragraph::new(footer_line), footer);
}

fn render_detail(frame: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let title = app
        .selected
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_default();
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" PE ", Style::new().fg(Color::Black).bg(Color::Green).bold()),
            Span::raw(" "),
            Span::styled(title, Style::new().fg(Color::Green).bold()),
        ])),
        header,
    );

    let lines = match &app.selected {
        Some(pe) => detail_lines(pe),
        None => vec![Line::raw("нет данных")],
    };

    let para = Paragraph::new(lines)
        .block(Block::bordered().title(" Разбор "))
        .scroll((app.detail_scroll, 0));
    frame.render_widget(para, body);

    frame.render_widget(
        Paragraph::new(Line::styled(
            "Esc/← назад   ↑/↓ прокрутка   e экспорт   i импорт   q выход",
            Style::new().fg(Color::DarkGray),
        )),
        footer,
    );
}

/// Полноэкранный прокручиваемый список (экспорт или импорт).
fn render_full_list(frame: &mut Frame, app: &mut App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Список ", Style::new().fg(Color::Black).bg(Color::Blue).bold()),
            Span::raw(" "),
            Span::styled(app.full_title.clone(), Style::new().fg(Color::Blue).bold()),
        ])),
        header,
    );

    let items: Vec<ListItem> = app
        .full_items
        .iter()
        .map(|it| {
            let style = if it.header {
                Style::new().fg(Color::Magenta).bold()
            } else {
                Style::new()
            };
            ListItem::new(Line::styled(it.text.clone(), style))
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered())
        .highlight_symbol("▶ ")
        .highlight_style(Style::new().bg(Color::DarkGray).bold());
    frame.render_stateful_widget(list, body, &mut app.full_state);

    frame.render_widget(
        Paragraph::new(Line::styled(
            "↑/↓ прокрутка   PgUp/PgDn   Home/End   Esc назад   q выход",
            Style::new().fg(Color::DarkGray),
        )),
        footer,
    );
}

fn field(key: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key}: "), Style::new().fg(Color::Cyan)),
        Span::raw(value.to_string()),
    ])
}

/// Собирает все строки карточки разбора PE-файла.
fn detail_lines(pe: &PeInfo) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // --- Базовые поля заголовков ---
    lines.push(field("Файл", &pe.name));
    lines.push(field("Путь", &pe.path.display().to_string()));
    lines.push(field("Размер файла", &format!("{} байт", pe.size)));
    lines.push(field(
        "Архитектура (Machine)",
        &format!("{} ({:#06x})", pe::machine_name(pe.machine), pe.machine),
    ));
    lines.push(field(
        "Magic",
        &format!("{} ({:#06x})", pe::magic_name(pe.magic), pe.magic),
    ));
    lines.push(field("Entry Point (RVA)", &format!("{:#010x}", pe.entry_point)));
    lines.push(field("Image Base", &format!("{:#018x}", pe.image_base)));
    lines.push(field(
        "File Alignment (физ.)",
        &format!("{:#x} ({})", pe.file_alignment, pe.file_alignment),
    ));
    lines.push(field(
        "Section Alignment (вирт.)",
        &format!("{:#x} ({})", pe.section_alignment, pe.section_alignment),
    ));
    lines.push(field(
        "Size Of Image",
        &format!("{:#x} ({} байт)", pe.size_of_image, pe.size_of_image),
    ));

    // --- Characteristics (FileHeader) ---
    lines.push(field("Characteristics", &format!("{:#06x}", pe.characteristics)));
    for f in pe::characteristics_flags(pe.characteristics) {
        lines.push(flag_line(f));
    }

    // --- DLL Characteristics ---
    lines.push(field(
        "DLL Characteristics",
        &format!("{:#06x}", pe.dll_characteristics),
    ));
    for f in pe::dll_characteristics_flags(pe.dll_characteristics) {
        lines.push(flag_line(f));
    }

    // --- Предупреждения о релокациях и ресурсах ---
    lines.push(field(
        "Релокации",
        if pe.has_relocations {
            "присутствуют"
        } else {
            "отсутствуют"
        },
    ));
    lines.push(field(
        "Ресурсы",
        if pe.has_resources {
            "присутствуют"
        } else {
            "отсутствуют"
        },
    ));

    // --- Секции ---
    lines.push(Line::raw(""));
    lines.push(heading(&format!("Секции ({})", pe.sections.len())));
    for s in &pe.sections {
        let perms = pe::section_flags(s.characteristics).join(" ");
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<8} ", s.name), Style::new().fg(Color::Green).bold()),
            Span::raw(format!(
                "VA {:#010x}  VSize {:#x}  Raw {:#010x}  RawSize {:#x}",
                s.virtual_address, s.virtual_size, s.raw_offset, s.raw_size
            )),
        ]));
        lines.push(Line::from(vec![
            Span::raw("           "),
            Span::styled(
                format!("[{}]  ({:#010x})", perms, s.characteristics),
                Style::new().fg(Color::Yellow),
            ),
        ]));
    }

    // --- Экспорт (превью) ---
    lines.push(Line::raw(""));
    if pe.exports.is_empty() {
        lines.push(heading("Экспорт: таблица отсутствует"));
    } else {
        lines.push(heading(&format!("Экспорт ({} функций)", pe.exports.len())));
        for name in pe.exports.iter().take(PREVIEW_EXPORTS) {
            lines.push(Line::from(Span::raw(format!("  {name}"))));
        }
        if pe.exports.len() > PREVIEW_EXPORTS {
            lines.push(more_line(pe.exports.len() - PREVIEW_EXPORTS, 'e'));
        }
    }

    // --- Импорт (превью) ---
    lines.push(Line::raw(""));
    if pe.imports.is_empty() {
        lines.push(heading("Импорт: таблица отсутствует"));
    } else {
        let total: usize = pe.imports.iter().map(|d| d.functions.len()).sum();
        lines.push(heading(&format!(
            "Импорт ({} DLL, {} функций)",
            pe.imports.len(),
            total
        )));
        let mut truncated = pe.imports.len() > PREVIEW_DLLS;
        for dll in pe.imports.iter().take(PREVIEW_DLLS) {
            lines.push(Line::from(Span::styled(
                format!("  {}", dll.name),
                Style::new().fg(Color::Magenta).bold(),
            )));
            for func in dll.functions.iter().take(PREVIEW_FUNCS) {
                lines.push(Line::from(Span::raw(format!("      {func}"))));
            }
            if dll.functions.len() > PREVIEW_FUNCS {
                truncated = true;
                lines.push(Line::from(Span::styled(
                    format!("      … ещё {} функций", dll.functions.len() - PREVIEW_FUNCS),
                    Style::new().fg(Color::DarkGray),
                )));
            }
        }
        if pe.imports.len() > PREVIEW_DLLS {
            lines.push(Line::from(Span::styled(
                format!("  … ещё {} DLL", pe.imports.len() - PREVIEW_DLLS),
                Style::new().fg(Color::DarkGray),
            )));
        }
        if truncated {
            lines.push(full_hint('i', "импорт"));
        }
    }

    lines
}

/// Подсказка «показать полностью» для усечённого превью.
fn more_line(rest: usize, key: char) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ↳ показать полностью (ещё {rest}) — клавиша «{key}»"),
        Style::new().fg(Color::Yellow).italic(),
    ))
}

fn full_hint(key: char, what: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ↳ показать {what} полностью — клавиша «{key}»"),
        Style::new().fg(Color::Yellow).italic(),
    ))
}

fn flag_line(name: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  • {name}"),
        Style::new().fg(Color::Gray),
    ))
}

fn heading(text: &str) -> Line<'static> {
    Line::styled(
        text.to_string(),
        Style::new().fg(Color::Cyan).bold().underlined(),
    )
}
