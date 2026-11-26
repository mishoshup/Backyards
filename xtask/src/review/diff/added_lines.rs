use std::collections::{BTreeMap, BTreeSet};

use crate::review::error::ReviewError;

const NEW_FILE_PREFIX: &str = "+++ ";
const DELETED_FILE_TARGET: &str = "/dev/null";
const NEW_SIDE_PATH_PREFIX: &str = "b/";
const HUNK_HEADER_PREFIX: &str = "@@ -";
const HUNK_HEADER_SUFFIX: &str = " @@";
const DEFAULT_RANGE_COUNT: u32 = 1;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AddedLines {
    added_by_path: BTreeMap<String, BTreeSet<u32>>,
    context_by_path: BTreeMap<String, BTreeSet<u32>>,
}

impl AddedLines {
    #[must_use]
    pub fn contains(&self, path: &str, line: u32) -> bool {
        has_line(&self.added_by_path, path, line)
    }

    #[must_use]
    pub fn contains_visible(&self, path: &str, line: u32) -> bool {
        self.contains(path, line) || has_line(&self.context_by_path, path, line)
    }

    #[must_use]
    pub fn contains_path(&self, path: &str) -> bool {
        self.added_by_path.contains_key(path) || self.context_by_path.contains_key(path)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added_by_path.is_empty() && self.context_by_path.is_empty()
    }

    fn record(&mut self, path: &str, line: u32) {
        record_line(&mut self.added_by_path, path, line);
    }

    fn record_context(&mut self, path: &str, line: u32) {
        record_line(&mut self.context_by_path, path, line);
    }
}

fn has_line(lines_by_path: &BTreeMap<String, BTreeSet<u32>>, path: &str, line: u32) -> bool {
    lines_by_path
        .get(path)
        .is_some_and(|lines| lines.contains(&line))
}

fn record_line(lines_by_path: &mut BTreeMap<String, BTreeSet<u32>>, path: &str, line: u32) {
    lines_by_path
        .entry(path.to_owned())
        .or_default()
        .insert(line);
}

pub fn parse_added_lines(diff: &str) -> Result<AddedLines, ReviewError> {
    let mut parser = DiffParser::default();
    for line in diff.lines() {
        parser.consume_line(line)?;
    }
    Ok(parser.added)
}

#[derive(Debug, Clone, Copy)]
struct HunkProgress {
    remaining_old: u32,
    remaining_new: u32,
    next_new_line: u32,
}

impl HunkProgress {
    const fn is_finished(self) -> bool {
        self.remaining_old == 0 && self.remaining_new == 0
    }

    const fn advance_after_addition(self) -> Self {
        Self {
            remaining_new: self.remaining_new.saturating_sub(1),
            next_new_line: self.next_new_line + 1,
            ..self
        }
    }

    const fn advance_after_removal(self) -> Self {
        Self {
            remaining_old: self.remaining_old.saturating_sub(1),
            ..self
        }
    }

    const fn advance_after_context(self) -> Self {
        Self {
            remaining_old: self.remaining_old.saturating_sub(1),
            remaining_new: self.remaining_new.saturating_sub(1),
            next_new_line: self.next_new_line + 1,
        }
    }
}

#[derive(Debug, Default)]
struct DiffParser {
    added: AddedLines,
    current_path: Option<String>,
    hunk: Option<HunkProgress>,
}

impl DiffParser {
    fn consume_line(&mut self, line: &str) -> Result<(), ReviewError> {
        match self.hunk {
            Some(progress) if !progress.is_finished() => self.consume_hunk_line(line, progress),
            _ => self.consume_header_line(line),
        }
    }

    fn consume_hunk_line(&mut self, line: &str, progress: HunkProgress) -> Result<(), ReviewError> {
        let updated = match line.chars().next() {
            Some('+') => {
                self.record_added_line(progress.next_new_line);
                progress.advance_after_addition()
            }
            Some('-') => progress.advance_after_removal(),
            Some(' ') | None => {
                self.record_context_line(progress.next_new_line);
                progress.advance_after_context()
            }
            Some('\\') => progress,
            Some(_) => {
                return Err(ReviewError::MalformedDiff {
                    line: line.to_owned(),
                });
            }
        };
        self.hunk = Some(updated);
        Ok(())
    }

    fn consume_header_line(&mut self, line: &str) -> Result<(), ReviewError> {
        if let Some(target) = line.strip_prefix(NEW_FILE_PREFIX) {
            self.current_path = parse_target_path(target);
        } else if line.starts_with(HUNK_HEADER_PREFIX) {
            self.hunk = Some(parse_hunk_header(line)?);
        }
        Ok(())
    }

    fn record_context_line(&mut self, line: u32) {
        if let Some(path) = &self.current_path {
            self.added.record_context(path, line);
        }
    }

    fn record_added_line(&mut self, line: u32) {
        if let Some(path) = &self.current_path {
            self.added.record(path, line);
        }
    }
}

fn parse_target_path(target: &str) -> Option<String> {
    let path = unquote_git_path(target.split('\t').next().unwrap_or(target));
    if path == DELETED_FILE_TARGET {
        return None;
    }
    Some(
        path.strip_prefix(NEW_SIDE_PATH_PREFIX)
            .unwrap_or(&path)
            .to_owned(),
    )
}

fn unquote_git_path(path: &str) -> String {
    let Some(inner) = path
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return path.to_owned();
    };
    let mut bytes = Vec::new();
    let mut characters = inner.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\\' {
            bytes.push(read_escape(&mut characters));
        } else {
            let mut buffer = [0_u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn read_escape(characters: &mut std::iter::Peekable<std::str::Chars<'_>>) -> u8 {
    match characters.next() {
        Some('t') => b'\t',
        Some('n') => b'\n',
        Some(digit @ '0'..='3') => read_octal_escape(digit, characters),
        Some(other) => u8::try_from(other).unwrap_or(b'?'),
        None => b'\\',
    }
}

fn read_octal_escape(first: char, characters: &mut std::iter::Peekable<std::str::Chars<'_>>) -> u8 {
    let mut value = first.to_digit(8).unwrap_or(0);
    for _ in 0..2 {
        if let Some(digit) = characters.next_if(|candidate| candidate.is_digit(8)) {
            value = value * 8 + digit.to_digit(8).unwrap_or(0);
        }
    }
    u8::try_from(value).unwrap_or(b'?')
}

fn parse_hunk_header(header: &str) -> Result<HunkProgress, ReviewError> {
    parse_hunk_ranges(header).ok_or_else(|| ReviewError::MalformedHunkHeader {
        header: header.to_owned(),
    })
}

fn parse_hunk_ranges(header: &str) -> Option<HunkProgress> {
    let body = header.strip_prefix(HUNK_HEADER_PREFIX)?;
    let (ranges, _) = body.split_once(HUNK_HEADER_SUFFIX)?;
    let (old_range, new_range) = ranges.split_once(" +")?;
    let (_, old_count) = parse_range(old_range)?;
    let (new_start, new_count) = parse_range(new_range)?;
    Some(HunkProgress {
        remaining_old: old_count,
        remaining_new: new_count,
        next_new_line: new_start,
    })
}

fn parse_range(text: &str) -> Option<(u32, u32)> {
    match text.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((text.parse().ok()?, DEFAULT_RANGE_COUNT)),
    }
}
