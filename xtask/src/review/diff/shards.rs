const FILE_HEADER_PREFIX: &str = "diff --git ";
const HUNK_START_MARKER: &str = "@@ ";
const HUNK_START_AFTER_NEWLINE: &str = "\n@@ ";

#[must_use]
pub fn split_diff_into_shards(diff: &str, maximum_shard_bytes: usize) -> Vec<String> {
    let mut shards: Vec<String> = Vec::new();
    for section in split_into_file_sections(diff)
        .into_iter()
        .flat_map(|section| split_oversized_section(section, maximum_shard_bytes))
    {
        match shards.last_mut() {
            Some(current) if current.len() + section.len() <= maximum_shard_bytes => {
                current.push_str(&section);
            }
            _ => shards.push(section),
        }
    }
    shards
}

fn split_into_file_sections(diff: &str) -> Vec<&str> {
    let starts = find_section_starts(diff);
    let mut boundaries = starts.clone();
    boundaries.push(diff.len());
    starts
        .iter()
        .zip(boundaries.iter().skip(1))
        .map(|(start, end)| &diff[*start..*end])
        .collect()
}

fn find_section_starts(diff: &str) -> Vec<usize> {
    find_line_starts(diff, FILE_HEADER_PREFIX)
}

fn find_line_starts(text: &str, prefix: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if line.starts_with(prefix) {
            starts.push(offset);
        }
        offset += line.len();
    }
    starts
}

fn split_oversized_section(section: &str, maximum_bytes: usize) -> Vec<String> {
    if section.len() <= maximum_bytes {
        return vec![section.to_owned()];
    }
    let Some(offset) = section.find(HUNK_START_AFTER_NEWLINE) else {
        return vec![section.to_owned()];
    };
    let (header, hunks) = section.split_at(offset + 1);
    pack_hunks(header, hunks, maximum_bytes)
}

fn pack_hunks(header: &str, hunks: &str, maximum_bytes: usize) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    for hunk in split_hunks(hunks) {
        match pieces.last_mut() {
            Some(current) if current.len() + hunk.len() <= maximum_bytes => current.push_str(hunk),
            _ => pieces.push(format!("{header}{hunk}")),
        }
    }
    pieces
}

fn split_hunks(hunks: &str) -> Vec<&str> {
    let mut starts = find_line_starts(hunks, HUNK_START_MARKER);
    starts.push(hunks.len());
    starts
        .windows(2)
        .map(|pair| &hunks[pair[0]..pair[1]])
        .collect()
}
