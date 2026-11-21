//! Application kernel, command registry, argument parser, and execution orchestrator.

use std::{collections::HashMap, sync::Arc};

use colored::Colorize;

use crate::{
    command::{Command, Context, Flag, FlagType, FlagValue},
    error::KarigorError,
    ui::Ui,
};

fn levenshtein(first: &str, second: &str) -> usize {
    let first_chars: Vec<char> = first.chars().collect();
    let second_chars: Vec<char> = second.chars().collect();
    let mut distances = vec![vec![0; second_chars.len() + 1]; first_chars.len() + 1];

    for (index, row) in distances.iter_mut().enumerate() {
        row[0] = index;
    }
    for (index, cell) in distances[0].iter_mut().enumerate() {
        *cell = index;
    }

    for (first_index, &first_char) in first_chars.iter().enumerate() {
        for (second_index, &second_char) in second_chars.iter().enumerate() {
            let cost = if first_char == second_char { 0 } else { 1 };
            distances[first_index + 1][second_index + 1] =
                (distances[first_index][second_index + 1] + 1)
                    .min(distances[first_index + 1][second_index] + 1)
                    .min(distances[first_index][second_index] + cost);
        }
    }

    distances[first_chars.len()][second_chars.len()]
}

/// Central application orchestrator managing command registration, parsing, and execution.
pub struct Kernel {
    name: String,
    description: String,
    version: String,
    commands: HashMap<String, Arc<dyn Command>>,
    groups: HashMap<String, String>,
    aliases: HashMap<String, String>,
    ui: Ui,
}

