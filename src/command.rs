use crate::cli::Command;
use crate::query::EvaluationError;
use crate::storage::{Storage, StorageError};
use crate::task::{Status, Task};
use chrono::NaiveDateTime;
use inquire::{CustomType, InquireError, Select, Text};
use std::fmt::{Debug, Display, Formatter};
use inquire::validator::ValueRequiredValidator;
use thiserror::Error;

impl Command {

    /// Runs the command
    pub fn run(self, storage: &Storage<Task>) -> Result<(), CommandError> {

        match self {
            Command::Add(task) => {
                if let Some(prev_task) = storage.insert(&task.name, &task)? {
                    println!("Replaced task: \n{prev_task}");
                };
            }
            Command::Done { task_name } => {
                let is_updated = storage.update(&task_name, |task| task.status = Status::On)?;
                if !is_updated {
                    println!("Task not found");
                }
            }
            Command::Update { task_name } => {
                let task = storage.get(&task_name)?;
                if let Some(task) = task {
                    let updated_task = Self::interactive_update(task)?;
                    let prev_task = storage.insert(&updated_task.name, &updated_task)?;
                    if updated_task.name != task_name {
                        storage.delete(&task_name)?;
                        if let Some(prev_task) = prev_task {
                            println!("Replaced task: \n{prev_task}")
                        }
                    }
                } else {
                    println!("Task not found");
                }
            }
            Command::Delete { task_name } => {
                if let None = storage.delete(&task_name)?{
                    println!("Task not found");
                }
            }
            Command::Select(query) => {
                let result_set = storage.select(query.0)?;
                println!("{result_set}");
            }
        }

        Ok(())
    }

    fn interactive_update(mut task: Task) -> Result<Task, InquireError> {
        task.name = Text::new("Name: ")
            .with_validator(ValueRequiredValidator::new("This field is required."))
            .with_default(&task.name)
            .prompt()?;

        task.description = Text::new("Description: ")
            .with_validator(ValueRequiredValidator::new("This field is required."))
            .with_default(&task.description)
            .prompt()?;

        task.date = CustomType::new("Date: ")
            .with_parser(&|date| {
                NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M")
                    .map(|date| date.and_utc())
                    .map_err(|_| ())
            })
            .with_error_message("Failed to parse date.")
            .with_help_message("Date must be in format: '%Y-%m-%d %H:%M'")
            .with_default(task.date)
            .with_default_value_formatter(&|date| date.format("%Y-%m-%d %H:%M").to_string())
            .with_formatter(&|date| date.format("%Y-%m-%d %H:%M").to_string())
            .prompt()?;

        task.category = Text::new("Category: ")
            .with_validator(ValueRequiredValidator::new("This field is required"))
            .with_default(&task.category)
            .prompt()?;
        task.status = Select::new("Status: ", Vec::from([Status::On, Status::Off]))
            .with_starting_cursor(if task.status == Status::On { 0 } else { 1 })
            .prompt()?;

        Ok(task)
    }
}

/// Represents possible errors of running command.
#[derive(Error)]
pub enum CommandError {
    #[error("Failed to read/write task from storage. \nReason: {0}")]
    Storage(#[from] StorageError),
    #[error("Failed to execute query. {0}")]
    QueryEvaluation(#[from] EvaluationError),
    #[error("Failed to read line. \nReason: {0}")]
    Readline(#[from] InquireError)
}

impl Debug for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}