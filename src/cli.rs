use std::iter::once;
use crate::command::CommandError;
use crate::query::Query;
use crate::task::Task;
use clap::builder::ValueParser;
use clap::{
    Arg, ArgAction, ArgMatches, Args, Error, FromArgMatches, Id, Parser,
};
use std::str::FromStr;
use inquire::InquireError;
use crate::storage::Storage;

const TODO_FILE_STORAGE: &str = "todo";

/// Cli command. May be specific command or read-eval-print-loop.
#[derive(Debug, Parser, PartialEq)]
#[command(about = "Simple todo-list command-line app")]
pub enum Cli {
    #[command(flatten)]
    Command(Command),
    #[command(about = "Run app in repl mode")]
    Repl,
}

/// Possible commands.
///
/// * `Command::Add` - Add task to list;
/// * `Command::Done` - Mark task as completed;
/// * `Command::Update` - Interactively update task;
/// * `Command::Delete` - Delete task;
/// * `Command::Select` - Select tasks that satisfy query;
#[derive(Debug, Parser, PartialEq)]
#[command(name = "", about = "Todo list commands")]
pub enum Command {
    #[command(alias = "ADD", about  = "Add task to list")]
    Add(Task),
    #[command(alias = "DONE", about  = "Mark task as completed")]
    Done { task_name: String },
    #[command(alias = "UPDATE", about  = "Update task")]
    Update { task_name: String },
    #[command(alias = "DELETE", about  = "Delete task")]
    Delete { task_name: String },
    #[command(alias = "SELECT", about  = "Select tasks")]
    Select(Select),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select(pub Query);

impl Cli {
    /// Runs the command or read-eval-print-loop
    pub fn run(self) -> Result<(), CommandError> {
        let storage = Storage::open(TODO_FILE_STORAGE)?;
        match self {
            Cli::Command(command) => command.run(&storage),
            Cli::Repl => loop {
                let line =  match repl::readline() {
                    Ok(value) => value,
                    Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
                    Err(err) => {
                        eprintln!("{}", CommandError::Readline(err));
                        continue;
                    }
                };
                let line = line.trim();
                if line.is_empty(){
                    continue;
                }
                let command = match repl::parse(line) {
                    Ok(command) => command,
                    Err(err) => {
                        eprintln!("{err}");
                        continue;
                    }
                };

                match command.run(&storage) {
                    Ok(_) => continue,
                    Err(err) => {
                        eprintln!("{err}");
                        continue;
                    }
                }
            },
        }
    }
}

mod repl {
    use clap::Parser;
    use inquire::ui::{Color, RenderConfig, Styled};
    use inquire::{InquireError, Text};
    use crate::cli::Command;

    pub fn readline() -> Result<String, InquireError> {
        Text::new("")
            .with_render_config(
                RenderConfig::default()
                    .with_prompt_prefix(Styled::new("<<").with_fg(Color::DarkBlue))
                    .with_answered_prompt_prefix(Styled::new("<<").with_fg(Color::DarkGreen)),
            )
            .prompt()
    }

    pub fn parse(line: &str) -> Result<Command, clap::Error> {
        let args = if line.starts_with("SELECT") || line.starts_with("select"){
            line.split_whitespace().map(ToString::to_string).collect()
        } else {
            shlex::split(line).unwrap_or(Vec::new())
        };

        Command::try_parse_from(std::iter::once(String::new()).chain(args))
    }
}

/// Parse query from command line arguments
impl FromArgMatches for Select {
    fn from_arg_matches(arg_matches: &ArgMatches) -> Result<Self, Error> {
        Self::from_arg_matches_mut(&mut arg_matches.clone())
    }
    fn from_arg_matches_mut(arg_matches: &mut ArgMatches) -> Result<Self, Error> {
        let query = arg_matches
            .remove_many::<String>("query")
            .map(|v| once("SELECT".to_string()).chain(v).collect::<Vec<_>>())
            .unwrap_or_else(Vec::new)
            .join(" ");

        Query::from_str(&query)
            .map(Select)
            .map_err(|err| clap::Error::raw(clap::error::ErrorKind::InvalidValue, err))
    }
    fn update_from_arg_matches(&mut self, arg_matches: &ArgMatches) -> Result<(), Error> {
        self.update_from_arg_matches_mut(&mut arg_matches.clone())
    }
    fn update_from_arg_matches_mut(&mut self, arg_matches: &mut ArgMatches) -> Result<(), Error> {
        if arg_matches.contains_id("query") {
            *self = Select::from_arg_matches(arg_matches)?;
        }
        Ok(())
    }
}
impl Args for Select {
    fn group_id() -> Option<Id> {
        Some(Id::from("Select"))
    }
    fn augment_args<'b>(app: clap::Command) -> clap::Command {
        app.arg(
            Arg::new("query")
                .value_name("QUERY")
                .value_parser(ValueParser::string())
                .required(true)
                .trailing_var_arg(true)
                .allow_hyphen_values(true)
                .action(ArgAction::Append),
        )
    }
    fn augment_args_for_update<'b>(app: clap::Command) -> clap::Command {
        app.arg(
            Arg::new("query")
                .value_name("QUERY")
                .value_parser(ValueParser::string())
                .required(false)
                .trailing_var_arg(true)
                .allow_hyphen_values(true)
                .action(ArgAction::Append),
        )
    }
}


#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use crate::query::ast::{Field, FieldsProjection, Predicate};
    use crate::query::ast::expression::{BinaryOp, BinaryOperation, Expression, Identifier, Literal, Operation};
    use crate::query::ast::expression::Number;
    use crate::task::Status;
    use super::*;
    #[test]
    fn select_command() {
        let cmd = shlex::split("todo-list select * where predicate = 10").unwrap_or_default();
        let command = Cli::try_parse_from(cmd).unwrap();
        let expected = Cli::Command(Command::Select(Select(Query{
            fields_projection: FieldsProjection(Vec::from([Field::Asterisk])),
            predicate: Some(Predicate{
                expr: Expression::Operation(Box::new(Operation::Binary(BinaryOperation{
                    left_expression: Expression::Identifier(Identifier("predicate".to_string())),
                    right_expression: Expression::Literal(Literal::Number(Number::Int(10))),
                    op: BinaryOp::Eq
                })))
            })
        })));

        assert_eq!(command, expected)
    }

    #[test]
    fn add_command() {
        let cmd = shlex::split("todo-list add name description \"2020-12-12 20:20\" category off").unwrap_or_default();
        let command = Cli::try_parse_from(cmd).unwrap();
        let expected = Cli::Command(Command::Add(Task{
            name: "name".to_string(),
            description: "description".to_string(),
            date: NaiveDateTime::parse_from_str("2020-12-12 20:20", "%Y-%m-%d %H:%M")
                .unwrap()
                .and_utc(),
            category: "category".to_string(),
            status: Status::Off
        }));

        assert_eq!(command, expected)
    }
}