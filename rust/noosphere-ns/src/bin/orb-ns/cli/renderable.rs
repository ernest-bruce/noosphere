use crate::cli::{CLICommand, CLIPeers, CLIRecords, CLI};
use crate::runner::{run, RunnerConfig};
use crate::utils;
use anyhow::{anyhow, Result};
use clap::Parser;
use noosphere::key::{InsecureKeyStorage, KeyStorage};
use noosphere_ns::server::HTTPClient;
use noosphere_ns::NameSystemClient;
use serde::Serialize;
use serde_json;

pub trait Renderable {
    fn render_json(&self) -> Result<String>;
}

impl<T: Serialize> Renderable for T {
    fn render_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}

/*
pub async fn process_command(command: CLICommand, key_storage: &InsecureKeyStorage) -> Result<()> {
    match command {
        command @ CLICommand::Run { .. } => {
            let config = RunnerConfig::try_from_command(command, &key_storage).await?;
            utils::run_until_abort(async move { run(config).await }).await?;
            Ok(())
        }
        CLICommand::KeyGen { key } => {
            if key_storage.require_key(&key).await.is_ok() {
                println!("Key \"{}\" already exists in `~/.noosphere/keys/`.", &key);
            } else {
                key_storage.create_key(&key).await?;
                println!("Key \"{}\" created in `~/.noosphere/keys/`.", &key);
            }
            Ok(())
        }
        CLICommand::Status { api_url } => {
            let client = HTTPClient::new(api_url).await?;
            let info = client.network_info().await?;
            println!("{:#?}", info);
            Ok(())
        }
        CLICommand::Records(CLIRecords::Get { identity, api_url }) => {
            let client = HTTPClient::new(api_url).await?;
            let maybe_record = client.get_record(&identity).await?;
            if let Some(record) = maybe_record {
                println!("{}", record.try_to_string()?);
            } else {
                println!("No record found.");
            }
            Ok(())
        }
        CLICommand::Records(CLIRecords::Put { record, api_url }) => {
            let client = HTTPClient::new(api_url).await?;
            client.put_record(record).await?;
            println!("success");
            Ok(())
        }
        CLICommand::Peers(CLIPeers::Ls { api_url }) => {
            let client = HTTPClient::new(api_url).await?;
            let peers = client.peers().await?;
            println!("{:#?}", peers);
            Ok(())
        }
        CLICommand::Peers(CLIPeers::Add { peer, api_url }) => {
            let client = HTTPClient::new(api_url).await?;
            client.add_peers(vec![peer]).await?;
            println!("success");
            Ok(())
        }
    }
}
*/
