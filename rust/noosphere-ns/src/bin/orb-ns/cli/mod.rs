mod address;
mod cli;
mod processor;
mod renderable;

pub use cli::*;

pub use processor::{process_args, process_command};

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;
    use anyhow::Result;
    use cid::Cid;
    use noosphere::key::{InsecureKeyStorage, KeyStorage};
    use noosphere_core::data::Did;
    use noosphere_ns::NSRecord;
    use tempdir::TempDir;
    use tokio;
    use ucan::crypto::KeyMaterial;
    use url::Url;

    use tracing::*;

    // ports 25600
    #[tokio::test]
    async fn it_processes_record_commands() -> Result<()> {
        let temp_dir = TempDir::new("orb-ns-processes-record-commands").unwrap();
        let key_storage = InsecureKeyStorage::new(temp_dir.path())?;
        let key_a = key_storage.create_key("key-a").await?;
        let key_b = key_storage.create_key("key-b").await?;
        let id_a = Did::from(key_a.get_did().await?);
        let id_b = Did::from(key_b.get_did().await?);
        let url_a: Url = "http://127.0.0.1:25601".parse()?;
        let url_b: Url = "http://127.0.0.1:25603".parse()?;

        info!("ONE");
        let ks_1 = key_storage.clone();
        let _handle_a = tokio::spawn(async move {
            process_command(
                CLICommand::Run {
                    config: None,
                    key: Some(String::from("key-a")),
                    listening_address: Some("/ip4/127.0.0.1/tcp/25600".parse().unwrap()),
                    api_address: Some("127.0.0.1:25601".parse().unwrap()),
                    bootstrap: None,
                },
                &ks_1,
            )
            .await
        });
        /*
        info!("TWO");
        let ks_2 = key_storage.clone();
        let _handle_b = tokio::spawn(async move {
            process_command(
                CLICommand::Run {
                    config: None,
                    key: Some(String::from("key-b")),
                    listening_address: Some("/ip4/127.0.0.1/tcp/25602".parse().unwrap()),
                    api_address: Some("127.0.0.1:25603".parse().unwrap()),
                    bootstrap: Some(vec!["/ip4/127.0.0.1/tcp/25600".parse().unwrap()]),
                },
                &ks_2,
            )
            .await
        });
        */
        println!("CLIPeers::Ls");
        println!("URL {}", url_a);
        // Wait until server is up
        while let Err(_e) = process_command(
            CLICommand::Peers(CLIPeers::Ls {
                api_url: url_a.clone(),
            }),
            &key_storage,
        )
        .await
        {
            println!("ERROR {}", _e);
        }
        return Err(anyhow::anyhow!("foo"));

        if let Some(json_str) = process_command(
            CLICommand::Peers(CLIPeers::Ls {
                api_url: url_a.clone(),
            }),
            &key_storage,
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        {
            println!("{}", json_str);
        }

        return Err(anyhow::anyhow!("foo"));

        let link = "bafy2bzacec4p5h37mjk2n6qi6zukwyzkruebvwdzqpdxzutu4sgoiuhqwne72";
        let cid_link: Cid = link.parse()?;
        let record = NSRecord::from_issuer(&key_a, &id_a, &cid_link, None).await?;
        process_command(
            CLICommand::Records(CLIRecords::Put {
                record,
                api_url: url_a,
            }),
            &key_storage,
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

        process_command(
            CLICommand::Records(CLIRecords::Get {
                identity: id_a,
                api_url: url_b,
            }),
            &key_storage,
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
    }
}
