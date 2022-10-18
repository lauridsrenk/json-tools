use clap::Parser;
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
    #[command(subcommand)]
    action: Actions,
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
        match &self.dest {
            Some(path) => {
                std::fs::write(path, value.to_string())?;
                Ok(())
            }
            None => {
                println!("{}", value);
                Ok(())
            }
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
enum Actions {
    /// transforms a Dictionary to an "[{key: key, val: val}]"-Array
    ToArray {
        #[arg(short, long)]
        /// the name of the generated key-field
        key_name: String,
        #[arg(short, long)]
        /// the path to the dict. will use the source itself if not specified
        path: Option<String>,
    },
    /// transforms an Array with a specified key value to a Dictionary
    ToDict {
        #[arg(short, long)]
        /// the name of the key-field
        key: String,
        #[arg(short, long)]
        /// the path to the array. will use the source itself if not specified
        path: Option<String>,
    },
}

impl Actions {
    fn apply(&self, value: Value) -> anyhow::Result<Value> {
        match self {
            Actions::ToArray { key_name, path } => {
                let value = get_sub_value(value, path.as_deref())?;
                let value = get_sub_value(value, Some(key_name))?;
                let v: Vec<Value> = value
                    .as_object()
                    .unwrap()
                    .into_iter()
                    .map(|(key, val)| json!({ key_name: key, "value": val }))
                    .collect();
                Ok(Value::Array(v))
            }
            Actions::ToDict { key, path } => {
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
