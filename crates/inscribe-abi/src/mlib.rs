use crate::calling_conv::AbiTarget;
use crate::versioning::{AbiVersion, CURRENT_ABI_VERSION};

pub const MLIB_MAGIC: [u8; 4] = *b"MLIB";
pub const MLIB_HEADER_SIZE: usize = 80;
pub const MLIB_EXPORT_ENTRY_SIZE: usize = 24;
pub const MLIB_FLAG_EMBEDDED_SOURCE: u32 = 1 << 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlibHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub abi_version: AbiVersion,
    pub target: AbiTarget,
    pub flags: u32,
    pub export_table_offset: u64,
    pub export_table_count: u32,
    pub string_table_offset: u64,
    pub string_table_size: u64,
    pub code_offset: u64,
    pub code_size: u64,
    pub data_offset: u64,
    pub data_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlibExportKind {
    Function,
    Global,
    Type,
}

impl MlibExportKind {
    fn to_byte(self) -> u8 {
        match self {
            Self::Function => 1,
            Self::Global => 2,
            Self::Type => 3,
        }
    }

    fn from_byte(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Function),
            2 => Some(Self::Global),
            3 => Some(Self::Type),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlibExport {
    pub name: String,
    pub kind: MlibExportKind,
    pub address: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlibFile {
    pub header: MlibHeader,
    pub exports: Vec<MlibExport>,
    pub string_table: Vec<u8>,
    pub code: Vec<u8>,
    pub data: Vec<u8>,
}

impl MlibFile {
    pub fn new(target: AbiTarget, exports: Vec<MlibExport>, code: Vec<u8>, data: Vec<u8>) -> Self {
        let mut file = Self {
            header: MlibHeader {
                magic: MLIB_MAGIC,
                version: 1,
                abi_version: CURRENT_ABI_VERSION,
                target,
                flags: 0,
                export_table_offset: 0,
                export_table_count: exports.len() as u32,
                string_table_offset: 0,
                string_table_size: 0,
                code_offset: 0,
                code_size: code.len() as u64,
                data_offset: 0,
                data_size: data.len() as u64,
            },
            exports,
            string_table: Vec::new(),
            code,
            data,
        };
        file.rebuild_layout();
        file
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let (string_table, export_table) = build_string_table_and_exports(&self.exports);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.extend_from_slice(&export_table);
        bytes.extend_from_slice(&string_table);
        bytes.extend_from_slice(&self.code);
        bytes.extend_from_slice(&self.data);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < MLIB_HEADER_SIZE {
            return None;
        }

        let header = MlibHeader::from_bytes(bytes[..MLIB_HEADER_SIZE].try_into().ok()?)?;
        if header.magic != MLIB_MAGIC {
            return None;
        }

        let export_table_start = header.export_table_offset as usize;
        let export_table_end =
            export_table_start + header.export_table_count as usize * MLIB_EXPORT_ENTRY_SIZE;
        let string_table_start = header.string_table_offset as usize;
        let string_table_end = string_table_start + header.string_table_size as usize;
        let code_start = header.code_offset as usize;
        let code_end = code_start + header.code_size as usize;
        let data_start = header.data_offset as usize;
        let data_end = data_start + header.data_size as usize;

        if export_table_end > bytes.len()
            || string_table_end > bytes.len()
            || code_end > bytes.len()
            || data_end > bytes.len()
        {
            return None;
        }

        let string_table = bytes[string_table_start..string_table_end].to_vec();
        let mut exports = Vec::new();
        for index in 0..header.export_table_count as usize {
            let offset = export_table_start + index * MLIB_EXPORT_ENTRY_SIZE;
            let entry = &bytes[offset..offset + MLIB_EXPORT_ENTRY_SIZE];
            let name_offset = u32::from_le_bytes(entry[0..4].try_into().ok()?) as usize;
            let name_len = u16::from_le_bytes(entry[4..6].try_into().ok()?) as usize;
            let kind = MlibExportKind::from_byte(entry[6])?;
            let address = u64::from_le_bytes(entry[8..16].try_into().ok()?);
            let signature_offset = u32::from_le_bytes(entry[16..20].try_into().ok()?) as usize;
            let signature_len = u32::from_le_bytes(entry[20..24].try_into().ok()?) as usize;

            let name = read_string(&string_table, name_offset, name_len)?;
            let signature = if signature_len == 0 {
                None
            } else {
                Some(string_table.get(signature_offset..signature_offset + signature_len)?.to_vec())
            };

            exports.push(MlibExport {
                name,
                kind,
                address,
                signature,
            });
        }

        Some(Self {
            header,
            exports,
            string_table,
            code: bytes[code_start..code_end].to_vec(),
            data: bytes[data_start..data_end].to_vec(),
        })
    }

    pub fn embedded_source(&self) -> Option<&str> {
        if self.header.flags & MLIB_FLAG_EMBEDDED_SOURCE == 0 {
            return None;
        }
        std::str::from_utf8(&self.data).ok()
    }

    fn rebuild_layout(&mut self) {
        let (string_table, _) = build_string_table_and_exports(&self.exports);
        self.string_table = string_table;
        let export_table_size = (self.exports.len() * MLIB_EXPORT_ENTRY_SIZE) as u64;
        let string_table_size = self.string_table.len() as u64;

        self.header.export_table_offset = MLIB_HEADER_SIZE as u64;
        self.header.export_table_count = self.exports.len() as u32;
        self.header.string_table_offset = self.header.export_table_offset + export_table_size;
        self.header.string_table_size = string_table_size;
        self.header.code_offset = self.header.string_table_offset + string_table_size;
        self.header.code_size = self.code.len() as u64;
        self.header.data_offset = self.header.code_offset + self.header.code_size;
        self.header.data_size = self.data.len() as u64;
    }
}

impl MlibHeader {
    pub fn to_bytes(&self) -> [u8; MLIB_HEADER_SIZE] {
        let mut bytes = [0u8; MLIB_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.version.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.abi_version.major.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.abi_version.minor.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.abi_version.patch.to_le_bytes());
        bytes[12..14].copy_from_slice(&encode_target(self.target).to_le_bytes());
        bytes[14..18].copy_from_slice(&self.flags.to_le_bytes());
        bytes[18..26].copy_from_slice(&self.export_table_offset.to_le_bytes());
        bytes[26..30].copy_from_slice(&self.export_table_count.to_le_bytes());
        bytes[30..38].copy_from_slice(&self.string_table_offset.to_le_bytes());
        bytes[38..46].copy_from_slice(&self.string_table_size.to_le_bytes());
        bytes[46..54].copy_from_slice(&self.code_offset.to_le_bytes());
        bytes[54..62].copy_from_slice(&self.code_size.to_le_bytes());
        bytes[62..70].copy_from_slice(&self.data_offset.to_le_bytes());
        bytes[70..78].copy_from_slice(&self.data_size.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: [u8; MLIB_HEADER_SIZE]) -> Option<Self> {
        let target = decode_target(u16::from_le_bytes([bytes[12], bytes[13]]))?;
        Some(Self {
            magic: bytes[0..4].try_into().ok()?,
            version: u16::from_le_bytes([bytes[4], bytes[5]]),
            abi_version: AbiVersion::new(
                u16::from_le_bytes([bytes[6], bytes[7]]),
                u16::from_le_bytes([bytes[8], bytes[9]]),
                u16::from_le_bytes([bytes[10], bytes[11]]),
            ),
            target,
            flags: u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]),
            export_table_offset: u64::from_le_bytes([
                bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23], bytes[24],
                bytes[25],
            ]),
            export_table_count: u32::from_le_bytes([bytes[26], bytes[27], bytes[28], bytes[29]]),
            string_table_offset: u64::from_le_bytes([
                bytes[30], bytes[31], bytes[32], bytes[33], bytes[34], bytes[35], bytes[36],
                bytes[37],
            ]),
            string_table_size: u64::from_le_bytes([
                bytes[38], bytes[39], bytes[40], bytes[41], bytes[42], bytes[43], bytes[44],
                bytes[45],
            ]),
            code_offset: u64::from_le_bytes([
                bytes[46], bytes[47], bytes[48], bytes[49], bytes[50], bytes[51], bytes[52],
                bytes[53],
            ]),
            code_size: u64::from_le_bytes([
                bytes[54], bytes[55], bytes[56], bytes[57], bytes[58], bytes[59], bytes[60],
                bytes[61],
            ]),
            data_offset: u64::from_le_bytes([
                bytes[62], bytes[63], bytes[64], bytes[65], bytes[66], bytes[67], bytes[68],
                bytes[69],
            ]),
            data_size: u64::from_le_bytes([
                bytes[70], bytes[71], bytes[72], bytes[73], bytes[74], bytes[75], bytes[76],
                bytes[77],
            ]),
        })
    }
}

