use color_eyre::eyre::Result;
use crossterm::event::{self, Event as CtEvent, KeyEvent, KeyEventKind};

/// Нормализованное событие, на которое реагирует `App`.
pub enum Event {
    Key(KeyEvent),
    Resize,
    Other,
}

/// Блокирующее ожидание следующего события терминала.
pub fn next() -> Result<Event> {
    Ok(match event::read()? {
        CtEvent::Key(key) if key.kind == KeyEventKind::Press => Event::Key(key),
        CtEvent::Resize(_, _) => Event::Resize,
        _ => Event::Other,
    })
}
