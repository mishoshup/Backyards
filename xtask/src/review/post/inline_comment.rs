use crate::report::GateError;
use crate::review::github::GitHub;
use crate::review::model::Violation;
use crate::review::post::finding_text::describe_finding_body;
use crate::review::post::footer::build_footer;

#[derive(Debug, Clone, Copy)]
pub struct PostTarget<'target> {
    pub pull_request: u64,
    pub head_sha: &'target str,
}

pub fn post_inline_comment(
    github: &GitHub,
    target: PostTarget<'_>,
    violation: &Violation,
) -> Result<(), GateError> {
    let body = format!(
        "{}{}",
        describe_finding_body(violation),
        build_footer(github.repository(), target.head_sha)
    );
    let body_field = format!("body={body}");
    let path_field = format!("path={}", violation.path);
    let line_field = format!("line={}", violation.line);
    let commit_field = format!("commit_id={}", target.head_sha);
    github
        .api(
            &format!("pulls/{}/comments", target.pull_request),
            &[
                "-X",
                "POST",
                "-f",
                &body_field,
                "-f",
                &path_field,
                "-F",
                &line_field,
                "-f",
                "side=RIGHT",
                "-f",
                &commit_field,
            ],
        )
        .map(|_| ())
}