fn build_string_table_and_exports(exports: &[MlibExport]) -> (Vec<u8>, Vec<u8>) {
    let mut strings = Vec::new();
    let mut entries = Vec::with_capacity(exports.len() * MLIB_EXPORT_ENTRY_SIZE);

    for export in exports {
        let name_offset = strings.len() as u32;
        let name_len = export.name.len() as u16;
        strings.extend_from_slice(export.name.as_bytes());
        strings.push(0);

        let (signature_offset, signature_len) = if let Some(signature) = &export.signature {
            let offset = strings.len() as u32;
            let len = signature.len() as u32;
            strings.extend_from_slice(signature);
            strings.push(0);
            (offset, len)
        } else {
            (0, 0)
        };

        entries.extend_from_slice(&name_offset.to_le_bytes());
        entries.extend_from_slice(&name_len.to_le_bytes());
        entries.push(export.kind.to_byte());
        entries.push(0);
        entries.extend_from_slice(&export.address.to_le_bytes());
        entries.extend_from_slice(&signature_offset.to_le_bytes());
        entries.extend_from_slice(&signature_len.to_le_bytes());
    }

    (strings, entries)
}

fn read_string(strings: &[u8], offset: usize, len: usize) -> Option<String> {
    let bytes = strings.get(offset..offset + len)?;
    std::str::from_utf8(bytes).ok().map(|value| value.to_string())
}

fn encode_target(target: AbiTarget) -> u16 {
    match target {
        AbiTarget::LinuxX86_64 => 1,
        AbiTarget::WindowsX86_64 => 2,
    }
}

fn decode_target(value: u16) -> Option<AbiTarget> {
    match value {
        1 => Some(AbiTarget::LinuxX86_64),
        2 => Some(AbiTarget::WindowsX86_64),
        _ => None,
    }
}
