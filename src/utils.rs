#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LRange {
    pub start: Location,
    pub end: Location,
}

impl<'a> Location {
    pub fn from_offset(s: &'a str, offset: usize) -> Self {
        let mut current = 0usize;
        let lines: Vec<_> = s.split('\n').into_iter().collect();
        let mut line = 0usize;
        if offset > s.len() {
            let line = lines.len() + 1;
            let col = 0;

            Self { line, col }
        } else {
            while line < lines.len() && current + lines[line].len() < offset {
                current += lines[line].len();
                line += 1;
            }
            if line == lines.len() {
                let col = lines[line - 1].len() + 1;
                Self { line, col }
            } else {
                line += 1;
                let col = offset - current + 1;
                Self { line, col }
            }
        }
    }
}

impl<'a> LRange {
    pub fn from_offset(s: &'a str, start: usize, end: usize) -> Self {
        let start = Location::from_offset(s, start);
        let end = Location::from_offset(s, end);

        Self { start, end }
    }
}
