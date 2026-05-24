//! Низкоуровневое чтение из буфера файла: числа little-endian, ASCII-строки,
//! перевод виртуального адреса (RVA) в физическое смещение.

use color_eyre::eyre::{Result, eyre};

use super::Section;

/// Переводит RVA в смещение в файле через таблицу секций.
pub(super) fn rva_to_offset(sections: &[Section], rva: u32) -> Option<usize> {
    sections.iter().find_map(|s| {
        let span = s.virtual_size.max(s.raw_size);
        let inside = rva >= s.virtual_address && rva < s.virtual_address.saturating_add(span);
        inside.then(|| (rva - s.virtual_address + s.raw_offset) as usize)
    })
}

pub(super) fn u16_at(buf: &[u8], off: usize) -> Result<u16> {
    let b = buf
        .get(off..off + 2)
        .ok_or_else(|| eyre!("чтение u16 за границей файла @ {off:#x}"))?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}

pub(super) fn u32_at(buf: &[u8], off: usize) -> Result<u32> {
    let b = buf
        .get(off..off + 4)
        .ok_or_else(|| eyre!("чтение u32 за границей файла @ {off:#x}"))?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

pub(super) fn u64_at(buf: &[u8], off: usize) -> Result<u64> {
    let b = buf
        .get(off..off + 8)
        .ok_or_else(|| eyre!("чтение u64 за границей файла @ {off:#x}"))?;
    Ok(u64::from_le_bytes([
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
    ]))
}

/// Читает null-терминированную ASCII-строку начиная со смещения.
pub(super) fn cstr_at(buf: &[u8], off: usize) -> String {
    let Some(tail) = buf.get(off..) else {
        return String::new();
    };
    let end = tail.iter().position(|&c| c == 0).unwrap_or(tail.len());
    String::from_utf8_lossy(&tail[..end]).into_owned()
}
