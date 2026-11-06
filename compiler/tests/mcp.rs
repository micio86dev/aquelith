//! MCP server (M8): protocol envelope, error codes, and each tool. Calls go through `handle_line`,
//! the same entry point the `aqelith-mcp` binary uses.

use aqelith::mcp::handle_line;

const GOOD: &str = "module demo\n\nfn add1(x: i32) -> i32 {\n    x + 1\n}\n";

fn call(id: &str, name: &str, args: &str) -> String {
    let line = format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{name}","arguments":{args}}}}}"#);
    handle_line(&line).expect("requests get a reply")
}

fn str_lit(s: &str) -> String {
    // Escapes a Rust string as a JSON string literal for the request arguments.
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
}

#[test]
fn initialize_echoes_the_id_and_names_the_protocol() {
    let reply = handle_line(r#"{"jsonrpc":"2.0","id":"abc","method":"initialize","params":{}}"#).unwrap();
    assert!(reply.starts_with(r#"{"jsonrpc":"2.0","id":"abc","result":{"protocolVersion":"2024-11-05""#), "{reply}");
    assert!(reply.contains(r#""tools":{}"#), "{reply}");
}

#[test]
fn notifications_get_no_reply() {
    assert_eq!(handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#), None);
}

#[test]
fn tools_list_names_all_five_tools() {
    let reply = handle_line(r#"{"jsonrpc":"2.0","id":7,"method":"tools/list"}"#).unwrap();
    for name in ["aqelith_check", "aqelith_format", "aqelith_query", "aqelith_explain", "aqelith_patch"] {
        assert!(reply.contains(&format!(r#""name":"{name}""#)), "missing {name}: {reply}");
    }
}

#[test]
fn protocol_errors_use_json_rpc_codes() {
    assert!(handle_line("not json").unwrap().contains(r#""code":-32700"#));
    assert!(handle_line("[1,2]").unwrap().contains(r#""code":-32600"#));
    let unknown = handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"resources/list"}"#).unwrap();
    assert!(unknown.contains(r#""code":-32601"#), "{unknown}");
    let no_tool = handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nope"}}"#).unwrap();
    assert!(no_tool.contains(r#""code":-32602"#), "{no_tool}");
}

#[test]
fn a_missing_argument_is_an_invalid_params_error() {
    let reply = call("1", "aqelith_check", "{}");
    assert!(reply.contains(r#""code":-32602"#), "{reply}");
}

#[test]
fn check_reports_diagnostics_as_data_not_as_a_tool_error() {
    let src = "module m\nfn f() -> i32 {\n    true\n}\n";
    let reply = call("1", "aqelith_check", &format!(r#"{{"source":{}}}"#, str_lit(src)));
    assert!(reply.contains(r#""isError":false"#), "{reply}");
    assert!(reply.contains("TYP001"), "{reply}");
    assert!(reply.contains(r#"\"ok\":false"#), "{reply}");
}

#[test]
fn format_returns_canonical_text_and_fails_on_unparsable_source() {
    let ok = call("1", "aqelith_format", &format!(r#"{{"source":{}}}"#, str_lit(GOOD)));
    assert!(ok.contains(r#""isError":false"#) && ok.contains("fn add1(x: i32) -> i32 {"), "{ok}");
    let bad = call("2", "aqelith_format", &format!(r#"{{"source":{}}}"#, str_lit("module m\nfn {")));
    assert!(bad.contains(r#""isError":true"#), "{bad}");
}

#[test]
fn query_returns_the_function_record_and_reports_a_missing_one() {
    let found = call("1", "aqelith_query", &format!(r#"{{"source":{},"function":"add1"}}"#, str_lit(GOOD)));
    assert!(found.contains(r#""isError":false"#) && found.contains("add1"), "{found}");
    let missing = call("2", "aqelith_query", &format!(r#"{{"source":{},"function":"nope"}}"#, str_lit(GOOD)));
    assert!(missing.contains(r#""isError":true"#), "{missing}");
}

#[test]
fn explain_knows_codes_and_flags_unknown_ones() {
    let known = call("1", "aqelith_explain", r#"{"code":"PAT002"}"#);
    assert!(known.contains(r#""isError":false"#) && known.contains("stale"), "{known}");
    let unknown = call("2", "aqelith_explain", r#"{"code":"NOPE001"}"#);
    assert!(unknown.contains(r#""isError":true"#), "{unknown}");
}

#[test]
fn a_patch_returns_a_candidate_and_never_touches_the_source_argument() {
    let base = {
        let reply = call("1", "aqelith_format", &format!(r#"{{"source":{}}}"#, str_lit(GOOD)));
        assert!(reply.contains("fn add1"), "{reply}");
        aqelith::patch::graph_hash(&aqelith::format_source(GOOD).unwrap())
    };
    let doc = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{base}","ops":[{{"op":"rename","target":"add1","expect":{{"name":"add1"}},"to":"inc"}}]}}"#
    );
    let args = format!(r#"{{"source":{},"patch":{}}}"#, str_lit(GOOD), str_lit(&doc));
    let reply = call("2", "aqelith_patch", &args);
    assert!(reply.contains(r#""isError":false"#), "{reply}");
    assert!(reply.contains(r#"\"status\":\"candidate\""#), "{reply}");
    assert!(reply.contains("fn inc(x: i32)"), "{reply}");
}

#[test]
fn a_rejected_patch_is_a_tool_error_with_its_code() {
    let doc = r#"{"api":"aqelith.patch.v1","base":"graph:0000000000000000","ops":[]}"#;
    let args = format!(r#"{{"source":{},"patch":{}}}"#, str_lit(GOOD), str_lit(doc));
    let reply = call("3", "aqelith_patch", &args);
    assert!(reply.contains(r#""isError":true"#), "{reply}");
    assert!(reply.contains("PAT002"), "{reply}");
}
