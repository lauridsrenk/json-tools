use clap::{Parser, ValueEnum};
use serde_json::{json, Value};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    /// optional input file. reads from STDIN if not specified
    source: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    /// optional output file. prints to STDOUT if not specified
    dest: Option<PathBuf>,

    #[arg(short, long)]
    /// input format. defaults to json
    input_format: Option<InputFormat>,

    #[arg(short, long)]
    /// output format. defaults to json-compact
    output_format: Option<OutputFormat>,

    #[command(subcommand)]
    action: Actions,
}

#[derive(ValueEnum,Copy,Clone,Default,Debug)]
enum InputFormat {
    #[default] Json,
}

#[derive(ValueEnum,Copy,Clone,Default,Debug)]
enum OutputFormat {
    #[default] JsonCompact,
    JsonPretty,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
enum Actions {
    /// transforms a Dictionary to a "[{key: key, val: val}]"-Array
    ToArray {
        #[arg(short, long)]
        /// the name of the generated key-field
        key_name: String,
        #[arg(short, long)]
        /// the name of the generated value-field
        value_name: String,
    },
    /// transforms an Array with a specified key value to a Dictionary
    ToDict {
        #[arg(short, long)]
        /// the name of the key-field
        key: String,
    },
}

impl Cli {
    fn read_source(&self) -> anyhow::Result<Value> {
        let res = match &self.source {
            Some(path) => std::fs::read_to_string(path)?.parse(),
            None => Ok(Value::Null),
        }?;
        Ok(res)
    }

    fn out(&self, value: Value) -> anyhow::Result<()> {
        let formatted = match self.output_format.unwrap_or_default() {
            OutputFormat::JsonCompact => serde_json::to_string(&value),
            OutputFormat::JsonPretty => serde_json::to_string_pretty(&value),
        }?;

        match &self.dest {
            Some(path) => {
                std::fs::write(path, formatted)?;
                Ok(())
            }
            None => {
                println!("{formatted}");
                Ok(())
            }
        }
    }
}

impl Actions {
    fn apply(&self, value: Value) -> anyhow::Result<Value> {
        match self {
            Actions::ToArray { key_name, value_name } => {
                let value = get_sub_value(value, None)?;
                let v: Vec<Value> = value
                    .as_object()
                    .unwrap()
                    .into_iter()
                    .map(|(key, val)| json!({ key_name: key, value_name: val }))
                    .collect();
                Ok(Value::Array(v))
            }
            Actions::ToDict { .. } => {
                todo!()
            }
        }
    }
}

fn get_sub_value(value: Value, path: Option<&str>) -> anyhow::Result<Value> {
    match path {
        None => Ok(value),
        Some(path) => path.split('.').try_fold(value, |mut value, segment| {
            let obj = value.as_object_mut().ok_or(WrongValueAtPath {
                at_path: segment.into(),
            })?;
            let val = obj.remove(segment).ok_or(PathNotFound {
                at_path: segment.into(),
            })?;
            Ok(val)
        }),
    }
}

#[derive(Debug, Error)]
#[error("could not find {at_path}")]
struct PathNotFound {
    at_path: String,
}

#[derive(Debug, Error)]
#[error("value at path {at_path} is not a Dict")]
struct WrongValueAtPath {
    at_path: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let in_val = cli.read_source()?;
    let out_val = cli.action.apply(in_val)?;
    cli.out(out_val)?;
    Ok(())
}
