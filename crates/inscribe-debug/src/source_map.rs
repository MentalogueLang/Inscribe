use inscribe_ast::span::{Position, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceFileId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: SourceFileId,
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRange {
    pub file: SourceFileId,
    pub start: SourceLocation,
    pub end: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub id: SourceFileId,
    pub name: String,
    pub source: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    fn new(id: SourceFileId, name: String, source: String) -> Self {
        Self {
            id,
            name,
            line_starts: compute_line_starts(&source),
            source,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, name: impl Into<String>, source: impl Into<String>) -> SourceFileId {
        let id = SourceFileId(self.files.len());
        self.files.push(SourceFile::new(id, name.into(), source.into()));
        id
    }

    pub fn file(&self, id: SourceFileId) -> Option<&SourceFile> {
        self.files.get(id.0)
    }

    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    pub fn lookup_offset(&self, id: SourceFileId, offset: usize) -> Option<SourceLocation> {
        let file = self.file(id)?;
        let line_index = file.line_starts.partition_point(|start| *start <= offset);
        let line_start_index = line_index.saturating_sub(1);
        let line_start = *file.line_starts.get(line_start_index)?;
        Some(SourceLocation {
            file: id,
            offset,
            line: line_start_index + 1,
            column: offset.saturating_sub(line_start) + 1,
        })
    }

    pub fn lookup_position(&self, id: SourceFileId, position: Position) -> Option<SourceLocation> {
        self.lookup_offset(id, position.offset).or_else(|| {
            Some(SourceLocation {
                file: id,
                offset: position.offset,
                line: position.line,
                column: position.column,
            })
        })
    }

    pub fn resolve_span(&self, id: SourceFileId, span: Span) -> Option<SourceRange> {
        Some(SourceRange {
            file: id,
            start: self.lookup_position(id, span.start)?,
            end: self.lookup_position(id, span.end)?,
        })
    }

    pub fn snippet(&self, id: SourceFileId, span: Span) -> Option<&str> {
        let file = self.file(id)?;
        if span.start.offset > span.end.offset || span.end.offset > file.source.len() {
            return None;
        }
        file.source.get(span.start.offset..span.end.offset)
    }
}

fn compute_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, ch) in source.char_indices() {
        if ch == '\n' {
            starts.push(index + 1);
        }
    }
    starts
}

#[cfg(test)]
mod tests {
    use inscribe_ast::span::{Position, Span};

    use crate::source_map::SourceMap;

    #[test]
    fn resolves_offsets_and_snippets() {
        let mut map = SourceMap::new();
        let file = map.add_file("main.ins", "let answer = 42\nanswer\n");
        let span = Span::new(Position::new(4, 1, 5), Position::new(10, 1, 11));
        let range = map.resolve_span(file, span).expect("span should resolve");

        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.column, 5);
        assert_eq!(range.end.column, 11);
        assert_eq!(map.snippet(file, span), Some("answer"));
    }
}
