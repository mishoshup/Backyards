const FILE_HEADER_PREFIX: &str = "diff --git ";

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
    let mut starts = Vec::new();
    let mut offset = 0;
    for line in diff.split_inclusive('\n') {
        if line.starts_with(FILE_HEADER_PREFIX) {
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
    let hunk_start = section
        .find("\n@@ ")
        .map_or(section.len(), |offset| offset + 1);
    let (header, hunks) = section.split_at(hunk_start);
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
    let mut starts: Vec<usize> = Vec::new();
    let mut offset = 0;
    for line in hunks.split_inclusive('\n') {
        if line.starts_with("@@ ") {
            starts.push(offset);
        }
        offset += line.len();
    }
    starts.push(hunks.len());
    starts
        .windows(2)
        .map(|pair| &hunks[pair[0]..pair[1]])
        .collect()
}
