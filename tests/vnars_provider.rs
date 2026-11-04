use narsese::lexical::{Sentence, Task, Term};
use narust_158::{
    inference::{match_task_and_belief, process_direct, reason, transform_task, InferenceEngine},
    parameters::DEFAULT_PARAMETERS,
    vm::alpha::LauncherAlpha,
    vnars_provider::{output_to_provider_value, serve, task_from_json, task_to_json},
};
use navm::{output::Output, vm::VmLauncher};
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Cursor, Write},
    process::{Command, Stdio},
};

fn sample_task() -> Task {
    Task {
        budget: vec![],
        sentence: Sentence {
            term: Term::new_statement_infix(
                Term::new_atom("", "A"),
                "-->",
                Term::new_atom("", "B"),
            ),
            punctuation: ".".into(),
            stamp: "".into(),
            truth: vec!["1.0".into(), "0.9".into()],
        },
    }
}

#[test]
fn lexical_task_round_trips_through_vnars_task_json() {
    let task = sample_task();
    let value = task_to_json(&task);

    assert_eq!(
        value,
        json!({
            "term": {
                "type": "statement",
                "copula": "-->",
                "subject": {"type": "atom", "prefix": "", "name": "A"},
                "predicate": {"type": "atom", "prefix": "", "name": "B"}
            },
            "punctuation": ".",
            "stamp": "",
            "truth": ["1.0", "0.9"],
            "budget": []
        })
    );
    assert_eq!(task_from_json(&value).expect("valid task"), task);
}

#[test]
fn omitted_atom_prefix_defaults_to_empty() {
    let task = task_from_json(&json!({
        "term": {"type": "atom", "name": "A"},
        "punctuation": "."
    }))
    .expect("vNARS omits atom prefixes when they are empty");

    assert_eq!(task.sentence.term, Term::new_atom("", "A"));
}

#[test]
fn invalid_task_json_is_rejected_with_context() {
    let error = task_from_json(&json!({
        "term": {"type": "atom", "prefix": ""},
        "punctuation": "."
    }))
    .expect_err("missing atom name must be rejected");

    assert!(error.to_string().contains("term.atom.name"));
}

#[test]
fn provider_projection_adds_task_for_core_narsese_outputs_only() {
    let task = sample_task();
    let input = Output::IN {
        content: "input".into(),
        narsese: Some(narsese::lexical::Narsese::Task(task.clone())),
    };
    let value = output_to_provider_value(&input);

    assert_eq!(value["type"], "IN");
    assert_eq!(value["content"], "input");
    assert_eq!(value["task"], task_to_json(&task));

    let info = Output::INFO {
        message: "status".into(),
    };
    let value = output_to_provider_value(&info);
    assert_eq!(value["type"], "INFO");
    assert!(value.get("task").is_none());
}

