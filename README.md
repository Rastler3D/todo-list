# todo-list


`todo-list` is a simple todo list command-line app written in Rust.


## Install

You can install `todo-list` with the following command:

```console
$ cargo install --git https://github.com/Rastler3D/todo-list.git
```


## Usage

```console
$ todo-list help
Simple todo-list command-line app

Usage: todo-list.exe <COMMAND>

Commands:
  add     Add task to list
  done    Mark task as completed
  update  Update task
  delete  Delete task
  select  Select tasks
  repl    Run app in repl mode
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

```

REPL

```console
$ todo-list repl
<<  SELECT status, date, * WHERE date > "2020-12-12 00:00" AND (category = "category" OR description LIKE "descript") AND status = "on"
╭────────┬──────────────────┬──────┬─────────────┬──────────╮
│ status │ date             │ name │ description │ category │
├────────┼──────────────────┼──────┼─────────────┼──────────┤
│ on     │ 2024-10-10 20:10 │ task │ description │ category │
╰────────┴──────────────────┴──────┴─────────────┴──────────╯
<<  UPDATE task
> Name:  task2
> Description:  description
> Date:  2024-10-10 20:10
> Category:  category
> Status:  on
<<  SELECT *
╭───────┬─────────────┬──────────────────┬──────────┬────────╮
│ name  │ description │ date             │ category │ status │
├───────┼─────────────┼──────────────────┼──────────┼────────┤
│ task2 │ description │ 2024-10-10 20:10 │ category │ on     │
╰───────┴─────────────┴──────────────────┴──────────┴────────╯
<<
```

Add new todo

```console
$ todo-list add --help
Add task to list

Usage: todo-list.exe add <NAME> <DESCRIPTION> <DATE> <CATEGORY> <STATUS>

Arguments:
  <NAME>
  <DESCRIPTION>
  <DATE>
  <CATEGORY>
  <STATUS>       [possible values: on, off]

Options:
  -h, --help  Print help

```

Mark todo as complete

```console
$ todo-list done --help
Mark task as completed

Usage: todo-list.exe done <TASK_NAME>

Arguments:
  <TASK_NAME>

Options:
  -h, --help  Print help

```

Update todo

```sh-
$ todo-list update --help
Update task

Usage: todo-list.exe update <TASK_NAME>

Arguments:
  <TASK_NAME>

Options:
  -h, --help  Print help
  
$ todo-list update task
> Name:  task
> Description:  description
> Date:  2024-10-10 20:10
> Category:  category
> Status:  on
```
Delete todo

```console
$ todo-list update --help
Delete task

Usage: todo-list.exe delete <TASK_NAME>

Arguments:
  <TASK_NAME>

Options:
  -h, --help  Print help
```
Select todo

```console
$ todo-list select --help
Select tasks

Usage: todo-list.exe select <QUERY>...

Arguments:
  <QUERY>...

Options:
  -h, --help  Print help
  
$ todo-list select date, * where status = 'on' and date = '2024-10-10 20:10'
╭──────────────────┬──────┬─────────────┬──────────┬────────╮
│ date             │ name │ description │ category │ status │
├──────────────────┼──────┼─────────────┼──────────┼────────┤
│ 2024-10-10 20:10 │ task │ description │ category │ on     │
╰──────────────────┴──────┴─────────────┴──────────┴────────╯
```

## License

The CLI is available as open source under the terms of the [MIT License](http://opensource.org/licenses/MIT).
