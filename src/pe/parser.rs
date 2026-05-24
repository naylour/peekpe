//! Проход по структуре PE: DOS → сигнатура PE → COFF → Optional Header → секции → импорт/экспорт.

use std::fs;
use std::path::Path;

use color_eyre::eyre::{Result, bail, eyre};

use super::reader::{cstr_at, rva_to_offset, u16_at, u32_at, u64_at};
use super::{ImportDll, PeInfo, Section};

// Индексы нужных нам Data Directory.
const DIR_EXPORT: usize = 0;
const DIR_IMPORT: usize = 1;
const DIR_RESOURCE: usize = 2;
const DIR_BASERELOC: usize = 5;

// Пределы на случай повреждённых таблиц — чтобы не уйти в бесконечный цикл.
const MAX_IMPORT_DLLS: usize = 4096;
const MAX_FUNCS_PER_DLL: usize = 65536;

/// Разбирает PE-файл целиком: читает в память и проходит заголовки вручную.
pub fn parse(path: &Path) -> Result<PeInfo> {
    let data = fs::read(path)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    if u16_at(&data, 0)? != 0x5A4D {
        bail!("нет DOS-сигнатуры MZ");
    }
    let pe_off = u32_at(&data, 0x3C)? as usize; // e_lfanew — смещение PE-заголовка

    if u32_at(&data, pe_off)? != 0x0000_4550 {
        bail!("нет PE-сигнатуры по смещению {pe_off:#x}");
    }

    // COFF File Header идёт сразу после сигнатуры "PE\0\0".
    let coff = pe_off + 4;
    let machine = u16_at(&data, coff)?;
    let section_count = u16_at(&data, coff + 2)? as usize;
    let opt_size = u16_at(&data, coff + 16)? as usize;
    let characteristics = u16_at(&data, coff + 18)?;

    // Optional Header. Разрядность задаётся Magic и сдвигает часть полей.
    let opt = coff + 20;
    let magic = u16_at(&data, opt)?;
    let is_pe32_plus = match magic {
        0x10B => false,
        0x20B => true,
        other => bail!("неизвестный Magic опционального заголовка: {other:#x}"),
    };

    let entry_point = u32_at(&data, opt + 16)?;
    let image_base = if is_pe32_plus {
        u64_at(&data, opt + 24)?
    } else {
        u32_at(&data, opt + 28)? as u64
    };
    let section_alignment = u32_at(&data, opt + 32)?;
    let file_alignment = u32_at(&data, opt + 36)?;
    let size_of_image = u32_at(&data, opt + 56)?;
    let dll_characteristics = u16_at(&data, opt + 70)?;

    // Кол-во и начало массива Data Directory различаются у PE32 и PE32+.
    let dir_count = if is_pe32_plus {
        u32_at(&data, opt + 108)?
    } else {
        u32_at(&data, opt + 92)?
    } as usize;
    let dir_start = opt + if is_pe32_plus { 112 } else { 96 };

    let directory = |index: usize| -> Result<(u32, u32)> {
        if index >= dir_count {
            return Ok((0, 0));
        }
        let at = dir_start + index * 8;
        Ok((u32_at(&data, at)?, u32_at(&data, at + 4)?))
    };
    let (export_rva, _) = directory(DIR_EXPORT)?;
    let (import_rva, _) = directory(DIR_IMPORT)?;
    let (resource_rva, _) = directory(DIR_RESOURCE)?;
    let (reloc_rva, _) = directory(DIR_BASERELOC)?;

    let sections = read_sections(&data, opt + opt_size, section_count)?;

    // Битая таблица импорта/экспорта не должна рушить весь разбор файла.
    let exports = match export_rva {
        0 => Vec::new(),
        rva => read_exports(&data, &sections, rva).unwrap_or_default(),
    };
    let imports = match import_rva {
        0 => Vec::new(),
        rva => read_imports(&data, &sections, rva, is_pe32_plus).unwrap_or_default(),
    };

    Ok(PeInfo {
        name,
        path: path.to_path_buf(),
        size: data.len() as u64,
        machine,
        characteristics,
        magic,
        is_pe32_plus,
        entry_point,
        image_base,
        file_alignment,
        section_alignment,
        size_of_image,
        dll_characteristics,
        sections,
        exports,
        imports,
        has_relocations: reloc_rva != 0,
        has_resources: resource_rva != 0,
    })
}

