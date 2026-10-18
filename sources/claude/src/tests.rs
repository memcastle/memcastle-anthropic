use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TREE: AtomicUsize = AtomicUsize::new(0);

struct Tree(PathBuf);

impl Tree {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "memcastle-claude-test-{}-{}",
            std::process::id(),
            NEXT_TREE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, body: &str) {
        let path = self.0.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn source(&self, options: Vec<(String, String)>) -> SourceRef {
        Claude::identify(Some(self.0.display().to_string()), options).unwrap()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn candidate(name: &str) -> Candidate {
    Candidate {
        handle: name.into(),
        external_id: name.into(),
        cursor_after: "null".into(),
    }
}

#[test]
fn a_timestamp_cutoff_honors_the_clock_and_offset() {
    let cutoff = parse_since("2026-09-14T18:00:00.250+02:00").unwrap();
    assert_eq!(cutoff, parse_since("2026-09-14T16:00:00.250Z").unwrap());
    assert_eq!(cutoff, parse_since("2026-09-14t16:00:00.250z").unwrap());
    assert_eq!(cutoff, parse_since("2026-09-14T11:00:00.25-05:00").unwrap());
    assert!(cutoff > parse_since("2026-09-14").unwrap());
    assert!(cutoff < parse_since("2026-09-15").unwrap());
    assert_eq!(
        parse_since("2026-09").unwrap(),
        parse_since("2026-09-01").unwrap()
    );
    assert!(parse_since("2024-02-29").is_ok());
    for invalid in [
        "",
        "2026",
        "2026-13",
        "2026-00",
        "2026-02-29",
        "2024-02-30",
        "2026-9-14",
        "2026-09-1",
        "2026-09-14T25:00:00Z",
        "2026-09-14T18:60:00Z",
        "2026-09-14T18:00:60Z",
        "2026-09-14T18:00:00",
        "2026-09-14T18:00:00+02:99",
        "2026-09-14T18:00:00+24:00",
        "2026-09-14T18:00:00+0200",
        "2026-09-14T18:00:00.Z",
        "2026-09-14T18:00:00.xZ",
        "2026-09-14T18:00:00:01Z",
    ] {
        assert!(parse_since(invalid).is_err(), "{invalid} should be refused");
    }
}

#[test]
fn identification_scopes_and_validates_options() {
    let tree = Tree::new();
    let source = tree.source(vec![
        ("since".into(), " 2026-09-14 ".into()),
        ("dir".into(), " /work/./app/../notes ".into()),
    ]);
    assert_eq!(source.source, "claude");
    assert_eq!(source.locator, tree.0.display().to_string());
    assert_eq!(source.account.as_deref(), Some("dir=/work/notes"));
    assert_eq!(source.options[0].1, "2026-09-14");
    assert_eq!(source.options[1].1, "/work/notes");
    assert_eq!(Claude::default_wing(source), "claude");
    assert_eq!(Claude::default_room(), "sessions");

    for options in [
        vec![("since".into(), "yesterday".into())],
        vec![("dir".into(), "relative/path".into())],
        vec![("dir".into(), " . ".into())],
        vec![("unknown".into(), "value".into())],
    ] {
        assert!(matches!(
            Claude::identify(Some(tree.0.display().to_string()), options),
            Err(SourceError::InvalidInput(_))
        ));
    }
    assert!(matches!(
        Claude::identify(Some(tree.0.join("missing").display().to_string()), vec![]),
        Err(SourceError::InvalidInput(_))
    ));
    let wildcard = tree.source(vec![("dir".into(), "*/notes".into())]);
    assert_eq!(wildcard.account.as_deref(), Some("dir=*/notes"));
}

#[test]
fn discovery_is_shallow_ordered_and_resumes_after_the_cursor() {
    let tree = Tree::new();
    tree.write("settings.jsonl", "{}");
    tree.write("project/a.jsonl", "{}");
    tree.write("project/b.jsonl", "{}");
    tree.write("project/ignored.txt", "{}");
    tree.write("project/nested/deep.jsonl", "{}");
    let source = tree.source(vec![]);
    let first = Claude::discover(source.clone(), "null".into(), 1).unwrap();
    assert_eq!(first.candidates.len(), 1);
    assert_eq!(first.candidates[0].handle, "project/a.jsonl");
    assert!(!first.exhausted);
    let second =
        Claude::discover(source.clone(), first.candidates[0].cursor_after.clone(), 1).unwrap();
    assert_eq!(second.candidates[0].external_id, "project/b.jsonl");
    assert!(second.exhausted);
    assert!(
        Claude::discover(source.clone(), second.candidates[0].cursor_after.clone(), 1)
            .unwrap()
            .candidates
            .is_empty()
    );
    let zero = Claude::discover(source, "null".into(), 0).unwrap();
    assert!(zero.candidates.is_empty());
    assert!(!zero.exhausted);
}

#[test]
fn discovery_filters_by_instant_and_working_directory() {
    let tree = Tree::new();
    tree.write("one/a.jsonl", "{\"cwd\":\"/work/app\"}\n");
    tree.write(
        "two/b.jsonl",
        "not json\n{\"cwd\":\"/work/notes/../notes\"}\n",
    );
    tree.write(
        "three/c.jsonl",
        "{\"message\":{\"role\":\"user\",\"content\":\"no cwd\"}}\n",
    );
    let notes = tree.source(vec![("dir".into(), "*/notes".into())]);
    let filtered = Claude::discover(notes, "null".into(), 10).unwrap();
    assert_eq!(filtered.candidates.len(), 1);
    assert_eq!(filtered.candidates[0].handle, "two/b.jsonl");

    let future = tree.source(vec![("since".into(), "9999-01-01".into())]);
    assert!(
        Claude::discover(future, "null".into(), 10)
            .unwrap()
            .candidates
            .is_empty()
    );
    let past = tree.source(vec![("since".into(), "1970-01-01".into())]);
    assert_eq!(
        Claude::discover(past, "null".into(), 10)
            .unwrap()
            .candidates
            .len(),
        3
    );
    let mut invalid = tree.source(vec![]);
    invalid.options.push(("since".into(), "bad".into()));
    assert!(matches!(
        Claude::discover(invalid, "null".into(), 10),
        Err(SourceError::InvalidInput(_))
    ));
    let mut missing = tree.source(vec![]);
    missing.locator = tree.0.join("missing").display().to_string();
    assert!(
        Claude::discover(missing, "null".into(), 10)
            .unwrap()
            .candidates
            .is_empty()
    );
}

#[test]
fn bad_cursors_are_rejected_before_discovery() {
    let tree = Tree::new();
    for cursor in [
        "not json",
        "{}",
        r#"{"mtime_ns":"1","key":"a"}"#,
        r#"{"mtime_ns":1}"#,
        r#"{"mtime_ns":1,"key":5}"#,
    ] {
        assert!(matches!(
            Claude::discover(tree.source(vec![]), cursor.into(), 1),
            Err(SourceError::CursorInvalid(_))
        ));
    }
    assert!(
        parse_cursor(r#"{"mtime_ns":1,"key":"project/a.jsonl"}"#)
            .unwrap()
            .is_some()
    );
}

#[test]
fn reading_and_normalizing_preserve_public_messages_only() {
    let tree = Tree::new();
    let body = concat!(
        "broken json\n",
        "{\"sessionId\":\"session-42\",\"cwd\":\"/work/app\",\"timestamp\":\"2026-09-14T16:00:00Z\",\"type\":\"metadata\"}\n",
        "{\"isMeta\":true,\"message\":{\"role\":\"user\",\"content\":\"private metadata\"}}\n",
        "{\"isCompactSummary\":true,\"message\":{\"role\":\"assistant\",\"content\":\"private summary\"}}\n",
        "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"text\",\"text\":\"  Rotate keys  \"},{\"type\":\"thinking\",\"text\":\"private thought\"}]},\"timestamp\":\"2026-09-14T16:00:01Z\"}\n",
        "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"name\":\"Read\"},{\"type\":\"text\",\"text\":\"Done\"}]}}\n",
        "{\"message\":{\"role\":\"tool\",\"content\":\"private tool result\"}}\n"
    );
    tree.write("project/session.jsonl", body);
    let source = tree.source(vec![]);
    let raw = Claude::read(source.clone(), candidate("project/session.jsonl"))
        .unwrap()
        .unwrap();
    assert_eq!(raw.body, body);
    assert_eq!(raw.external_id, "project/session.jsonl");
    assert_eq!(raw.occurred_at.as_deref(), Some("2026-09-14T16:00:00Z"));
    assert_eq!(
        serde_json::from_str::<Value>(&raw.metadata).unwrap()["session_id"],
        "session-42"
    );
    assert_eq!(
        serde_json::from_str::<Value>(&raw.metadata).unwrap()["cwd"],
        "/work/app"
    );
    assert_eq!(
        raw.revision,
        Claude::read(source, candidate("project/session.jsonl"))
            .unwrap()
            .unwrap()
            .revision
    );
    let document = Claude::normalize(raw).unwrap();
    assert_eq!(document.title.as_deref(), Some("Rotate keys"));
    assert_eq!(document.room.as_deref(), Some("app"));
    assert_eq!(document.name.as_deref(), Some("session"));
    assert!(matches!(document.kind, SourceKind::Transcript));
    assert_eq!(
        document.uri.as_deref(),
        Some(tree.0.join("project/session.jsonl").to_str().unwrap())
    );
    assert_eq!(document.tags, ["transcript", "claude"]);
    assert_eq!(document.segments.len(), 3);
    assert_eq!(
        document.segments[0].text,
        "# Claude Code session session-42\nworking directory: /work/app\n\n"
    );
    assert_eq!(
        document.segments[1].text,
        "### user · 2026-09-14T16:00:01Z\nRotate keys\n\n"
    );
    assert_eq!(
        document.segments[2].text,
        "### assistant · unknown\n[tool: Read]\nDone\n\n"
    );
    let combined = document
        .segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<String>();
    for private in [
        "private metadata",
        "private summary",
        "private thought",
        "private tool result",
    ] {
        assert!(!combined.contains(private));
    }
}

#[test]
fn read_skips_missing_empty_and_unusable_transcripts() {
    let tree = Tree::new();
    tree.write("project/empty.jsonl", " \n ");
    tree.write("project/meta.jsonl", "not json\n{\"type\":\"progress\"}\n{\"message\":{\"role\":\"assistant\",\"content\":\"  \"}}\n");
    tree.write(
        "project/top-level.jsonl",
        "{\"type\":\"user\",\"content\":\"top level works\"}\n",
    );
    let source = tree.source(vec![]);
    for name in ["missing", "empty", "meta"] {
        assert!(
            Claude::read(source.clone(), candidate(&format!("project/{name}.jsonl")))
                .unwrap()
                .is_none()
        );
    }
    let raw = Claude::read(source, candidate("project/top-level.jsonl"))
        .unwrap()
        .unwrap();
    let document = Claude::normalize(raw).unwrap();
    assert_eq!(document.title.as_deref(), Some("top level works"));
    assert_eq!(document.segments.len(), 2);
    assert_eq!(
        document.segments[0].text,
        "# Claude Code session top-level\nworking directory: unknown\n\n"
    );
}

#[test]
fn normalize_falls_back_when_metadata_or_record_fields_are_missing() {
    let long_title = "é".repeat(TITLE_CHARS + 5);
    let raw = RawDocument {
        external_id: "group/fallback.jsonl".into(),
        revision: "revision".into(),
        body: format!(
            "invalid\n{{\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"tool_use\"}}]}}}}\n{{\"message\":{{\"role\":\"user\",\"content\":\"{long_title}\\nnext line\"}}}}\n"
        ),
        metadata: "not json".into(),
        occurred_at: None,
    };
    let doc = Claude::normalize(raw).unwrap();
    assert_eq!(doc.title.as_deref(), Some("é".repeat(TITLE_CHARS).as_str()));
    assert_eq!(doc.room, None);
    assert_eq!(doc.uri, None);
    assert_eq!(doc.name.as_deref(), Some("fallback"));
    assert_eq!(
        doc.segments[0].text,
        "# Claude Code session fallback\nworking directory: unknown\n\n"
    );
    assert_eq!(
        doc.segments[1].text,
        "### assistant · unknown\n[tool: tool]\n\n"
    );
    assert!(doc.segments[2].text.contains("next line"));

    let empty = Claude::normalize(RawDocument {
        external_id: "empty.jsonl".into(),
        revision: String::new(),
        body: "{\"type\":\"progress\"}\n".into(),
        metadata: "{}".into(),
        occurred_at: None,
    })
    .unwrap();
    assert!(empty.segments.is_empty());
    assert!(empty.title.is_none());
}

#[test]
fn path_filters_match_only_expected_directories() {
    assert_eq!(normalize_dir("/work/./app/../notes/"), "/work/notes");
    assert_eq!(normalize_dir("../notes"), "notes");
    assert!(glob_match("*/notes", "/work/notes"));
    assert!(glob_match("/work/*/notes", "/work/project/notes"));
    assert!(glob_match("/work/**", "/work/project/notes"));
    assert!(!glob_match("/work/app", "/work/notes"));
    assert!(!glob_match("/work/*/notes", "/work/notes"));
}
