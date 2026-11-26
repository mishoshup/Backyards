use xtask::review::split_diff_into_shards;

fn file_section(name: &str, hunks: usize) -> String {
    let header = format!("diff --git a/{name} b/{name}\n--- a/{name}\n+++ b/{name}\n");
    let hunk = "@@ -1,1 +1,1 @@\n-old\n+new\n".repeat(hunks);
    header + &hunk
}

#[test]
fn small_files_are_packed_into_one_shard() {
    let diff = format!("{}{}", file_section("a.rs", 1), file_section("b.rs", 1));
    assert_eq!(split_diff_into_shards(&diff, 10_000).len(), 1);
}

#[test]
fn files_over_the_limit_start_separate_shards() {
    let diff = format!("{}{}", file_section("a.rs", 2), file_section("b.rs", 2));
    let shards = split_diff_into_shards(&diff, file_section("a.rs", 2).len());
    assert_eq!(shards.len(), 2);
}

#[test]
fn oversized_file_is_split_at_hunks_with_its_header() {
    let diff = file_section("big.rs", 20);
    let shards = split_diff_into_shards(&diff, 300);
    assert!(shards.len() > 1);
    assert!(
        shards
            .iter()
            .all(|shard| shard.starts_with("diff --git a/big.rs"))
    );
}