/// Читает таблицу секций: по 40 байт на запись, начиная сразу за Optional Header.
fn read_sections(data: &[u8], start: usize, count: usize) -> Result<Vec<Section>> {
    (0..count)
        .map(|i| {
            let head = start + i * 40;
            let raw_name = data
                .get(head..head + 8)
                .ok_or_else(|| eyre!("заголовок секции #{i} за границей файла"))?;
            let end = raw_name.iter().position(|&c| c == 0).unwrap_or(8);
            Ok(Section {
                name: String::from_utf8_lossy(&raw_name[..end]).into_owned(),
                virtual_size: u32_at(data, head + 8)?,
                virtual_address: u32_at(data, head + 12)?,
                raw_size: u32_at(data, head + 16)?,
                raw_offset: u32_at(data, head + 20)?,
                characteristics: u32_at(data, head + 36)?,
            })
        })
        .collect()
}

/// Таблица экспортов: имена функций из массива AddressOfNames.
fn read_exports(data: &[u8], sections: &[Section], export_rva: u32) -> Result<Vec<String>> {
    let dir = rva_to_offset(sections, export_rva).ok_or_else(|| eyre!("export dir вне секций"))?;
    let name_count = u32_at(data, dir + 24)? as usize;
    let names_rva = u32_at(data, dir + 32)?;
    let names = rva_to_offset(sections, names_rva).ok_or_else(|| eyre!("AddressOfNames вне секций"))?;

    let mut exports = Vec::with_capacity(name_count);
    for i in 0..name_count {
        let name_rva = u32_at(data, names + i * 4)?;
        if let Some(at) = rva_to_offset(sections, name_rva) {
            exports.push(cstr_at(data, at));
        }
    }
    Ok(exports)
}

/// Таблица импортов: список DLL, по каждой — импортируемые функции.
fn read_imports(
    data: &[u8],
    sections: &[Section],
    import_rva: u32,
    is_pe32_plus: bool,
) -> Result<Vec<ImportDll>> {
    let mut descriptor =
        rva_to_offset(sections, import_rva).ok_or_else(|| eyre!("import dir вне секций"))?;
    let mut dlls = Vec::new();

    for _ in 0..MAX_IMPORT_DLLS {
        let lookup_rva = u32_at(data, descriptor)?; // OriginalFirstThunk (ILT)
        let name_rva = u32_at(data, descriptor + 12)?;
        let iat_rva = u32_at(data, descriptor + 16)?; // FirstThunk (IAT)
        // Список дескрипторов завершается нулевой записью.
        if name_rva == 0 {
            break;
        }

        let name = rva_to_offset(sections, name_rva)
            .map(|at| cstr_at(data, at))
            .unwrap_or_default();
        // Идём по ILT: в загруженном образе IAT уже перезаписан реальными адресами.
        let thunks_rva = if lookup_rva != 0 { lookup_rva } else { iat_rva };
        let functions = read_thunks(data, sections, thunks_rva, is_pe32_plus)?;

        dlls.push(ImportDll { name, functions });
        descriptor += 20; // размер IMAGE_IMPORT_DESCRIPTOR
    }

    Ok(dlls)
}

/// Проходит таблицу thunk'ов одной DLL: имена функций либо импорт по ординалу.
fn read_thunks(
    data: &[u8],
    sections: &[Section],
    thunks_rva: u32,
    is_pe32_plus: bool,
) -> Result<Vec<String>> {
    let Some(mut thunk) = rva_to_offset(sections, thunks_rva) else {
        return Ok(Vec::new());
    };
    let step = if is_pe32_plus { 8 } else { 4 };
    let ordinal_flag: u64 = if is_pe32_plus {
        0x8000_0000_0000_0000
    } else {
        0x8000_0000
    };

    let mut functions = Vec::new();
    for _ in 0..MAX_FUNCS_PER_DLL {
        let entry = if is_pe32_plus {
            u64_at(data, thunk)?
        } else {
            u32_at(data, thunk)? as u64
        };
        if entry == 0 {
            break; // конец таблицы
        }
        if entry & ordinal_flag != 0 {
            functions.push(format!("Ordinal #{}", entry & 0xFFFF));
        } else if let Some(at) = rva_to_offset(sections, entry as u32) {
            functions.push(cstr_at(data, at + 2)); // IMAGE_IMPORT_BY_NAME: u16 Hint, затем имя
        }
        thunk += step;
    }
    Ok(functions)
}
