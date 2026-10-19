use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture(name: &str) -> PathBuf {
    let timestamp =
        SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock must be after unix epoch").as_nanos();
    let path = std::env::temp_dir().join(format!("owly-{name}-{}-{timestamp}", std::process::id()));
    fs::create_dir_all(&path).expect("fixture directory should be created");
    path
}

fn write_config(root: &Path) {
    fs::write(
        root.join("owly.yml"),
        r#"owly_version: 0.1.0
wiki_root: .
mandatory_fields:
  type:
    type: string
types:
  note:
    fields:
      title:
        type: string
      status:
        type: string
        optional: true
        values: [draft, active]
"#,
    )
    .expect("config should be written");
}

fn run_validate(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_owly"))
        .args([
            "validate",
            "--config",
            root.join("owly.yml").to_str().expect("config path should be utf-8"),
            "--details",
        ])
        .output()
        .expect("owly should run")
}

#[test]
fn valid_wiki_returns_success() {
    let root = fixture("valid");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[Welcome](note.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("note.md"), "---\nid: welcome\ntype: note\ntitle: Welcome\nstatus: active\n---\n# Welcome\n")
        .expect("document should be written");

    let output = run_validate(&root);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Validation summary"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn validate_requires_entrance_page() {
    let root = fixture("missing-entrance");
    write_config(&root);
    fs::write(root.join("note.md"), "---\ntype: note\ntitle: Note\n---\n").expect("document should be written");

    let output = run_validate(&root);
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("missing-entrance"));
    assert!(text.contains("wiki must contain entrance.md"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn validate_prints_summary_by_default_and_details_on_request() {
    let root = fixture("validate-output");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\n---\n").expect("entrance should be written");

    let config = root.join("owly.yml");
    let summary = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", config.to_str().unwrap()])
        .output()
        .expect("validate should run");
    assert_eq!(summary.status.code(), Some(1));
    let summary_text = String::from_utf8(summary.stdout).expect("summary should be utf-8");
    assert!(summary_text.contains("Validation summary"));
    assert!(summary_text.contains("missing-required-field: 1"));
    assert!(!summary_text.contains("message:"));

    let details = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", config.to_str().unwrap(), "--details"])
        .output()
        .expect("validate details should run");
    assert_eq!(details.status.code(), Some(1));
    let details_text = String::from_utf8(details.stdout).expect("details should be utf-8");
    assert!(details_text.contains("Validation summary"));
    assert!(details_text.contains("message: required field 'title' is missing"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn source_tracking_checks_existence_and_hash_drift() {
    let root = fixture("source-tracking");
    fs::write(
        root.join("owly.yml"),
        "wiki_root: .\nsource_tracking:\n  enabled: true\nwiki_lint:\n  exclude: [raw/**]\nmandatory_fields:\n  type:\n    type: string\ntypes:\n  note:\n    fields: {}\n",
    )
    .expect("config should be written");
    fs::create_dir_all(root.join("raw")).expect("raw directory should be created");
    fs::write(root.join("raw/source.txt"), "abc").expect("source should be written");
    fs::write(
        root.join("entrance.md"),
        "---\ntype: note\nsources:\n  - path: raw/source.txt\n    sha256: ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n---\n",
    )
    .expect("page should be written");

    let valid = run_validate(&root);
    assert!(valid.status.success());
    fs::write(root.join("raw/source.txt"), "changed").expect("source should be changed");
    let drift = run_validate(&root);
    assert_eq!(drift.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&drift.stdout).contains("source-drift"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn overview_and_search_return_structured_results() {
    let root = fixture("overview-search");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[Welcome](note.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("note.md"), "---\ntype: note\ntitle: Welcome\nstatus: active\n---\n")
        .expect("document should be written");

    let config = root.join("owly.yml");
    let index = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["index", "--config", config.to_str().unwrap()])
        .output()
        .expect("index should run");
    assert!(index.status.success());
    let overview = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["overview", "types", "--config", config.to_str().unwrap(), "--type", "note"])
        .output()
        .expect("overview should run");
    assert!(overview.status.success());
    let overview_json: serde_json::Value = serde_json::from_slice(&overview.stdout).expect("valid JSON");
    assert_eq!(overview_json[0]["name"], "note");
    assert_eq!(overview_json[0]["schema"]["required_fields"]["type"]["type"], "string");

    let search = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["search", "--config", config.to_str().unwrap(), "--query", "status:active"])
        .output()
        .expect("search should run");
    assert!(search.status.success());
    let search_json: serde_json::Value = serde_json::from_slice(&search.stdout).expect("valid JSON");
    assert_eq!(search_json[0]["page_id"], "note");
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn inspect_returns_consolidated_wiki_state() {
    let root = fixture("inspect");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[Welcome](note.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("note.md"), "---\ntype: note\ntitle: Welcome\nstatus: active\n---\n")
        .expect("note should be written");

    let config = root.join("owly.yml");
    let index = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["index", "--config", config.to_str().unwrap()])
        .output()
        .expect("index should run");
    assert!(index.status.success());

    let inspect = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["inspect", "--config", config.to_str().unwrap(), "--format", "json"])
        .output()
        .expect("inspect should run");
    assert!(inspect.status.success());
    let state: serde_json::Value = serde_json::from_slice(&inspect.stdout).expect("valid JSON");
    assert_eq!(state["page_count"], 2);
    assert_eq!(state["types"]["note"], 2);
    assert_eq!(state["graph"]["entrance"], "entrance");
    assert_eq!(state["graph"]["broken_references"], 0);
    assert_eq!(state["graph"]["orphan_pages"].as_array().unwrap().len(), 0);
    assert_eq!(state["index"]["exists"], true);
    assert_eq!(state["index"]["document_count"], 2);
    assert_eq!(state["validation"]["errors"], 0);
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn search_builds_and_refreshes_the_index_automatically() {
    let root = fixture("search-auto-index");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n").expect("entrance should be written");
    fs::write(root.join("note.md"), "---\ntype: note\ntitle: Welcome\nstatus: draft\n---\n")
        .expect("note should be written");

    let config = root.join("owly.yml");
    let first = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["search", "--config", config.to_str().unwrap(), "--query", "status:draft"])
        .output()
        .expect("search should run");
    assert!(first.status.success());
    assert!(root.join(".owly/index.sqlite").is_file());

    fs::write(root.join("note.md"), "---\ntype: note\ntitle: Welcome\nstatus: active\n---\n")
        .expect("note should be updated");
    let second = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["search", "--config", config.to_str().unwrap(), "--query", "status:active"])
        .output()
        .expect("search should refresh the index");
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("note"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn config_validate_validates_without_scanning_the_wiki() {
    let root = fixture("config-lint");
    write_config(&root);

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["config", "validate", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("config validate should run");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"OK: no issues found\n");
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn enabled_source_tracking_warns_and_ignores_custom_source_rules() {
    let root = fixture("source-definition-warning");
    fs::write(
        root.join("owly.yml"),
        "owly_version: 0.1.0\nwiki_root: .\nsource_tracking:\n  enabled: true\nmandatory_fields:\n  sources:\n    type: array\n    items:\n      type: string\ntypes: {}\n",
    )
    .expect("config should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["config", "validate", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("config validate should run");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output should be UTF-8");
    assert!(text.contains("source-definition-ignored"));
    assert!(text.contains("warning"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn config_validate_is_the_configuration_validation_command() {
    let root = fixture("config-validate");
    write_config(&root);

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["config", "validate", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("config validate should run");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"OK: no issues found\n");
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn graph_exports_nodes_edges_and_maintenance_signals() {
    let root = fixture("graph");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[Welcome](note.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("note.md"), "---\ntype: note\ntitle: Welcome\n---\n[Missing](missing.md)\n")
        .expect("note should be written");
    fs::write(root.join("orphan.md"), "---\ntype: note\ntitle: Orphan\n---\n").expect("orphan should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["graph", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("graph should run");
    assert!(output.status.success());
    let graph: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(graph["edges"][0]["target"], "note");
    assert_eq!(graph["broken_references"][0]["target"], "missing");
    assert_eq!(graph["orphan_pages"][0], "orphan");
    assert_eq!(graph["unreachable_pages"][0], "orphan");
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn external_graph_links_only_allow_entrance() {
    let root = fixture("external-graph-links");
    write_config(&root);
    fs::write(
        root.join("entrance.md"),
        "---\ntype: note\ntitle: Entrance\n---\n[Other](../other/entrance.md)\n[Invalid](../other/page.md)\n",
    )
    .expect("entrance should be written");

    let output = run_validate(&root);
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("external-graph-entry"));
    assert!(text.contains("external graph link must target entrance.md"));
    assert!(text.contains("missing-external-graph"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn schema_exports_configuration_for_agents() {
    let root = fixture("schema");
    write_config(&root);
    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["schema", "--config", root.join("owly.yml").to_str().unwrap(), "--format", "json"])
        .output()
        .expect("schema should run");
    assert!(output.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(schema["types"]["note"]["fields"]["title"]["type"], "string");
    assert!(schema["diagnostics"].as_array().unwrap().is_empty());
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn validate_reports_maintenance_diagnostics_without_modifying_files() {
    let root = fixture("maintenance");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[Missing](missing.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("orphan.md"), "---\ntype: note\ntitle: Orphan\n---\n").expect("orphan should be written");
    let before = fs::read(root.join("orphan.md")).expect("orphan should be readable");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", root.join("owly.yml").to_str().unwrap(), "--details"])
        .output()
        .expect("validate should run");
    assert_eq!(output.status.code(), Some(1));
    let output_text = String::from_utf8_lossy(&output.stdout);
    assert!(output_text.contains("orphan-page"));
    assert!(output_text.contains("broken-link"));
    assert_eq!(fs::read(root.join("orphan.md")).expect("orphan should be readable"), before);
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn validate_reports_orphans_and_can_filter_to_one_file() {
    let root = fixture("orphans");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[Welcome](note.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("note.md"), "---\ntype: note\ntitle: Welcome\n---\n").expect("document should be written");
    fs::write(root.join("orphan.md"), "---\ntype: note\ntitle: Orphan\n---\n").expect("document should be written");

    let config = root.join("owly.yml");
    let all = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", config.to_str().unwrap(), "--details"])
        .output()
        .expect("check should run");
    let all_output = String::from_utf8(all.stdout).expect("human output should be utf-8");
    assert!(all_output.contains("orphan-page"));

    let one = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", config.to_str().unwrap(), "--file", "orphan.md", "--details"])
        .output()
        .expect("file check should run");
    let one_output = String::from_utf8(one.stdout).expect("human output should be utf-8");
    assert!(one_output.contains("orphan-page"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn missing_field_is_reported() {
    let root = fixture("missing-field");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[Missing](note.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("note.md"), "---\nid: missing-title\ntype: note\n---\n").expect("document should be written");

    let output = run_validate(&root);
    assert_eq!(output.status.code(), Some(1));
    let diagnostics = String::from_utf8(output.stdout).expect("human output should be utf-8");
    assert!(diagnostics.contains("missing-required-field"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn duplicate_identifier_and_invalid_frontmatter_are_reported() {
    let root = fixture("invalid");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\n---\n[One](one.md) [Two](two.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("one.md"), "---\nid: same\ntype: note\ntitle: One\n---\n")
        .expect("document should be written");
    fs::write(root.join("two.md"), "---\nid: same\ntype: note\ntitle: Two\n---\n")
        .expect("document should be written");
    fs::write(root.join("broken.md"), "---\nid: broken\ntype: [note\n---\n").expect("document should be written");

    let output = run_validate(&root);
    assert_eq!(output.status.code(), Some(1));
    let diagnostics = String::from_utf8(output.stdout).expect("human output should be utf-8");
    assert!(diagnostics.contains("invalid-frontmatter"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn skill_command_writes_skill_file() {
    let root = fixture("skill");
    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["skill", root.to_str().expect("fixture path should be utf-8")])
        .output()
        .expect("owly should run");

    assert!(output.status.success());
    assert!(root.join("owly/using-wiki/SKILL.md").is_file());
    assert!(root.join("owly/wiki-config/SKILL.md").is_file());
    assert!(root.join("owly/wiki-maintenance/SKILL.md").is_file());
    assert!(root.join("owly/wiki-query/SKILL.md").is_file());
    assert!(root.join("owly/wiki-overview/SKILL.md").is_file());
    assert!(root.join("owly/wiki-search/SKILL.md").is_file());
    assert!(!root.join("owly/SKILL.md").exists());
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn onboard_command_prints_agent_workflow_without_writing_files() {
    let root = fixture("onboard");
    let output =
        Command::new(env!("CARGO_BIN_EXE_owly")).arg("onboard").current_dir(&root).output().expect("owly should run");

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("onboard output should be UTF-8");
    assert!(text.contains("owly skill <agent-skills-directory>"));
    assert!(text.contains("wiki-config Skill"));
    assert!(!root.join("owly.yml").exists());
    assert_eq!(fs::read_dir(&root).expect("fixture should be readable").count(), 0);
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn init_command_writes_default_config() {
    let root = fixture("init");
    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["init", "--template", "personal"])
        .current_dir(&root)
        .output()
        .expect("owly should run");

    assert!(output.status.success());
    let config = fs::read_to_string(root.join("owly.yml")).expect("config should be written");
    assert!(config.contains("mandatory_fields:"));
    assert!(config.contains("owly_version: 0.1.1"));
    assert!(config.contains("wiki_root: ."));
    assert!(config.contains("directories:"));
    assert!(config.contains("description:"));
    assert!(config.contains("types:"));
    assert!(config.contains("wiki_lint:"));
    assert!(config.contains("config_lint:"));
    assert!(config.contains("max_nesting_depth: 1"));
    assert!(config.contains("Agents should usually set it to 14 days after creation"));
    assert!(config.contains("**/.*"));
    assert!(config.contains("**/README.md"));
    assert!(config.contains("raw/**"));
    assert!(root.join("entrance.md").is_file());
    assert!(config.contains("# Wiki root path used by Owly commands.\nwiki_root:"));
    assert!(config.contains("\n\n# Directory structure and descriptions.\ndirectories:"));
    assert!(root.join(".owly/index.sqlite").is_file());
    assert!(config.contains("\n\n# Fields required on every document.\nmandatory_fields:"));
    let mandatory_fields = config.split("mandatory_fields:").nth(1).expect("mandatory fields should be written");
    let field_order = ["type:", "title:", "description:", "tags:", "generated:", "stale_after:"];
    let mut previous = 0;
    for field in field_order {
        let position = mandatory_fields
            .lines()
            .scan(0, |offset, line| {
                let current = *offset;
                *offset += line.len() + 1;
                Some((current, line))
            })
            .find_map(|(position, line)| (line == format!("  {field}")).then_some(position))
            .expect("default field should be written");
        assert!(position >= previous, "field {field} is out of order");
        previous = position;
    }

    let second_output =
        Command::new(env!("CARGO_BIN_EXE_owly")).arg("init").current_dir(&root).output().expect("owly should run");
    assert_eq!(second_output.status.code(), Some(2));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn init_lists_templates_without_writing_files() {
    let root = fixture("init-list");
    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["init", "--list-templates"])
        .current_dir(&root)
        .output()
        .expect("owly should run");

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("template list should be utf-8");
    assert!(text.contains("blank"));
    assert!(text.contains("personal"));
    assert!(text.contains("research"));
    assert!(!root.join("owly.yml").exists());
    assert!(!root.join("entrance.md").exists());
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn configured_root_and_nested_directories_are_supported() {
    let root = fixture("configured-root");
    fs::create_dir_all(root.join("wiki/raw/inbox")).expect("wiki directory should be created");
    fs::write(
        root.join("owly.yml"),
        "wiki_root: wiki\ndirectories:\n  raw:\n    description: Raw source\n    directories:\n      inbox:\n        description: Incoming files\nmandatory_fields:\n  type:\n    type: string\ntypes:\n  note:\n    description: A note\n    fields: {}\n",
    )
    .expect("config should be written");
    fs::write(root.join("wiki/entrance.md"), "---\ntype: note\n---\n[Wrong](raw/inbox/wrong.md)\n")
        .expect("entrance should be written");
    fs::write(root.join("wiki/raw/inbox/wrong.md"), "---\nid: wrong\ntype: note\n---\n")
        .expect("document should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", root.join("owly.yml").to_str().expect("path should be utf-8")])
        .output()
        .expect("owly should run");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Validation summary"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn config_validate_reports_invalid_rule_shapes() {
    let root = fixture("invalid-config");
    fs::write(
        root.join("owly.yml"),
        "wiki_root: .\nmandatory_fields:\n  tags:\n    type: string\n    items:\n      type: string\ntypes:\n  note:\n    fields:\n      status:\n        type: boolean\n        values: [active]\n",
    )
    .expect("config should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["config", "validate", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("config validate should run");
    assert_eq!(output.status.code(), Some(1));
    let diagnostics = String::from_utf8(output.stdout).expect("human output should be utf-8");
    assert!(diagnostics.contains("config-items-require-array"));
    assert!(diagnostics.contains("config-values-require-string"));
    assert!(diagnostics.contains("help: use 'items' only with type: array"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn invalid_config_stops_validate_with_configuration_exit_code() {
    let root = fixture("invalid-config-check");
    fs::write(
        root.join("owly.yml"),
        "wiki_root: .\ntypes:\n  note:\n    fields:\n      status:\n        type: boolean\n        values: [active]\n",
    )
    .expect("config should be written");

    let output = run_validate(&root);
    assert_eq!(output.status.code(), Some(2));
    let diagnostics = String::from_utf8(output.stdout).expect("human output should be utf-8");
    assert!(diagnostics.contains("config-values-require-string"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn validate_reports_type_value_and_nested_shape_errors() {
    let root = fixture("value-errors");
    fs::write(
        root.join("owly.yml"),
        "wiki_root: .\nmandatory_fields:\n  type:\n    type: string\n  tags:\n    type: array\n    items:\n      type: string\ntypes:\n  note:\n    fields:\n      status:\n        type: string\n        values: [draft, active]\n      published:\n        type: date\n",
    )
    .expect("config should be written");
    fs::write(
        root.join("entrance.md"),
        "---\ntype: note\ntags: [one, 2]\nstatus: archived\npublished: 2026-2-1\n---\n",
    )
    .expect("entrance should be written");

    let output = run_validate(&root);
    assert_eq!(output.status.code(), Some(1));
    let diagnostics = String::from_utf8(output.stdout).expect("human output should be utf-8");
    assert!(diagnostics.contains("invalid-field-type"));
    assert!(diagnostics.contains("invalid-field-value"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn search_matches_scalars_and_array_values_and_rejects_bad_queries() {
    let root = fixture("search-values");
    write_config(&root);
    fs::write(root.join("entrance.md"), "---\ntype: note\ntitle: Index\nstatus: active\n---\n")
        .expect("entrance should be written");
    fs::write(root.join("draft.md"), "---\ntype: note\ntitle: Draft\nstatus: draft\n---\n")
        .expect("draft should be written");

    let config = root.join("owly.yml");
    let index = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["index", "--config", config.to_str().unwrap()])
        .output()
        .expect("index should run");
    assert!(index.status.success());
    let scalar = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["search", "--config", config.to_str().unwrap(), "--query", "status:active", "--format", "json"])
        .output()
        .expect("search should run");
    let results: Vec<serde_json::Value> = serde_json::from_slice(&scalar.stdout).expect("valid JSON");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["value"], "active");

    let bad = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["search", "--config", config.to_str().unwrap(), "--query", "status"])
        .output()
        .expect("search should run");
    assert_eq!(bad.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("field:value"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn overview_reports_directory_statistics_and_unknown_types_fail() {
    let root = fixture("overview-errors");
    fs::create_dir_all(root.join("docs")).expect("directory should be created");
    fs::write(
        root.join("owly.yml"),
        "wiki_root: .\ndirectories:\n  docs:\n    description: Documentation\nmandatory_fields:\n  type:\n    type: string\ntypes:\n  note:\n    fields: {}\n",
    )
    .expect("config should be written");
    fs::write(root.join("docs/guide.md"), "---\ntype: note\n---\n").expect("guide should be written");
    let config = root.join("owly.yml");

    let directories = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["overview", "directories", "--config", config.to_str().unwrap(), "--statistics"])
        .output()
        .expect("overview should run");
    let views: Vec<serde_json::Value> = serde_json::from_slice(&directories.stdout).expect("valid JSON");
    assert_eq!(views[0]["path"], "docs");
    assert_eq!(views[0]["statistics"]["total"], 1);

    let unknown = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["overview", "types", "--config", config.to_str().unwrap(), "--type", "missing"])
        .output()
        .expect("overview should run");
    assert_eq!(unknown.status.code(), Some(2));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn graph_resolves_relative_links_and_ignores_non_page_links() {
    let root = fixture("graph-links");
    write_config(&root);
    fs::create_dir_all(root.join("docs")).expect("directory should be created");
    fs::write(
        root.join("entrance.md"),
        "---\ntype: note\ntitle: Index\n---\n[Guide](docs/guide.md) [asset](image.png) [web](https://example.com)\n",
    )
    .expect("index should be written");
    fs::write(root.join("docs/guide.md"), "---\ntype: note\ntitle: Guide\n---\n[Home](../entrance.md#top)\n")
        .expect("guide should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["graph", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("graph should run");
    let graph: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(graph["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(graph["edges"].as_array().unwrap().len(), 2);
    assert!(graph["broken_references"].as_array().unwrap().is_empty());
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn generated_skills_are_english_and_contain_no_todos() {
    let root = fixture("skill-content");
    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["skill", root.to_str().unwrap()])
        .output()
        .expect("skill should run");
    assert!(output.status.success());
    for path in [
        "owly/using-wiki/SKILL.md",
        "owly/wiki-config/SKILL.md",
        "owly/wiki-ingest/SKILL.md",
        "owly/wiki-maintenance/SKILL.md",
        "owly/wiki-query/SKILL.md",
    ] {
        let content = fs::read_to_string(root.join(path)).expect("skill file should be readable");
        assert!(!content.contains("TODO"), "{path} should not contain TODO");
        assert!(!content.contains("です"), "{path} should be English");
        assert!(content.contains("Workflow"));
        assert!(content.starts_with("---\nname: "), "{path} should have Agent Skills frontmatter");
    }
    assert!(root.join("owly/wiki-ingest/references/source-file.md").is_file());
    assert!(root.join("owly/wiki-ingest/references/git-repository.md").is_file());
    assert!(root.join("owly/wiki-maintenance/references/internal-refactor.md").is_file());
    assert!(root.join("owly/wiki-maintenance/references/graph-migration.md").is_file());
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn missing_config_and_missing_file_return_execution_errors() {
    let root = fixture("missing-input");
    let missing_config = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", root.join("missing.yml").to_str().unwrap()])
        .output()
        .expect("owly should run");
    assert_eq!(missing_config.status.code(), Some(2));

    write_config(&root);
    let missing_file = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["validate", "--config", root.join("owly.yml").to_str().unwrap(), "--file", "missing.md"])
        .output()
        .expect("owly should run");
    assert_eq!(missing_file.status.code(), Some(1));
    let diagnostics = String::from_utf8(missing_file.stdout).expect("human output should be utf-8");
    assert!(diagnostics.contains("file-not-found"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn malformed_config_reports_yaml_location_and_help() {
    let root = fixture("malformed-config");
    fs::write(root.join("owly.yml"), "wiki_root: .\ntypes:\n  note\n    fields: {}\n")
        .expect("config should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["config", "validate", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("config validate should run");

    assert_eq!(output.status.code(), Some(2));
    let error = String::from_utf8(output.stderr).expect("error output should be utf-8");
    assert!(error.contains("invalid YAML in"));
    assert!(error.contains("location: line 3"));
    assert!(error.contains("help: check indentation"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}

#[test]
fn unknown_config_key_reports_a_configuration_error() {
    let root = fixture("unknown-config-key");
    fs::write(root.join("owly.yml"), "wiki_root: .\nmandatory_fileds: {}\n").expect("config should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_owly"))
        .args(["config", "validate", "--config", root.join("owly.yml").to_str().unwrap()])
        .output()
        .expect("config validate should run");

    assert_eq!(output.status.code(), Some(2));
    let error = String::from_utf8(output.stderr).expect("error output should be utf-8");
    assert!(error.contains("invalid configuration in"));
    assert!(error.contains("unknown field"), "unexpected error output: {error}");
    assert!(error.contains("mandatory_fileds"), "unexpected error output: {error}");
    assert!(error.contains("help: check the setting name"));
    fs::remove_dir_all(root).expect("fixture should be removed");
}
