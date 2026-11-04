//! `vnars.provider/1` compatibility boundary for NARust-158.

use anyhow::{bail, Context, Result};
use narsese::lexical::{Narsese, Sentence, Task, Term};
use navm::{
    cmd::Cmd,
    output::{Output, OutputJSON},
    vm::VmRuntime,
};
use serde_json::{Map, Value};
use std::{
    collections::BTreeSet,
    io::{self, BufRead, ErrorKind, Write},
};

pub const PROVIDER_PROTOCOL: &str = "vnars.provider/1";
pub const DEFAULT_MAX_FRAME_BYTES: usize = 1024 * 1024;
const TASK_FIELDS: [&str; 5] = ["term", "punctuation", "stamp", "truth", "budget"];

pub fn task_to_json(task: &Task) -> Value {
    let sentence = &task.sentence;
    serde_json::json!({
        "term": term_to_json(&sentence.term),
        "punctuation": sentence.punctuation,
        "stamp": sentence.stamp,
        "truth": sentence.truth,
        "budget": task.budget,
    })
}

fn term_to_json(term: &Term) -> Value {
    match term {
        Term::Atom { prefix, name } => {
            serde_json::json!({"type": "atom", "prefix": prefix, "name": name})
        }
        Term::Statement {
            copula,
            subject,
            predicate,
        } => serde_json::json!({
            "type": "statement",
            "copula": copula,
            "subject": term_to_json(subject),
            "predicate": term_to_json(predicate),
        }),
        Term::Compound { connecter, terms } => serde_json::json!({
            "type": "compound",
            "connecter": connecter,
            "terms": terms.iter().map(term_to_json).collect::<Vec<_>>(),
        }),
        Term::Set {
            left_bracket,
            terms,
            right_bracket,
        } => serde_json::json!({
            "type": "set",
            "left_bracket": left_bracket,
            "terms": terms.iter().map(term_to_json).collect::<Vec<_>>(),
            "right_bracket": right_bracket,
        }),
    }
}

pub fn task_from_json(value: &Value) -> Result<Task> {
    let object = value.as_object().context("task must be a JSON object")?;
    reject_unknown_fields(object, &TASK_FIELDS, "task")?;
    let term = term_from_json(
        object.get("term").context("task.term is required")?,
        "task.term",
    )?;
    let punctuation = required_string(object, "punctuation", "task.punctuation")?;
    if !matches!(punctuation.as_str(), "." | "?" | "!") {
        bail!("task.punctuation must be one of '.', '?', or '!'");
    }
    Ok(Task {
        budget: string_array(object, "budget", "task.budget")?,
        sentence: Sentence {
            term,
            punctuation,
            stamp: optional_string(object, "stamp", "task.stamp")?.unwrap_or_default(),
            truth: string_array(object, "truth", "task.truth")?,
        },
    })
}

fn term_from_json(value: &Value, path: &str) -> Result<Term> {
    let object = value
        .as_object()
        .with_context(|| format!("{path} must be a JSON object"))?;
    let kind = required_string(object, "type", &format!("{path}.type"))?;
    match kind.as_str() {
        "atom" => {
            reject_unknown_fields(object, &["type", "prefix", "name"], path)?;
            Ok(Term::Atom {
                prefix: optional_string(object, "prefix", &format!("{path}.atom.prefix"))?
                    .unwrap_or_default(),
                name: required_string(object, "name", &format!("{path}.atom.name"))?,
            })
        }
        "statement" => {
            reject_unknown_fields(object, &["type", "copula", "subject", "predicate"], path)?;
            Ok(Term::Statement {
                copula: required_string(object, "copula", &format!("{path}.statement.copula"))?,
                subject: Box::new(term_from_json(
                    object
                        .get("subject")
                        .with_context(|| format!("{path}.statement.subject is required"))?,
                    &format!("{path}.subject"),
                )?),
                predicate: Box::new(term_from_json(
                    object
                        .get("predicate")
                        .with_context(|| format!("{path}.statement.predicate is required"))?,
                    &format!("{path}.predicate"),
                )?),
            })
        }
        "compound" => {
            reject_unknown_fields(object, &["type", "connecter", "terms"], path)?;
            Ok(Term::Compound {
                connecter: required_string(
                    object,
                    "connecter",
                    &format!("{path}.compound.connecter"),
                )?,
                terms: term_array(object, &format!("{path}.compound.terms"))?,
            })
        }
        "set" => {
            reject_unknown_fields(
                object,
                &["type", "left_bracket", "terms", "right_bracket"],
                path,
            )?;
            Ok(Term::Set {
                left_bracket: required_string(
                    object,
                    "left_bracket",
                    &format!("{path}.set.left_bracket"),
                )?,
                terms: term_array(object, &format!("{path}.set.terms"))?,
                right_bracket: required_string(
                    object,
                    "right_bracket",
                    &format!("{path}.set.right_bracket"),
                )?,
            })
        }
        other => bail!("{path}.type has unsupported value {other:?}"),
    }
}

