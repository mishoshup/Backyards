const FILE_HEADER_PREFIX: &str = "diff --git ";

#[must_use]
pub fn split_diff_into_shards(diff: &str, maximum_shard_bytes: usize) -> Vec<String> {
    let mut shards: Vec<String> = Vec::new();
    for section in split_into_file_sections(diff) {
        match shards.last_mut() {
            Some(current) if current.len() + section.len() <= maximum_shard_bytes => {
                current.push_str(section);
            }
            _ => shards.push(section.to_owned()),
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
