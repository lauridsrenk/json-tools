mod error;

use clap::{Parser, ValueEnum};
use serde_json::{json, Value};
use std::path::PathBuf;

use error::{PathNotFound, WrongValueAtPath};

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

#[derive(ValueEnum, Copy, Clone, Default, Debug)]
enum InputFormat {
    #[default]
    Json,
}

#[derive(ValueEnum, Copy, Clone, Default, Debug)]
enum OutputFormat {
    #[default]
    JsonCompact,
    JsonPretty,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
enum Actions {
    /// transforms a Dictionary to an Array
    ToArray {
        #[command(subcommand)]
        method: ToArray,
    },
    /// transforms an Array with a specified key value to a Dictionary
    ToDict {
        #[arg(short, long)]
        /// the name of the key-field
        key: String,
    },
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
enum ToArray {
    /// reshapes key and value into a "{key: key, value: value}"-like object
    Box {
        #[arg(short, long)]
        /// the name of the generated key-field
        key_name: String,
        #[arg(short, long)]
        /// the name of the generated value-field
        value_name: String,
    },
    /// Integrates the key into the value-object
    Integrate {
        #[arg(short, long)]
        /// the name of the generated key-field
        key_name: String,
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
            Actions::ToArray { method } => method.apply(value),
            Actions::ToDict { .. } => {
                todo!()
            }
        }
    }
}

impl ToArray {
    fn apply(&self, value: Value) -> anyhow::Result<Value> {
        let value = get_sub_value(value, None)?;
        let v: anyhow::Result<Vec<Value>> = value
            .as_object()
            .unwrap()
            .into_iter()
            .map(|(key, val)| match self {
                ToArray::Box {
                    key_name,
                    value_name,
                } => Ok(json!({ key_name: key, value_name: val })),
                ToArray::Integrate { key_name } => {
                    let mut val = val.clone();
                    val.as_object_mut()
                        .ok_or(WrongValueAtPath { at_path: "".into() })?
                        .insert(key_name.clone(), Value::String(key.into()));
                    Ok(val)
                }
            })
            .collect();
        Ok(Value::Array(v?))
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let in_val = cli.read_source()?;
    let out_val = cli.action.apply(in_val)?;
    cli.out(out_val)?;
    Ok(())
}
