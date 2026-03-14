use crate::boundary::ComptimeValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComptimeAllocId(pub usize);

#[derive(Debug, Clone, Default)]
pub struct ComptimeAllocator {
    slots: Vec<ComptimeValue>,
}

impl ComptimeAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc(&mut self, value: ComptimeValue) -> ComptimeAllocId {
        let id = ComptimeAllocId(self.slots.len());
        self.slots.push(value);
        id
    }

    pub fn get(&self, id: ComptimeAllocId) -> Option<&ComptimeValue> {
        self.slots.get(id.0)
    }

    pub fn get_mut(&mut self, id: ComptimeAllocId) -> Option<&mut ComptimeValue> {
        self.slots.get_mut(id.0)
    }

    pub fn replace(&mut self, id: ComptimeAllocId, value: ComptimeValue) -> Option<ComptimeValue> {
        self.slots
            .get_mut(id.0)
            .map(|slot| std::mem::replace(slot, value))
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn clear(&mut self) {
        self.slots.clear();
    }
}
