use crate::calling_conv::AbiTarget;
use crate::versioning::{AbiVersion, CURRENT_ABI_VERSION};

pub const MLIB_MAGIC: [u8; 4] = *b"MLIB";
pub const MLIB_HEADER_SIZE: usize = 80;
pub const MLIB_EXPORT_ENTRY_SIZE: usize = 24;

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
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.to_bytes());

        bytes.extend_from_slice(&self.export_table_bytes());
        bytes.extend_from_slice(&self.string_table);
        bytes.extend_from_slice(&self.code);
        bytes.extend_from_slice(&self.data);
        bytes
    }

    fn export_table_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.exports.len() * MLIB_EXPORT_ENTRY_SIZE);
        let mut string_cursor = 0u32;
        for export in &self.exports {
            let name_bytes = export.name.as_bytes();
            let name_offset = string_cursor;
            let name_len = name_bytes.len() as u16;
            string_cursor = string_cursor.saturating_add(name_bytes.len() as u32 + 1);

            bytes.extend_from_slice(&name_offset.to_le_bytes());
            bytes.extend_from_slice(&name_len.to_le_bytes());
            bytes.push(export.kind.to_byte());
            bytes.push(0);
            bytes.extend_from_slice(&export.address.to_le_bytes());

            let signature_offset = export
                .signature
                .as_ref()
                .map(|_| 0u32)
                .unwrap_or(0u32);
            bytes.extend_from_slice(&signature_offset.to_le_bytes());
        }
        bytes
    }

    fn rebuild_layout(&mut self) {
        self.string_table = build_string_table(&self.exports);
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

fn build_string_table(exports: &[MlibExport]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for export in exports {
        bytes.extend_from_slice(export.name.as_bytes());
        bytes.push(0);
    }
    bytes
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
