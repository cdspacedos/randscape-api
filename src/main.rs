use std::path::PathBuf;

use dotenvy::dotenv;
mod landscape_api;
use landscape_api::*;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "randscape-register",
    about = "The landscape-api command that actually works"
)]
struct CreateScriptAttachment {
    #[structopt(help = "Upload the attachment to the script")]
    title: String,
    script_name: PathBuf,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "randscape-register",
    about = "The landscape-api command that actually works"
)]
enum Command {
    #[structopt(about = "Get script details")]
    GetScript {
        #[structopt(help = "Specify the script name")]
        title: String,
    },
    #[structopt(about = "List all scripts")]
    GetScripts {},
    #[structopt(about = "Get script details")]
    RemoveScriptAttachment {
        #[structopt(help = "Remove script attachment if found")]
        title: String,
        script_name: PathBuf,
    },
    CreateScriptAttachment(CreateScriptAttachment),
    #[structopt(about = "Check the existing attachments")]
    GetScriptAttachments {
        #[structopt(help = "List all the attachment names for given script")]
        title: String,
    },
    #[structopt(about = "Execute the script over the hosts")]
    ExecuteScript {
        #[structopt(help = "Script name")]
        title: String,
        #[structopt(help = "Query to identify the Landscape hosts")]
        query: String,
    },
    #[structopt(about = "Get information about all registered hosts")]
    GetAllHosts,
}

fn main() {
    dotenv().ok();
    let _api = Api::new();
    let opt = Command::from_args();

    match opt {
        Command::GetScript { title } => println!("{:#?}", _api.get_script(&title).unwrap()),
        Command::GetScripts {} => println!("{:#?}", _api.get_scripts()),
        Command::RemoveScriptAttachment { title, script_name } => {
            println!("{}", _api.remove_script_attachment(&title, script_name))
        }
        Command::CreateScriptAttachment(CreateScriptAttachment { title, script_name }) => {
            println!("{}", _api.create_script_attachment(&title, &script_name))
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
            println!("{:#?}", _api.get_all_hosts())
        }
    }
}
