## Landscape API client made in Rust

An automatic update to the new version of the 'landscape-api' snap by Canonical introduced a bug that led to application crashes and disrupted my automation workflow.

To circumvent this issue, I created a small, purpose-built utility tailored to manage Landscape Script Attachments with efficiency.

Key advantages of this application include:

- Pre-built API authentication capabilities, saving you initial setup time
- Easily extendable features to meet evolving needs
- Simplified native OS packaging with cargo-deb and cargo-rpm
- Independence from Python or any other specific OS tools
- Potential to create a static release using 'musl' if required
- Can be cross compiled for Windows and probably macOS

### Building

Clone the repository. Step in to the `randscape-api` folder, and run `cargo build`.

### Usage

Setup the environment in the same way you would do for the `landscape-api` made by Canonical. You can use the template file `load-env.sh` for details.
Load the environment variables:

```bash
source load-env.sh
```

Run the command:

```bash
./target/debug/randscape-api 
randscape-register 0.3.0
The landscape-api command that actually works

USAGE:
    randscape-api <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    create-script-attachment    The landscape-api command that actually works
    execute-script              Execute the script over the hosts
    get-all-hosts               Get information about all registered hosts
    get-script                  Get script details
    get-script-attachments      Check the existing attachments
    get-scripts                 List all scripts
    help                        Prints this message or the help of the given subcommand(s)
    remove-script-attachment    Get script details
```
