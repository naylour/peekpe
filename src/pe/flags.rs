//! Расшифровка числовых/битовых полей PE в человекочитаемый вид.

/// Архитектура по полю Machine.
pub fn machine_name(machine: u16) -> &'static str {
    match machine {
        0x014c => "x86 (i386)",
        0x8664 => "x64 (AMD64)",
        0x01c0 => "ARM",
        0x01c4 => "ARM Thumb-2",
        0xaa64 => "ARM64",
        0x0200 => "Intel IA-64",
        0x0000 => "неизвестно",
        _ => "прочая",
    }
}

/// Имя формата по полю Magic опционального заголовка.
pub fn magic_name(magic: u16) -> &'static str {
    match magic {
        0x10B => "PE32",
        0x20B => "PE32+ (64-бит)",
        0x107 => "ROM",
        _ => "?",
    }
}

/// Биты Characteristics (FileHeader) → имена флагов.
pub fn characteristics_flags(value: u16) -> Vec<&'static str> {
    const FLAGS: &[(u32, &str)] = &[
        (0x0001, "RELOCS_STRIPPED"),
        (0x0002, "EXECUTABLE_IMAGE"),
        (0x0004, "LINE_NUMS_STRIPPED"),
        (0x0008, "LOCAL_SYMS_STRIPPED"),
        (0x0010, "AGGRESSIVE_WS_TRIM"),
        (0x0020, "LARGE_ADDRESS_AWARE"),
        (0x0080, "BYTES_REVERSED_LO"),
        (0x0100, "32BIT_MACHINE"),
        (0x0200, "DEBUG_STRIPPED"),
        (0x0400, "REMOVABLE_RUN_FROM_SWAP"),
        (0x0800, "NET_RUN_FROM_SWAP"),
        (0x1000, "SYSTEM"),
        (0x2000, "DLL"),
        (0x4000, "UP_SYSTEM_ONLY"),
        (0x8000, "BYTES_REVERSED_HI"),
    ];
    set_flags(FLAGS, value as u32)
}

/// Биты DllCharacteristics → имена флагов.
pub fn dll_characteristics_flags(value: u16) -> Vec<&'static str> {
    const FLAGS: &[(u32, &str)] = &[
        (0x0020, "HIGH_ENTROPY_VA"),
        (0x0040, "DYNAMIC_BASE"),
        (0x0080, "FORCE_INTEGRITY"),
        (0x0100, "NX_COMPAT"),
        (0x0200, "NO_ISOLATION"),
        (0x0400, "NO_SEH"),
        (0x0800, "NO_BIND"),
        (0x1000, "APPCONTAINER"),
        (0x2000, "WDM_DRIVER"),
        (0x4000, "GUARD_CF"),
        (0x8000, "TERMINAL_SERVER_AWARE"),
    ];
    set_flags(FLAGS, value as u32)
}

/// Характеристики секции буквами/именами: права доступа и тип содержимого.
pub fn section_flags(value: u32) -> Vec<&'static str> {
    const FLAGS: &[(u32, &str)] = &[
        (0x4000_0000, "R"), // MEM_READ
        (0x8000_0000, "W"), // MEM_WRITE
        (0x2000_0000, "X"), // MEM_EXECUTE
        (0x0000_0020, "CODE"),
        (0x0000_0040, "INIT_DATA"),
        (0x0000_0080, "UNINIT_DATA"),
        (0x0200_0000, "DISCARDABLE"),
        (0x1000_0000, "SHARED"),
    ];
    set_flags(FLAGS, value)
}

/// Оставляет имена тех флагов, чьи биты выставлены в значении.
fn set_flags(flags: &[(u32, &'static str)], value: u32) -> Vec<&'static str> {
    flags
        .iter()
        .filter(|(bit, _)| value & bit != 0)
        .map(|&(_, name)| name)
        .collect()
}
