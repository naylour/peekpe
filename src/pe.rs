//! Разбор PE-файла вручную, без внешних крейтов.
//!
//! Модуль разбит по ответственностям:
//! - `parser` — проход по заголовкам и таблицам файла;
//! - `reader` — низкоуровневое чтение чисел/строк и перевод RVA в смещение;
//! - `flags` — расшифровка битовых полей в читаемый вид.

mod flags;
mod parser;
mod reader;

use std::path::PathBuf;

pub use flags::{
    characteristics_flags, dll_characteristics_flags, machine_name, magic_name, section_flags,
};
pub use parser::parse;

/// Полный набор сведений о PE-файле по спецификации лабораторной.
#[derive(Debug, Clone)]
pub struct PeInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,

    pub machine: u16,
    pub characteristics: u16,
    pub magic: u16,
    pub is_pe32_plus: bool,

    pub entry_point: u32,
    pub image_base: u64,
    pub file_alignment: u32,
    pub section_alignment: u32,
    pub size_of_image: u32,
    pub dll_characteristics: u16,

    pub sections: Vec<Section>,
    pub exports: Vec<String>,
    pub imports: Vec<ImportDll>,
    pub has_relocations: bool,
    pub has_resources: bool,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub raw_size: u32,
    pub raw_offset: u32,
    pub characteristics: u32,
}

#[derive(Debug, Clone)]
pub struct ImportDll {
    pub name: String,
    pub functions: Vec<String>,
}
