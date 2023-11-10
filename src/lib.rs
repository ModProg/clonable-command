#![warn(clippy::pedantic, missing_docs, clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Anoyingly [`std::process::Command`] does not implement e.g. [`Clone`].
//!
//! This is due to these containing fields that cannor be easily support this [rust-lang/rust#22858 etc.](https://github.com/rust-lang/rust/pull/22858).

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::{Child, Command as StdCommand, ExitStatus, Output, Stdio as StdStdio};

/// Enum version of [std's Stdio](StdStdio), allowing easy copying and
/// serialization.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stdio {
    /// A new pipe should be arranged to connect the parent and child processes.
    /// [more](StdStdio::piped)
    Piped,
    /// The child inherits from the corresponding parent descriptor.
    /// [more](StdStdio::inherit)
    Inherit,
    /// This stream will be ignored. This is the equivalent of attaching the
    /// stream to `/dev/null`. [more](StdStdio::null)
    Null,
}

impl From<Stdio> for StdStdio {
    fn from(value: Stdio) -> Self {
        match value {
            Stdio::Inherit => StdStdio::inherit(),
            Stdio::Piped => StdStdio::piped(),
            Stdio::Null => StdStdio::null(),
        }
    }
}

// TODO platform specific extentions

/// A process builder, providing fine-grained control over how a new process
/// should be spawned. Equivalent to [std's Command](StdCommand) but allowing
/// field access cloning and serialization.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    /// Name of the program invoked.
    pub name: OsString,
    /// Arguments passed to the child process.
    pub arguments: Vec<OsString>,
    /// Controlls whether the child process will inherit the parent process'
    /// environment.
    pub inherit_environment: bool,
    /// Environment for the child process, `None` represents variables that will
    /// not be inherited from the parent, even when `inherit_environment ==
    /// true`.
    pub environment: HashMap<OsString, Option<OsString>>,
    /// Working directory for the child process.
    pub current_dir: Option<PathBuf>,
    /// Child process' standard input (stdin) handle, [`None` will use default
    /// for invocation type](StdCommand::stdin).
    pub stdin: Option<Stdio>,
    /// Child process' standard output (stdout) handle, [`None` will use default
    /// for invocation type](StdCommand::stdout).
    pub stdout: Option<Stdio>,
    /// Child process' standard error (stderr) handle, [`None` will use default
    /// for invocation type](StdCommand::stderr).
    pub stderr: Option<Stdio>,
}

/// Builder
impl Command {
    /// Constructs a new `Command`. [more](StdCommand::new)
    #[must_use]
    pub fn new(name: impl AsRef<OsStr>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            arguments: Vec::new(),
            inherit_environment: true,
            environment: HashMap::new(),
            current_dir: None,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    /// Adds an argument to pass to the program. [more](StdCommand::arg)
    #[must_use]
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.add_arg(arg);
        self
    }

    /// Adds multiple arguments to pass to the program. [more](StdCommand::args)
    #[must_use]
    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self {
        self.add_args(args);
        self
    }

    /// Inserts or updates an environment variable. [more](StdCommand::env)
    #[must_use]
    pub fn env(mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> Self {
        self.set_env(key, val);
        self
    }

    /// Inserts or updates multiple environment variables.
    /// [more](StdCommand::envs)
    #[must_use]
    pub fn envs(
        mut self,
        vars: impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>,
    ) -> Self {
        self.set_envs(vars);
        self
    }

    /// Removes an explicitly set environment variable and prevents inheriting
    /// it from a parent process. [more](StdCommand::env_remove)
    #[must_use]
    pub fn env_remove(mut self, key: impl AsRef<OsStr>) -> Self {
        self.remove_env(key);
        self
    }

    /// Clears all explicitly set environment variables and prevents inheriting
    /// any parent process environment variables. [more](StdCommand::env_clear)
    #[must_use]
    pub fn env_clear(mut self) -> Self {
        self.environment.clear();
        self.inherit_environment = false;
        self
    }

    /// Prevents inheriting any parent process environment variables (like
    /// [`env_clear`](Self::env_clear) without clearing set envs).
    #[must_use]
    pub fn env_no_inherit(mut self) -> Self {
        self.inherit_environment = false;
        self
    }

    /// Sets the working directory for the child process.
    /// [more](StdCommand::current_dir)
    #[must_use]
    pub fn current_dir(mut self, key: impl AsRef<Path>) -> Self {
        self.set_current_dir(key);
        self
    }

    /// Configuration for the child process’s standard input (stdin) handle.
    /// [more](StdCommand::stdin)
    #[must_use]
    pub fn stdin(mut self, stdin: Stdio) -> Self {
        self.stdin = Some(stdin);
        self
    }

    /// Configuration for the child process’s standard output (stdout) handle.
    /// [more](StdCommand::stdout)
    #[must_use]
    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = Some(stdout);
        self
    }

