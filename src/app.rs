use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, widgets::ListState};

use crate::pe::{self, PeInfo};
use crate::{tui, ui};

/// Текущий экран приложения.
pub enum Screen {
    Explorer,
    Detail,
    FullList,
}

/// Какую таблицу показываем в полноэкранном списке.
#[derive(Clone, Copy)]
pub enum ListKind {
    Exports,
    Imports,
}

/// Строка полноэкранного списка экспорта/импорта.
pub struct FullItem {
    pub text: String,
    pub header: bool,
}

/// Тип записи в списке проводника.
pub enum EntryKind {
    Parent,
    Dir,
    PeFile,
}

pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
}

pub struct App {
    pub should_quit: bool,
    pub screen: Screen,
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub list_state: ListState,
    pub selected: Option<PeInfo>,
    pub detail_scroll: u16,
    pub error: Option<String>,

    pub full_items: Vec<FullItem>,
    pub full_title: String,
    pub full_state: ListState,
}

impl App {
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let entries = read_entries(&cwd)?;
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }
        Ok(Self {
            should_quit: false,
            screen: Screen::Explorer,
            cwd,
            entries,
            list_state,
            selected: None,
            detail_scroll: 0,
            error: None,
            full_items: Vec::new(),
            full_title: String::new(),
            full_state: ListState::default(),
        })
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| ui::render(frame, self))?;
            if let tui::Event::Key(key) = tui::next()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::Explorer => self.handle_explorer_key(key),
            Screen::Detail => self.handle_detail_key(key),
            Screen::FullList => self.handle_full_list_key(key),
        }
    }

    fn handle_explorer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.select_prev(),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => self.activate(),
            KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => self.go_parent(),
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => {
                self.screen = Screen::Explorer;
            }
            KeyCode::Char('e') => self.open_full(ListKind::Exports),
            KeyCode::Char('i') => self.open_full(ListKind::Imports),
            KeyCode::Down | KeyCode::Char('j') => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn handle_full_list_key(&mut self, key: KeyEvent) {
        let len = self.full_items.len();
        match key.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => {
                self.screen = Screen::Detail;
            }
            KeyCode::Down | KeyCode::Char('j') => move_selection(&mut self.full_state, len, 1),
            KeyCode::Up | KeyCode::Char('k') => move_selection(&mut self.full_state, len, -1),
            KeyCode::PageDown => move_selection(&mut self.full_state, len, 10),
            KeyCode::PageUp => move_selection(&mut self.full_state, len, -10),
            KeyCode::Home if len > 0 => self.full_state.select(Some(0)),
            KeyCode::End if len > 0 => self.full_state.select(Some(len - 1)),
            _ => {}
        }
    }

    /// Открывает полноэкранный список экспорта или импорта.
    fn open_full(&mut self, kind: ListKind) {
        let built = self.selected.as_ref().map(|pe| build_full_items(pe, kind));
        let Some((title, items)) = built else { return };
        if items.is_empty() {
            return;
        }
        self.full_title = title;
        self.full_items = items;
        self.full_state = ListState::default();
        self.full_state.select(Some(0));
        self.screen = Screen::FullList;
    }

    fn select_next(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < self.entries.len() => i + 1,
            _ => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_prev(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => self.entries.len() - 1,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }

    fn activate(&mut self) {
        let Some(i) = self.list_state.selected() else {
            return;
        };
        let Some(entry) = self.entries.get(i) else {
            return;
        };
        match entry.kind {
            EntryKind::Dir | EntryKind::Parent => {
                let path = entry.path.clone();
                self.change_dir(path);
            }
            EntryKind::PeFile => {
                let path = entry.path.clone();
                match pe::parse(&path) {
                    Ok(info) => {
                        self.selected = Some(info);
                        self.detail_scroll = 0;
                        self.error = None;
                        self.screen = Screen::Detail;
                    }
                    Err(e) => self.error = Some(format!("Ошибка разбора: {e}")),
                }
            }
        }
    }

    fn go_parent(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            let path = parent.to_path_buf();
            self.change_dir(path);
        }
    }

    fn change_dir(&mut self, path: PathBuf) {
        match read_entries(&path) {
            Ok(entries) => {
                self.entries = entries;
                self.list_state
                    .select((!self.entries.is_empty()).then_some(0));
                self.cwd = path;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Не удалось открыть {}: {e}", path.display()));
            }
        }
    }
}

/// Сдвигает выделение в списке на `delta` с зажимом в границах (без заворота).
fn move_selection(state: &mut ListState, len: usize, delta: i32) {
    if len == 0 {
        return;
    }
    let cur = state.selected().unwrap_or(0) as i32;
    let next = (cur + delta).clamp(0, len as i32 - 1);
    state.select(Some(next as usize));
}

/// Разворачивает экспорт/импорт PE-файла в плоский список строк для полноэкранного вида.
fn build_full_items(pe: &PeInfo, kind: ListKind) -> (String, Vec<FullItem>) {
    match kind {
        ListKind::Exports => {
            let title = format!("Экспорт — {} функций", pe.exports.len());
            let items = pe
                .exports
                .iter()
                .map(|n| FullItem {
                    text: n.clone(),
                    header: false,
                })
                .collect();
            (title, items)
        }
        ListKind::Imports => {
            let total: usize = pe.imports.iter().map(|d| d.functions.len()).sum();
            let title = format!("Импорт — {} DLL, {} функций", pe.imports.len(), total);
            let mut items = Vec::new();
            for dll in &pe.imports {
                items.push(FullItem {
                    text: format!("{} ({})", dll.name, dll.functions.len()),
                    header: true,
                });
                for f in &dll.functions {
                    items.push(FullItem {
                        text: format!("    {f}"),
                        header: false,
                    });
                }
            }
            (title, items)
        }
    }
}

/// Читает директорию: сначала `..`, затем папки, затем `.dll`/`.exe` (по алфавиту).
fn read_entries(dir: &Path) -> Result<Vec<Entry>> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let Ok(entry) = entry else { continue };
        let Ok(ft) = entry.file_type() else { continue };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if ft.is_dir() {
            dirs.push(Entry {
                name,
                path,
                kind: EntryKind::Dir,
            });
        } else if is_pe(&path) {
            files.push(Entry {
                name,
                path,
                kind: EntryKind::PeFile,
            });
        }
    }

    dirs.sort_by_key(|e| e.name.to_lowercase());
    files.sort_by_key(|e| e.name.to_lowercase());

    let mut entries = Vec::with_capacity(dirs.len() + files.len() + 1);
    if let Some(parent) = dir.parent() {
        entries.push(Entry {
            name: "..".into(),
            path: parent.to_path_buf(),
            kind: EntryKind::Parent,
        });
    }
    entries.extend(dirs);
    entries.extend(files);
    Ok(entries)
}

fn is_pe(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("dll" | "exe")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_app() {
        let mut app = App::new().unwrap();
        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn detects_pe_extensions() {
        assert!(is_pe(Path::new("a.exe")));
        assert!(is_pe(Path::new("b.DLL")));
        assert!(!is_pe(Path::new("c.txt")));
        assert!(!is_pe(Path::new("noext")));
    }
}
