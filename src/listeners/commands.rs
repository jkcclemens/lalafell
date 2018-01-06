use commands::*;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;

use std::collections::HashMap;

#[derive(Default)]
pub struct CommandListener<'a> {
  commands: HashMap<Vec<String>, Box<Command<'a> + Send + Sync>>
}

impl<'a> CommandListener<'a> {
  pub fn add_command<T: AsRef<str>>(&mut self, names: &[T], command: Box<Command<'a> + Send + Sync>) {
    self.commands.insert(names.iter().map(|t| t.as_ref().to_string()).collect(), command);
  }
}

impl<'a> EventHandler for CommandListener<'a> {
  fn message(&self, context: Context, message: Message) {
    let parts: Vec<&str> = message.content.split_whitespace().collect();
    if parts.is_empty() {
      return;
    }
    let first = parts[0];
    if !first.starts_with('!') {
      return;
    }
    let command_name = first[1..].to_lowercase();
    let params = &parts[1..];
    let (_, command) = match self.commands.iter().find(|&(names, _)| names.contains(&command_name)) {
      Some(c) => c,
      None => return
    };
    debug!("running command: {}", command_name);
    let run_result = command.run(&context, &message, params);
    match run_result {
      Ok(info) => match info.message {
        Some(embed) => { message.channel_id.send_message(|c| c.embed(|e| embed(e).color(0x196358))).ok(); },
        None => { message.react("\u{2705}").ok(); }
      },
      Err(CommandFailure::Internal(info)) => {
        message.channel_id.send_message(|c| c.embed(|e| e.description("An internal error happened while processing this command."))).ok();
        for err in info.error.iter() {
          error!("error: {:#?}", err);
        }
      },
      Err(CommandFailure::External(info)) => match info.message {
        Some(embed) => { message.channel_id.send_message(|c| c.embed(|e| embed(e).color(0x63191b))).ok(); },
        None => { message.react("\u{274c}").ok(); }
      }
    }
  }
}