    /// Configuration for the child process’s standard error (stderr) handle.
    /// [more](StdCommand::stderr)
    #[must_use]
    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = Some(stderr);
        self
    }
}
/// Setters
impl Command {
    /// Adds an argument to pass to the program. [more](StdCommand::arg)
    pub fn add_arg(&mut self, arg: impl AsRef<OsStr>) {
        self.arguments.push(arg.as_ref().to_owned());
    }

    /// Adds multiple arguments to pass to the program. [more](StdCommand::args)
    pub fn add_args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) {
        self.arguments
            .extend(args.into_iter().map(|i| i.as_ref().to_owned()));
    }

    /// Inserts or updates an environment variable. [more](StdCommand::env)
    pub fn set_env(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) {
        self.environment
            .insert(key.as_ref().to_owned(), Some(val.as_ref().to_owned()));
    }

    /// Inserts or updates multiple environment variables.
    /// [more](StdCommand::envs)
    pub fn set_envs(
        &mut self,
        vars: impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>,
    ) {
        self.environment.extend(
            vars.into_iter()
                .map(|(k, v)| (k.as_ref().to_owned(), Some(v.as_ref().to_owned()))),
        );
    }

    /// Removes an explicitly set environment variable and prevents inheriting
    /// it from a parent process. [more](StdCommand::env_remove)
    pub fn remove_env(&mut self, key: impl AsRef<OsStr>) {
        self.environment.remove(key.as_ref());
    }

    /// Sets the working directory for the child process.
    /// [more](StdCommand::current_dir)
    pub fn set_current_dir(&mut self, path: impl AsRef<Path>) {
        self.current_dir = Some(path.as_ref().to_owned());
    }
}

impl From<&Command> for StdCommand {
    fn from(
        Command {
            name,
            arguments: args,
            inherit_environment: inherit_env,
            environment: env,
            current_dir,
            stdin,
            stdout,
            stderr,
        }: &Command,
    ) -> Self {
        let mut command = StdCommand::new(name);
        if *inherit_env {
            // Only need to remove inherited vars if not cleared
            for (removed, _) in env.iter().filter(|i| i.1.is_none()) {
                command.env_remove(removed);
            }
        } else {
            command.env_clear();
        }
        command
            .args(args)
            .envs(env.iter().filter_map(|(k, v)| v.as_ref().map(|v| (k, v))));
        current_dir
            .as_ref()
            .map(|current_dir| command.current_dir(current_dir));
        stdin.map(|stdin| command.stdin(stdin));
        stdout.map(|stdout| command.stdout(stdout));
        stderr.map(|stderr| command.stderr(stderr));

        command
    }
}

/// Execution
impl Command {
    /// Behaves identical to std's [`Command::spawn`](StdCommand::spawn).
    pub fn spawn(&self) -> Result<Child> {
        StdCommand::from(self).spawn()
    }

    /// Behaves identical to std's [`Command::output`](StdCommand::output).
    pub fn output(&self) -> Result<Output> {
        StdCommand::from(self).output()
    }

    /// Behaves identical to std's [`Command::status`](StdCommand::status).
    pub fn status(&self) -> Result<ExitStatus> {
        StdCommand::from(self).status()
    }
}