fn term_array(object: &Map<String, Value>, path: &str) -> Result<Vec<Term>> {
    let values = object
        .get("terms")
        .with_context(|| format!("{path} is required"))?
        .as_array()
        .with_context(|| format!("{path} must be an array"))?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| term_from_json(value, &format!("{path}[{index}]")))
        .collect()
}

fn required_string(object: &Map<String, Value>, key: &str, path: &str) -> Result<String> {
    object
        .get(key)
        .with_context(|| format!("{path} is required"))?
        .as_str()
        .map(str::to_owned)
        .with_context(|| format!("{path} must be a string"))
}

fn optional_string(object: &Map<String, Value>, key: &str, path: &str) -> Result<Option<String>> {
    object
        .get(key)
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .with_context(|| format!("{path} must be a string"))
        })
        .transpose()
}

fn string_array(object: &Map<String, Value>, key: &str, path: &str) -> Result<Vec<String>> {
    let Some(value) = object.get(key) else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .with_context(|| format!("{path} must be an array"))?
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_str()
                .map(str::to_owned)
                .with_context(|| format!("{path}[{index}] must be a string"))
        })
        .collect()
}

fn reject_unknown_fields(object: &Map<String, Value>, allowed: &[&str], path: &str) -> Result<()> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(field.as_str()))
    {
        bail!("{path}.{field} is not recognized");
    }
    Ok(())
}

