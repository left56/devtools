//! JSON execution state and error positioning, independent of GPUI.
use devtools_tools::json::{self, FormatOptions, IndentStyle, JsonDocument, JsonError};

#[derive(Clone, Copy)]
pub enum JsonAction {
    Prettify,
    Minify,
}

#[derive(Default)]
pub struct JsonRun {
    pub output: Option<String>,
    pub document: Option<JsonDocument>,
    pub error: Option<JsonError>,
    pub error_offset: Option<usize>,
}

/// Format with exact options (used by live preview).
pub fn run_json_opts(input: &str, opts: FormatOptions) -> JsonRun {
    match json::format_with_options(input, opts) {
        Ok(output) => {
            let document = JsonDocument::parse(&output).ok();
            JsonRun {
                output: Some(output),
                document,
                ..JsonRun::default()
            }
        }
        Err(error) => {
            let mut start = 0;
            let mut line = "";
            for (i, value) in input.split('\n').enumerate() {
                line = value;
                if i + 1 >= error.line {
                    break;
                }
                start += value.len() + 1;
            }
            let column = error.column.saturating_sub(1).min(line.len());
            let offset = start.min(input.len()) + line.floor_char_boundary(column);
            JsonRun {
                output: None,
                document: None,
                error: Some(error),
                error_offset: Some(offset.min(input.len())),
            }
        }
    }
}

pub fn run_json(input: &str, action: JsonAction, opts: FormatOptions) -> JsonRun {
    let mut opts = opts;
    match action {
        JsonAction::Minify => opts.indent = IndentStyle::Minified,
        JsonAction::Prettify => {
            if matches!(opts.indent, IndentStyle::Minified) {
                opts.indent = IndentStyle::Spaces2;
            }
        }
    }
    run_json_opts(input, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use devtools_tools::json;

    #[test]
    fn output_is_exact_tool_result_in_input_key_order() {
        let input = r#"{"b":1,"a":[true,null],"中文":"😀"}"#;
        let opts = FormatOptions::default_options();
        let pretty = run_json(input, JsonAction::Prettify, opts);
        assert_eq!(
            pretty.output.as_deref(),
            Some(json::prettify(input).unwrap().as_str())
        );
        assert_eq!(
            run_json(pretty.output.as_deref().unwrap(), JsonAction::Minify, opts)
                .output
                .as_deref(),
            Some(input)
        );
        assert!(pretty.error.is_none());
    }

    #[test]
    fn invalid_json_never_produces_repaired_output() {
        let opts = FormatOptions::default_options();
        for input in ["", "   ", r#"{"a":1,}"#, "[1,]", "{\n\"你好\": ☃\n}"] {
            for action in [JsonAction::Prettify, JsonAction::Minify] {
                let run = run_json(input, action, opts);
                assert!(run.output.is_none());
                assert!(run.document.is_none());
                assert!(run.error.is_some());
                let offset = run.error_offset.unwrap();
                assert!(offset <= input.len() && input.is_char_boundary(offset));
            }
        }
        let run = run_json("[1,]", JsonAction::Prettify, opts);
        assert_eq!(run.error.as_ref().unwrap().column, 4);
        assert_eq!(run.error_offset, Some(3));
        let run = run_json("", JsonAction::Prettify, opts);
        assert_eq!(run.error.as_ref().map(|e| (e.line, e.column)), Some((1, 0)));
        assert_eq!(run.error_offset, Some(0));
    }

    #[test]
    fn auto_repair_opt_in_accepts_trailing_comma() {
        let mut opts = FormatOptions::default_options();
        opts.auto_repair = true;
        opts.indent = IndentStyle::Minified;
        let run = run_json_opts(r#"{"a":1,}"#, opts);
        assert_eq!(run.output.as_deref(), Some(r#"{"a":1}"#));
        assert!(run.error.is_none());
    }
}
