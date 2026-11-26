use xtask::review::{MAXIMUM_DIFF_BYTES, parse_added_lines, reject_oversized_diff};

const SAMPLE_DIFF: &str = "\
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,3 @@
 fn one() {}
+fn two() {}
-fn old() {}
 fn three() {}
@@ -10 +11,2 @@ fn context()
 keep
+added after keep
diff --git a/gone.rs b/gone.rs
deleted file mode 100644
--- a/gone.rs
+++ /dev/null
@@ -1 +0,0 @@
-removed
diff --git a/new.rs b/new.rs
new file mode 100644
--- /dev/null
+++ b/new.rs
@@ -0,0 +1,2 @@
+first
+++ not a header
";

#[test]
fn added_lines_use_new_file_numbering() {
    let added = parse_added_lines(SAMPLE_DIFF).unwrap();
    assert!(added.contains("src/lib.rs", 2));
    assert!(added.contains("src/lib.rs", 12));
    assert!(!added.contains("src/lib.rs", 1));
    assert!(!added.contains("src/lib.rs", 3));
}

#[test]
fn deleted_files_contribute_no_lines() {
    let added = parse_added_lines(SAMPLE_DIFF).unwrap();
    assert!(!added.contains_path("gone.rs"));
}

#[test]
fn content_that_looks_like_a_file_header_stays_inside_its_hunk() {
    let added = parse_added_lines(SAMPLE_DIFF).unwrap();
    assert!(added.contains("new.rs", 1));
    assert!(added.contains("new.rs", 2));
    assert!(!added.contains_path("not a header"));
}

#[test]
fn empty_diff_has_no_added_lines() {
    assert!(parse_added_lines("").unwrap().is_empty());
}

#[test]
fn malformed_hunk_header_is_reported() {
    let error = parse_added_lines("+++ b/a.rs\n@@ -x +1 @@\n").unwrap_err();
    assert!(error.to_string().contains("malformed hunk header"));
}

#[test]
fn unexpected_line_inside_a_hunk_is_reported() {
    let error = parse_added_lines("+++ b/a.rs\n@@ -1 +1 @@\n?bad\n").unwrap_err();
    assert!(error.to_string().contains("malformed line"));
}

#[test]
fn diff_over_the_limit_is_rejected_with_its_size() {
    let diff = "x".repeat(MAXIMUM_DIFF_BYTES + 1);
    let error = reject_oversized_diff(&diff).unwrap_err();
    let message = error.to_string();
    assert!(message.contains(&(MAXIMUM_DIFF_BYTES + 1).to_string()));
    assert!(message.contains(&MAXIMUM_DIFF_BYTES.to_string()));
    assert!(message.contains("split the pull request"));
    assert!(reject_oversized_diff("small").is_ok());
}
