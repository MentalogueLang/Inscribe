#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiType {
    Unit,
    Int,
    Float,
    Bool,
    Error,
    Pointer,
    Struct(StructLayout),
    Result(Box<AbiType>, Box<AbiType>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub ty: AbiType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub size: u32,
    pub align: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldLayout {
    pub name: String,
    pub offset: u32,
    pub layout: Layout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructMemoryLayout {
    pub layout: Layout,
    pub fields: Vec<FieldLayout>,
}

impl Layout {
    pub const fn new(size: u32, align: u32) -> Self {
        Self { size, align }
    }
}

impl AbiType {
    pub fn layout(&self) -> Layout {
        match self {
            Self::Unit => Layout::new(0, 1),
            Self::Bool => Layout::new(1, 1),
            Self::Int | Self::Float | Self::Error | Self::Pointer => Layout::new(8, 8),
            Self::Struct(struct_layout) => struct_layout.memory_layout().layout,
            Self::Result(ok, err) => result_layout(ok.layout(), err.layout()),
        }
    }
}

impl StructLayout {
    pub fn new(name: impl Into<String>, fields: Vec<StructField>) -> Self {
        Self {
            name: name.into(),
            fields,
        }
    }

    pub fn memory_layout(&self) -> StructMemoryLayout {
        let mut offset = 0u32;
        let mut align = 1u32;
        let mut fields = Vec::with_capacity(self.fields.len());

        for field in &self.fields {
            let field_layout = field.ty.layout();
            offset = align_to(offset, field_layout.align);
            align = align.max(field_layout.align);
            fields.push(FieldLayout {
                name: field.name.clone(),
                offset,
                layout: field_layout,
            });
            offset += field_layout.size;
        }

        StructMemoryLayout {
            layout: Layout::new(align_to(offset, align), align),
            fields,
        }
    }
}

impl StructMemoryLayout {
    pub fn field(&self, name: &str) -> Option<&FieldLayout> {
        self.fields.iter().find(|field| field.name == name)
    }
}

fn result_layout(ok: Layout, err: Layout) -> Layout {
    let payload_align = ok.align.max(err.align);
    let payload_size = ok.size.max(err.size);
    let tag_size = 1u32;
    let tag_aligned = align_to(tag_size, payload_align);
    Layout::new(
        align_to(tag_aligned + payload_size, payload_align),
        payload_align,
    )
}

fn align_to(value: u32, align: u32) -> u32 {
    if align <= 1 {
        value
    } else {
        ((value + align - 1) / align) * align
    }
}