impl Kernel {
    /// Creates a new `Kernel` instance with the given application name and description.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            version: "0.1.0".to_string(),
            commands: HashMap::new(),
            groups: HashMap::new(),
            aliases: HashMap::new(),
            ui: Ui::new(),
        }
    }

    /// Sets the application version string.
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Registers a command into the kernel.
    #[must_use]
    pub fn register<C: Command + 'static>(mut self, command: C) -> Self {
        let name = command.name().to_string();
        for &alias in command.aliases() {
            self.aliases.insert(alias.to_string(), name.clone());
        }
        self.commands.insert(name, Arc::new(command));
        self
    }

    /// Registers a descriptive group label for namespaced commands (e.g. `"make"`).
    #[must_use]
    pub fn group(mut self, namespace: impl Into<String>, description: impl Into<String>) -> Self {
        self.groups.insert(namespace.into(), description.into());
        self
    }

    /// Executes command dispatch against the process environment arguments.
    pub async fn handle_env(&self) -> Result<(), KarigorError> {
        let raw_args: Vec<String> = std::env::args().collect();
        self.handle(raw_args).await
    }

    /// Normalizes incoming command line tokens, handling cargo subcommand invocations and colon namespaces.
    #[must_use]
    pub fn normalize_args(&self, raw: &[String]) -> Vec<String> {
        if raw.is_empty() {
            return Vec::new();
        }

        let mut tokens: Vec<String> = raw.to_vec();

        // 1. Strip binary name if present
        if !tokens.is_empty()
            && (tokens[0].ends_with(&self.name)
                || tokens[0].ends_with("cargo")
                || tokens[0].contains('/'))
        {
            tokens.remove(0);
        }

        // 2. Strip cargo subcommand alias if invoked as `cargo <name> ...`
        if !tokens.is_empty() && tokens[0] == self.name {
            tokens.remove(0);
        }

        // 3. Normalizes space-separated subcommands to colon commands (e.g. `make route` -> `make:route`)
        if tokens.len() >= 2 && !tokens[0].starts_with('-') && !tokens[1].starts_with('-') {
            let colon_candidate = format!("{}:{}", tokens[0], tokens[1]);
            if self.commands.contains_key(&colon_candidate)
                || self.aliases.contains_key(&colon_candidate)
            {
                tokens.remove(0);
                tokens[0] = colon_candidate;
            }
        }

        tokens
    }

    /// Dispatches execution to the appropriate command based on input arguments.
    pub async fn handle(&self, raw_args: Vec<String>) -> Result<(), KarigorError> {
        let tokens = self.normalize_args(&raw_args);

        if tokens.is_empty() || tokens[0] == "-h" || tokens[0] == "--help" || tokens[0] == "help" {
            self.print_help();
            return Ok(());
        }

        if tokens[0] == "-V" || tokens[0] == "--version" || tokens[0] == "version" {
            println!("{} {}", self.name.bold().cyan(), self.version);
            return Ok(());
        }

        let requested_name = &tokens[0];
        let resolved_name = self
            .aliases
            .get(requested_name)
            .unwrap_or(requested_name)
            .to_string();

        let command = match self.commands.get(&resolved_name) {
            Some(cmd) => Arc::clone(cmd),
            None => {
                let suggestion = self.find_closest_match(requested_name);
                return Err(KarigorError::CommandNotFound {
                    name: requested_name.clone(),
                    suggestion,
                });
            }
        };

        // Check if command-specific help is requested
        if tokens.iter().any(|t| t == "--help" || t == "-h") {
            self.print_command_help(command.as_ref());
            return Ok(());
        }

        let context = self.parse_command_context(command.as_ref(), &tokens[1..])?;
        let mut mutable_context = context;

        command.run(&mut mutable_context).await
    }

    fn find_closest_match(&self, input: &str) -> Option<String> {
        let mut closest = None;
        let mut min_distance = 3;

        for name in self.commands.keys().chain(self.aliases.keys()) {
            let distance = levenshtein(input, name);
            if distance < min_distance {
                min_distance = distance;
                closest = Some(name.clone());
            }
        }

        closest
    }

    fn parse_command_context(
        &self,
        command: &dyn Command,
        args_slice: &[String],
    ) -> Result<Context, KarigorError> {
        let declared_args = command.arguments();
        let declared_flags = command.flags();

        let mut flag_lookup: HashMap<String, &Flag> = HashMap::new();
        let mut short_lookup: HashMap<char, &Flag> = HashMap::new();
        for flag in &declared_flags {
            flag_lookup.insert(flag.name.clone(), flag);
            if let Some(ch) = flag.short {
                short_lookup.insert(ch, flag);
            }
        }

        let mut parsed_flags: HashMap<String, FlagValue> = HashMap::new();
        let mut positional_values = Vec::new();
        let mut index = 0;

        while index < args_slice.len() {
            let token = &args_slice[index];

            if let Some(long_name) = token.strip_prefix("--") {
                if let Some((name, val)) = long_name.split_once('=') {
                    if let Some(flag) = flag_lookup.get(name) {
                        match flag.flag_type {
                            FlagType::Boolean => {
                                parsed_flags.insert(
                                    flag.name.clone(),
                                    FlagValue::Bool(val.parse().unwrap_or(true)),
                                );
                            }
                            FlagType::String => {
                                parsed_flags
                                    .insert(flag.name.clone(), FlagValue::Str(val.to_string()));
                            }
                        }
                    } else {
                        parsed_flags.insert(name.to_string(), FlagValue::Str(val.to_string()));
                    }
                } else if let Some(flag) = flag_lookup.get(long_name) {
                    match flag.flag_type {
                        FlagType::Boolean => {
                            parsed_flags.insert(flag.name.clone(), FlagValue::Bool(true));
                        }
                        FlagType::String => {
                            index += 1;
                            if index < args_slice.len() && !args_slice[index].starts_with('-') {
                                parsed_flags.insert(
                                    flag.name.clone(),
                                    FlagValue::Str(args_slice[index].clone()),
                                );
                            } else if let Some(ref default) = flag.default {
                                parsed_flags
                                    .insert(flag.name.clone(), FlagValue::Str(default.clone()));
                            } else {
                                return Err(KarigorError::InvalidFlag(format!(
                                    "Option `--{}` requires a value",
                                    flag.name
                                )));
                            }
                        }
                    }
                } else {
                    parsed_flags.insert(long_name.to_string(), FlagValue::Bool(true));
                }
            } else if let Some(short_str) = token.strip_prefix('-') {
                for ch in short_str.chars() {
                    if let Some(flag) = short_lookup.get(&ch) {
                        match flag.flag_type {
                            FlagType::Boolean => {
                                parsed_flags.insert(flag.name.clone(), FlagValue::Bool(true));
                            }
                            FlagType::String => {
                                index += 1;
                                if index < args_slice.len() && !args_slice[index].starts_with('-') {
                                    parsed_flags.insert(
                                        flag.name.clone(),
                                        FlagValue::Str(args_slice[index].clone()),
                                    );
                                } else if let Some(ref default) = flag.default {
                                    parsed_flags
                                        .insert(flag.name.clone(), FlagValue::Str(default.clone()));
                                } else {
                                    return Err(KarigorError::InvalidFlag(format!(
                                        "Option `-{}` requires a value",
                                        ch
                                    )));
                                }
                            }
                        }
                    }
                }
            } else {
                positional_values.push(token.clone());
            }

            index += 1;
        }

        // Apply defaults for unsupplied flags
        for flag in &declared_flags {
            if !parsed_flags.contains_key(&flag.name) {
                if let Some(ref default_val) = flag.default {
                    parsed_flags.insert(flag.name.clone(), FlagValue::Str(default_val.clone()));
                } else if flag.flag_type == FlagType::Boolean {
                    parsed_flags.insert(flag.name.clone(), FlagValue::Bool(false));
                }
            }
        }

        // Map positional arguments
        let mut parsed_args = HashMap::new();
        for (arg_idx, arg_spec) in declared_args.iter().enumerate() {
            if arg_idx < positional_values.len() {
                parsed_args.insert(arg_spec.name.clone(), positional_values[arg_idx].clone());
            } else if let Some(ref default) = arg_spec.default {
                parsed_args.insert(arg_spec.name.clone(), default.clone());
            } else if arg_spec.required {
                return Err(KarigorError::InvalidArgument(format!(
                    "Missing required argument `{}`",
                    arg_spec.name
                )));
            }
        }

        Ok(Context {
            args: parsed_args,
            flags: parsed_flags,
            raw_args: positional_values,
            ui: self.ui.clone(),
        })
    }

    /// Prints the application help page with grouped commands.
    pub fn print_help(&self) {
        println!();
        println!("{} {}", self.name.bold().cyan(), self.version.dimmed());
        println!("{}", self.description);
        println!();

        println!("{}", "USAGE:".bold().yellow());
        println!("  {} [command] [options] [arguments]", self.name.cyan());
        println!();

        println!("{}", "GLOBAL OPTIONS:".bold().yellow());
        println!("  {:20} Show this help overview", "-h, --help".cyan());
        println!("  {:20} Print application version", "-V, --version".cyan());
        println!();

        println!("{}", "AVAILABLE COMMANDS:".bold().yellow());

        // Group commands
        let mut root_commands = Vec::new();
        let mut grouped_commands: HashMap<String, Vec<&dyn Command>> = HashMap::new();

        for cmd in self.commands.values() {
            let name = cmd.name();
            if let Some((prefix, _)) = name.split_once(':') {
                grouped_commands
                    .entry(prefix.to_string())
                    .or_default()
                    .push(cmd.as_ref());
            } else {
                root_commands.push(cmd.as_ref());
            }
        }

        root_commands.sort_by_key(|c| c.name());

        for cmd in root_commands {
            println!("  {:22} {}", cmd.name().bold().white(), cmd.description());
        }

        let mut sorted_groups: Vec<_> = grouped_commands.into_iter().collect();
        sorted_groups.sort_by(|a, b| a.0.cmp(&b.0));

        for (group, mut cmds) in sorted_groups {
            println!();
            let label = self.groups.get(&group).cloned().unwrap_or(group);
            println!(" {}:", label.bold().yellow());
            cmds.sort_by_key(|c| c.name());
            for cmd in cmds {
                println!("  {:22} {}", cmd.name().bold().white(), cmd.description());
            }
        }

        println!();
    }

    /// Prints detailed usage information for a specific command.
    pub fn print_command_help(&self, command: &dyn Command) {
        println!();
        println!(
            "{}: {}",
            command.name().bold().cyan(),
            command.description()
        );
        println!();

        let mut usage = format!("  {} {}", self.name, command.name());
        for arg in command.arguments() {
            if arg.required {
                usage.push_str(&format!(" <{}>", arg.name));
            } else {
                usage.push_str(&format!(" [{}]", arg.name));
            }
        }
        if !command.flags().is_empty() {
            usage.push_str(" [options]");
        }

        println!("{}", "USAGE:".bold().yellow());
        println!("{usage}");
        println!();

        if !command.arguments().is_empty() {
            println!("{}", "ARGUMENTS:".bold().yellow());
            for arg in command.arguments() {
                let req = if arg.required { "required" } else { "optional" };
                println!(
                    "  {:20} {} ({})",
                    arg.name.cyan(),
                    arg.description,
                    req.dimmed()
                );
            }
            println!();
        }

        if !command.flags().is_empty() {
            println!("{}", "OPTIONS:".bold().yellow());
            for flag in command.flags() {
                let short_part = flag.short.map_or(String::new(), |s| format!("-{s}, "));
                let flag_label = format!("{short_part}--{}", flag.name);
                println!("  {:20} {}", flag_label.cyan(), flag.description);
            }
            println!();
        }
    }
}
