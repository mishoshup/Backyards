pub const OUTPUT_CONTRACT: &str = r#"Respond with JSON only, in this shape:
{"violations":[{"ruleId":"<rule identifier, or the area name for a holistic finding>","path":"<file path>","line":<line number in the new file>,"body":"<rule identifier, defect and smallest correction>","confidence":"<high, or low when you are unsure>"}]}
The body of an inline violation must contain its rule identifier.
Report a finding you are unsure about with confidence low. Report a confirmed finding with confidence high.
Do not report a model name, API name, flag or library behavior as wrong unless the diff itself shows the failure.
Always answer with the JSON object. Use {"violations":[]} only as a definitive statement that you checked the diff and found nothing to report. Report only lines present in the diff."#;
