use crate::report::GateError;
use crate::review::diff::{MAXIMUM_DIFF_BYTES, reject_oversized_diff};
use crate::review::github::client::GitHub;

impl GitHub {
    pub fn read_pull_request_diff(
        &self,
        pull_request: u64,
        head_sha: &str,
    ) -> Result<String, GateError> {
        let base = self.read_base_branch(pull_request)?;
        let diff = self.api_limited(
            &format!("compare/{base}...{head_sha}"),
            &["-H", "Accept: application/vnd.github.diff"],
            MAXIMUM_DIFF_BYTES,
        )?;
        reject_oversized_diff(&diff)?;
        Ok(diff)
    }

    fn read_base_branch(&self, pull_request: u64) -> Result<String, GateError> {
        let output = self.api(&format!("pulls/{pull_request}"), &["--jq", ".base.ref"])?;
        Ok(output.trim().to_owned())
    }
}