pub fn output_to_provider_value(output: &Output) -> Value {
    let OutputJSON {
        r#type,
        content,
        narsese,
        operation,
    } = output.to_json_struct();
    let mut value = serde_json::json!({"type": r#type, "content": content});
    if let Some(narsese) = narsese {
        value["narsese"] = Value::String(narsese);
    }
    if let Some(operation) = operation {
        value["operation"] = serde_json::to_value(operation)
            .expect("NAVM operation components are always JSON serializable");
    }
    if matches!(output.type_name(), "IN" | "OUT" | "ANSWER") {
        if let Some(task) = output.get_narsese().and_then(narsese_as_task) {
            value["task"] = task_to_json(&task);
        }
    }
    value
}

fn narsese_as_task(narsese: &Narsese) -> Option<Task> {
    match narsese {
        Narsese::Task(task) => Some(task.clone()),
        Narsese::Sentence(sentence) => Some(Task {
            budget: Vec::new(),
            sentence: sentence.clone(),
        }),
        Narsese::Term(_) => None,
    }
}

pub fn serve<R: BufRead, W: Write>(
    runtime: &mut impl VmRuntime,
    mut input: R,
    output: &mut W,
) -> Result<()> {
    send_frame(
        output,
        &serde_json::json!({
            "kind": "ready",
            "protocol": PROVIDER_PROTOCOL,
            "readiness": "confirmed",
            "features": [],
        }),
    )?;
    while let Some(frame) = read_frame(&mut input, DEFAULT_MAX_FRAME_BYTES)? {
        let request: Value =
            serde_json::from_str(&frame).context("provider request is not valid JSON")?;
        let object = request
            .as_object()
            .context("provider request must be a JSON object")?;
        match required_string(object, "type", "request.type")?.as_str() {
            "command" => {
                let id = required_string(object, "id", "request.id")?;
                validate_id(&id)?;
                let command = object
                    .get("command")
                    .context("request.command is required")?;
                match command_to_navm(command) {
                    Ok(command) => {
                        if let Err(error) = runtime.input_cmd(command) {
                            send_error(output, &id, &error)?;
                            return Err(error.context("NAVM command failed"));
                        }
                        while let Some(value) = runtime.try_fetch_output()? {
                            send_frame(
                                output,
                                &serde_json::json!({
                                    "kind": "output",
                                    "id": id,
                                    "output": output_to_provider_value(&value),
                                }),
                            )?;
                        }
                        send_frame(
                            output,
                            &serde_json::json!({
                                "kind": "command_end",
                                "id": id,
                                "boundary": "sealed",
                            }),
                        )?;
                    }
                    Err(error) => {
                        send_error(output, &id, &error)?;
                        return Err(error);
                    }
                }
            }
            "close" => {
                let id = required_string(object, "id", "request.id")?;
                validate_id(&id)?;
                send_frame(
                    output,
                    &serde_json::json!({"kind": "close_received", "id": id}),
                )?;
                let report = match runtime.terminate() {
                    Ok(()) => serde_json::json!({
                        "disposition": "graceful",
                        "receipt": "processed",
                        "exit": {"state": "not_applicable"},
                        "faults": [],
                    }),
                    Err(error) => serde_json::json!({
                        "disposition": "unknown",
                        "receipt": "processed",
                        "exit": {"state": "not_applicable"},
                        "faults": [{
                            "stage": "provider",
                            "diagnostic": error.to_string(),
                        }],
                    }),
                };
                send_frame(
                    output,
                    &serde_json::json!({"kind": "closed", "id": id, "report": report}),
                )?;
                return Ok(());
            }
            other => bail!("request.type has unsupported value {other:?}"),
        }
    }
    runtime
        .terminate()
        .context("runtime termination failed after provider stdin EOF")?;
    Ok(())
}

fn command_to_navm(command: &Value) -> Result<Cmd> {
    let object = command
        .as_object()
        .context("request.command must be a JSON object")?;
    match required_string(object, "type", "command.type")?.as_str() {
        "input" => Ok(Cmd::NSE(task_from_json(
            object
                .get("narsese")
                .context("command.narsese is required")?,
        )?)),
        "advance" => {
            let cycles = integer_field(object, "cycles", "command.cycles")?;
            Ok(Cmd::CYC(cycles))
        }
        "configure" => {
            let volume = integer_field(object, "volume", "command.volume")?;
            if volume > 100 {
                bail!("command.volume must be between 0 and 100");
            }
            Ok(Cmd::VOL(volume))
        }
        "reset" => {
            if object.get("target").is_some() {
                bail!("command.target is not supported for provider v1 reset");
            }
            Ok(Cmd::RES {
                target: String::new(),
            })
        }
        "register_operation" => {
            bail!("provider v1 register_operation has no NARust-158 mapping")
        }
        "control" => bail!("provider v1 control has no NARust-158 mapping"),
        other => bail!("command.type has unsupported value {other:?}"),
    }
}

fn integer_field(object: &Map<String, Value>, key: &str, path: &str) -> Result<usize> {
    let value = object
        .get(key)
        .with_context(|| format!("{path} is required"))?
        .as_u64()
        .with_context(|| format!("{path} must be a non-negative integer"))?;
    usize::try_from(value).with_context(|| format!("{path} does not fit this platform"))
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 20 || !id.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("request.id must be a decimal string of at most 20 bytes");
    }
    Ok(())
}

fn send_error(output: &mut impl Write, id: &str, error: &anyhow::Error) -> Result<()> {
    send_frame(
        output,
        &serde_json::json!({
            "kind": "error",
            "id": id,
            "message": error.to_string(),
        }),
    )
}

fn send_frame(output: &mut impl Write, frame: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec(frame).context("provider frame could not be serialized")?;
    if bytes.len() > DEFAULT_MAX_FRAME_BYTES {
        bail!("provider output frame exceeds the size limit");
    }
    bytes.push(b'\n');
    output
        .write_all(&bytes)
        .context("provider output frame could not be written")?;
    output
        .flush()
        .context("provider output could not be flushed")
}

fn read_frame(input: &mut impl BufRead, limit: usize) -> Result<Option<String>> {
    let mut frame = Vec::new();
    loop {
        let available = input
            .fill_buf()
            .context("provider input could not be read")?;
        if available.is_empty() {
            if frame.is_empty() {
                return Ok(None);
            }
            break;
        }
        if let Some(newline) = available.iter().position(|byte| *byte == b'\n') {
            if frame.len().saturating_add(newline) > limit {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "provider input frame exceeds the size limit",
                )
                .into());
            }
            frame.extend_from_slice(&available[..newline]);
            input.consume(newline + 1);
            break;
        }
        let consumed = available.len();
        if frame.len().saturating_add(consumed) > limit {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "provider input frame exceeds the size limit",
            )
            .into());
        }
        frame.extend_from_slice(available);
        input.consume(consumed);
    }
    if frame.last() == Some(&b'\r') {
        frame.pop();
    }
    String::from_utf8(frame)
        .map(Some)
        .context("provider input frame is not UTF-8")
}
