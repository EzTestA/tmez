use std::process::Command;

/// Статическая информация о процессоре.
#[derive(Debug, Clone, Default)]
pub struct CpuStaticInfo {
    pub virtualization_enabled: bool,
    pub l2_cache_kb: u64,
    pub l3_cache_kb: u64,
}

/// Статическая информация о памяти.
#[derive(Debug, Clone, Default)]
pub struct MemoryStaticInfo {
    /// Тип памяти: "DDR3", "DDR4" и т.п.
    pub memory_type: String,
    /// Скорость в MT/s
    pub speed_mts: u64,
    /// Форм-фактор: "SODIMM", "DIMM" и т.п.
    pub form_factor: String,
    /// Всего слотов
    pub slots_total: u64,
    /// Занято слотов
    pub slots_used: u64,
}

/// Запрашивает статическую информацию о CPU через PowerShell.
pub fn query_cpu_static_info() -> CpuStaticInfo {
    let script = "Get-CimInstance Win32_Processor \
                  | Select-Object VirtualizationFirmwareEnabled, L2CacheSize, L3CacheSize \
                  | Format-List";

    let output = match Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
    {
        Ok(o) => o,
        Err(_) => return CpuStaticInfo::default(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = CpuStaticInfo::default();

    for line in stdout.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "VirtualizationFirmwareEnabled" => {
                    info.virtualization_enabled = value.eq_ignore_ascii_case("True");
                }
                "L2CacheSize" => {
                    info.l2_cache_kb = value.parse().unwrap_or(0);
                }
                "L3CacheSize" => {
                    info.l3_cache_kb = value.parse().unwrap_or(0);
                }
                _ => {}
            }
        }
    }

    info
}

/// Запрашивает статическую информацию о памяти через PowerShell.
pub fn query_memory_static_info() -> MemoryStaticInfo {
    let mut info = MemoryStaticInfo::default();

    // --- Первый запрос: одна планка памяти (тип, скорость, форм-фактор) ---
    let script1 = "Get-CimInstance Win32_PhysicalMemory \
                   | Select-Object -First 1 \
                   | Select-Object SMBIOSMemoryType, Speed, FormFactor \
                   | Format-List";

    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script1])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "SMBIOSMemoryType" => {
                        info.memory_type = decode_memory_type(value);
                    }
                    "Speed" => {
                        info.speed_mts = value.parse().unwrap_or(0);
                    }
                    "FormFactor" => {
                        info.form_factor = decode_form_factor(value);
                    }
                    _ => {}
                }
            }
        }
    }

    // --- Второй запрос: количество слотов ---
    let script2 = "Get-CimInstance Win32_PhysicalMemoryArray \
                   | Select-Object MemoryDevices \
                   | Format-List";

    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script2])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                if key == "MemoryDevices" {
                    info.slots_total = value.parse().unwrap_or(0);
                }
            }
        }
    }

    // --- Третий запрос: сколько планок реально вставлено ---
    let script3 = "(Get-CimInstance Win32_PhysicalMemory | Measure-Object).Count";
    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script3])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        info.slots_used = trimmed.parse().unwrap_or(0);
    }

    info
}

/// Расшифровывает код SMBIOSMemoryType в человекочитаемый тип.
/// Коды из спецификации SMBIOS.
fn decode_memory_type(code_str: &str) -> String {
    let code: u64 = code_str.parse().unwrap_or(0);
    match code {
        20 => "DDR".to_string(),
        21 => "DDR2".to_string(),
        24 => "DDR3".to_string(),
        26 => "DDR4".to_string(),
        34 => "DDR5".to_string(),
        0 => String::new(),
        _ => format!("Тип {}", code),
    }
}

/// Расшифровывает код FormFactor в человекочитаемый вид.
fn decode_form_factor(code_str: &str) -> String {
    let code: u64 = code_str.parse().unwrap_or(0);
    match code {
        8 => "DIMM".to_string(),
        12 => "SODIMM".to_string(),
        13 => "SRIMM".to_string(),
        0 => String::new(),
        _ => format!("FF {}", code),
    }
}

/// Возвращает map: буква тома (например, 'C') -> модель физического диска.
/// Например: [('C', "KINGSTON SA400S37480G"), ('D', "WDC WD10JPVX-22JC3T0")]
pub fn query_disk_models_by_letter() -> std::collections::HashMap<char, String> {
    use std::collections::HashMap;

    let script = r#"
$result = @{}
Get-CimInstance Win32_DiskDrive | ForEach-Object {
    $disk = $_
    $partitions = Get-CimAssociatedInstance -InputObject $disk -ResultClassName Win32_DiskPartition
    foreach ($p in $partitions) {
        $logicals = Get-CimAssociatedInstance -InputObject $p -ResultClassName Win32_LogicalDisk
        foreach ($l in $logicals) {
            if ($l.DeviceID -match '^([A-Z]):') {
                $result[$matches[1]] = $disk.Model
            }
        }
    }
}
$result.GetEnumerator() | ForEach-Object { "$($_.Key)|$($_.Value)" }
"#;

    let output = match Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
    {
        Ok(o) => o,
        Err(_) => return HashMap::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut map = HashMap::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((letter_str, model)) = line.split_once('|') {
            if let Some(c) = letter_str.trim().chars().next() {
                map.insert(c, model.trim().to_string());
            }
        }
    }

    map
}