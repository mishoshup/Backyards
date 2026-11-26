use crate::report::GateError;
use crate::review::github::client::GitHub;

impl GitHub {
    pub fn add_comment_reaction(&self, comment_id: &str, content: &str) -> Result<(), GateError> {
        let content_field = format!("content={content}");
        self.call_api(
            &format!("issues/comments/{comment_id}/reactions"),
            &["-f", &content_field],
        )
        .map(|_| ())
    }

    pub fn remove_comment_reaction(
        &self,
        comment_id: &str,
        content: &str,
        reactor_login: &str,
    ) -> Result<(), GateError> {
        let route = format!("issues/comments/{comment_id}/reactions");
        let filter = format!(
            "[.[] | select(.content == \"{content}\" and .user.login == \"{reactor_login}\")] | first | .id // empty"
        );
        let reaction_identifier = self.call_api(&route, &["--jq", &filter])?;
        let reaction_identifier = reaction_identifier.trim();
        if reaction_identifier.is_empty() {
            return Ok(());
        }
        self.call_api(&format!("{route}/{reaction_identifier}"), &["-X", "DELETE"])
            .map(|_| ())
    }
}