#[test]
fn provider_session_reports_ready_outputs_boundary_and_close() {
    let runtime = LauncherAlpha::new(
        "provider-test",
        DEFAULT_PARAMETERS,
        InferenceEngine::new(
            process_direct,
            transform_task,
            match_task_and_belief,
            reason,
        ),
    )
    .launch()
    .expect("runtime launches");
    let mut runtime = runtime;
    let task = sample_task();
    let frames = vec![
        json!({"type":"command","id":"1","command":{"type":"input","narsese":task_to_json(&task)}}),
        json!({"type":"command","id":"2","command":{"type":"advance","cycles":1}}),
        json!({"type":"close","id":"3"}),
    ];
    let input = frames
        .into_iter()
        .map(|frame| frame.to_string())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let mut output = Vec::new();

    serve(&mut runtime, Cursor::new(input), &mut output).expect("provider session succeeds");

    let values = String::from_utf8(output)
        .expect("provider output is UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(values[0]["kind"], "ready");
    assert_eq!(values[0]["readiness"], "confirmed");
    assert!(values.iter().any(|value| {
        value["kind"] == "output" && value["id"] == "1" && value["output"]["type"] == "IN"
    }));
    assert!(values.iter().any(|value| {
        value["kind"] == "command_end" && value["id"] == "1" && value["boundary"] == "sealed"
    }));
    assert!(values.iter().any(|value| {
        value["kind"] == "command_end" && value["id"] == "2" && value["boundary"] == "sealed"
    }));
    assert!(values
        .iter()
        .any(|value| value["kind"] == "close_received" && value["id"] == "3"));
    assert!(values
        .iter()
        .any(|value| value["kind"] == "closed" && value["id"] == "3"));
}

#[test]
fn unsupported_provider_command_returns_an_id_bound_error() {
    let runtime = LauncherAlpha::new(
        "provider-test",
        DEFAULT_PARAMETERS,
        InferenceEngine::new(
            process_direct,
            transform_task,
            match_task_and_belief,
            reason,
        ),
    )
    .launch()
    .expect("runtime launches");
    let mut runtime = runtime;
    let input = json!({
        "type": "command",
        "id": "7",
        "command": {"type": "register_operation", "name": "^demo"}
    })
    .to_string()
        + "\n";
    let mut output = Vec::new();

    let error = serve(&mut runtime, Cursor::new(input), &mut output)
        .expect_err("unsupported command must terminate the polluted session");

    assert!(error.to_string().contains("register_operation"));
    let values = String::from_utf8(output)
        .expect("provider output is UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(values.iter().any(|value| {
        value["kind"] == "error"
            && value["id"] == "7"
            && value["message"]
                .as_str()
                .is_some_and(|message| message.contains("register_operation"))
    }));
    assert!(!values.iter().any(|value| value["kind"] == "command_end"));
}

#[test]
fn provider_binary_reports_ready_boundary_and_close_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_narust_158_vnars_provider"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("provider starts");
    let mut stdin = child.stdin.take().expect("provider stdin");
    let stdout = child.stdout.take().expect("provider stdout");
    let mut stdout = BufReader::new(stdout);

    let ready = read_json_line(&mut stdout);
    assert_eq!(ready["kind"], "ready");
    assert_eq!(ready["readiness"], "confirmed");

    write_json_line(
        &mut stdin,
        &json!({
            "type": "command",
            "id": "1",
            "command": {"type": "input", "narsese": task_to_json(&sample_task())}
        }),
    );
    let frames = read_until_command_end(&mut stdout, "1");
    assert!(frames.iter().any(|frame| {
        frame["kind"] == "output"
            && frame["output"]["type"] == "IN"
            && frame["output"]["task"]["term"]["subject"]["name"] == "A"
    }));
    assert_eq!(frames.last().expect("command end")["boundary"], "sealed");

    write_json_line(&mut stdin, &json!({"type":"close","id":"2"}));
    let close_received = read_json_line(&mut stdout);
    let closed = read_json_line(&mut stdout);
    assert_eq!(close_received["kind"], "close_received");
    assert_eq!(closed["kind"], "closed");
    assert_eq!(closed["report"]["receipt"], "processed");
    assert!(child.wait().expect("provider exits").success());
}

fn write_json_line(stdin: &mut impl Write, value: &serde_json::Value) {
    writeln!(stdin, "{value}").expect("provider frame writes");
    stdin.flush().expect("provider frame flushes");
}

fn read_json_line(reader: &mut impl BufRead) -> serde_json::Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("provider frame reads");
    serde_json::from_str(&line).expect("provider frame is JSON")
}

fn read_until_command_end(reader: &mut impl BufRead, id: &str) -> Vec<serde_json::Value> {
    let mut frames = Vec::new();
    loop {
        let frame = read_json_line(reader);
        assert_eq!(frame["id"], id);
        let end = frame["kind"] == "command_end";
        frames.push(frame);
        if end {
            return frames;
        }
    }
}
