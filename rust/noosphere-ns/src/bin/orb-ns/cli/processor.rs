use crate::cli::{CLICommand, CLIPeers, CLIRecords, CLI};
use crate::runner::{run, RunnerConfig};
use crate::utils;
use anyhow::Result;
use clap::Parser;
use noosphere::key::{InsecureKeyStorage, KeyStorage};
use noosphere_ns::server::HTTPClient;
use noosphere_ns::NameSystemClient;
use serde::Serialize;
use tracing::*;

fn jsonify<T: Serialize>(value: T) -> Result<String> {
    serde_json::to_string(&Ok::<T, T>(value)).map_err(|e| e.into())
}

pub async fn process_args(key_storage: &InsecureKeyStorage) -> Result<(), String> {
    if let Some(json_str) = process_command(CLI::parse().command, key_storage).await? {
        println!("{}", json_str);
    }
    Ok(())
}

pub async fn process_command(
    command: CLICommand,
    key_storage: &InsecureKeyStorage,
) -> Result<Option<String>, String> {
    async fn process_command_inner(
        command: CLICommand,
        key_storage: &InsecureKeyStorage,
    ) -> Result<Option<String>> {
        match command {
            command @ CLICommand::Run { .. } => {
                let config = RunnerConfig::try_from_command(command, &key_storage).await?;
                run(config).await?;
                Ok(None)
            }
            CLICommand::KeyGen { key } => {
                if key_storage.require_key(&key).await.is_ok() {
                    info!("Key \"{}\" already exists in `~/.noosphere/keys/`.", &key);
                } else {
                    key_storage.create_key(&key).await?;
                    info!("Key \"{}\" created in `~/.noosphere/keys/`.", &key);
                }
                Ok(None)
            }
            CLICommand::Status { api_url } => {
                let client = HTTPClient::new(api_url).await?;
                let info = client.network_info().await?;
                Ok(Some(jsonify(info)?))
            }
            CLICommand::Records(CLIRecords::Get { identity, api_url }) => {
                let client = HTTPClient::new(api_url).await?;
                let maybe_record = client.get_record(&identity).await?;
                Ok(Some(jsonify(maybe_record)?))
            }
            CLICommand::Records(CLIRecords::Put { record, api_url }) => {
                let client = HTTPClient::new(api_url).await?;
                client.put_record(record).await?;
                Ok(Some(jsonify(())?))
            }
            CLICommand::Peers(CLIPeers::Ls { api_url }) => {
                let client = HTTPClient::new(api_url).await?;
                let peers = client.peers().await?;
                Ok(Some(jsonify(peers)?))
            }
            CLICommand::Peers(CLIPeers::Add { peer, api_url }) => {
                let client = HTTPClient::new(api_url).await?;
                client.add_peers(vec![peer]).await?;
                Ok(Some(jsonify(())?))
            }
        }
    }

    process_command_inner(command, key_storage)
        .await
        .map_err(|e| serde_json::to_string(&Err::<String, String>(e.to_string())).unwrap())
}
