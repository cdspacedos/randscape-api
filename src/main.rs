use std::path::PathBuf;

use dotenvy::dotenv;
mod landscape_api;
use landscape_api::*;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Args)]
struct CreateScriptAttachment {
    #[arg(help = "Upload the attachment to the script")]
    script_title: String,
    attachment_name: PathBuf,
}

#[derive(Debug, Parser)]
#[command(
    name = "randscape-register",
    about = "The landscape-api command that actually works"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Get script details")]
    GetScript {
        #[arg(help = "Specify the script name")]
        title: String,
    },
    #[command(about = "List all scripts")]
    GetScripts {},
    #[command(about = "Get script details")]
    RemoveScriptAttachment {
        #[arg(help = "Remove script attachment if found")]
        title: String,
        script_name: PathBuf,
    },
    CreateScriptAttachment(CreateScriptAttachment),
    #[command(about = "Check the existing attachments")]
    GetScriptAttachments {
        #[arg(help = "List all the attachment names for given script")]
        title: String,
    },
    #[command(about = "Execute the script over the hosts")]
    ExecuteScript {
        #[arg(help = "Script name")]
        title: String,
        #[arg(help = "Query to identify the Landscape hosts")]
        query: String,
    },
    #[command(about = "Get information about all registered hosts")]
    GetAllHosts,
}

fn main() {
    dotenv().ok();
    let _api = Api::new();
    let opt = Cli::parse().command;

    match opt {
        Command::GetScript { title } => {
            if let Some(script) = _api.get_script(&title) {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&script).expect("Failed to serialize")
                )
            } else {
                println!("Script not found")
            }
        }
        Command::GetScripts {} => println!("{:#?}", _api.get_scripts()),
        Command::RemoveScriptAttachment { title, script_name } => {
            println!("{}", _api.remove_script_attachment(&title, script_name))
        }
        Command::CreateScriptAttachment(CreateScriptAttachment {
            script_title,
            attachment_name,
        }) => {
            println!(
                "{}",
                _api.create_script_attachment(&script_title, &attachment_name)
            )
        }
        Command::GetScriptAttachments { title } => _api
            .get_script_attachments(&title)
            .iter()
            .map(|a| println!("{}", a))
            .collect(),
        Command::ExecuteScript { title, query } => {
            println!("{:#?}", _api.execute_script(&query, &title))
        }
        Command::GetAllHosts => {
            println!(
                "{}",
                serde_json::to_string_pretty(&_api.get_all_hosts()).expect("Failed to serialize")
            )
        }
    }
}
