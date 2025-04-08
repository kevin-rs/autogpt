use crate::common::utils::Communication;
use anyhow::Result;
use gems::Client;
use pinecone_sdk::models::{Kind, Value, Vector};
use pinecone_sdk::pinecone::PineconeClientConfig;
use std::borrow::Cow;
use std::collections::BTreeMap;
use tracing::error;

async fn embed_gemini_text(client: &mut Client, content: Cow<'static, str>) -> Vec<f64> {
    client.model = "embedding-001".into();
    match client.embed_content(&content).await {
        Ok(embed_response) => {
            if let Some(embedding) = embed_response.embedding {
                embedding.values
            } else {
                error!("No embedding returned.");
                vec![]
            }
        }
        Err(err) => {
            error!("Failed to embed content: {}", err);
            vec![]
        }
    }
}

pub async fn save_long_term_memory(
    client: &mut Client,
    agent_id: Cow<'static, str>,
    communication: Communication,
) -> Result<()> {
    let config = PineconeClientConfig {
        api_key: Some(std::env::var("PINECONE_API_KEY").unwrap_or_default()),
        ..Default::default()
    };

    let pinecone_result = config.client();
    let pinecone = match pinecone_result {
        Ok(client) => client,
        Err(e) => {
            error!("Error creating Pinecone client: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create Pinecone client",
            )
            .into());
        }
    };

    let index_result = pinecone
        .index(&std::env::var("PINECONE_INDEX_URL").unwrap_or_default())
        .await;
    let mut index = match index_result {
        Ok(index) => index,
        Err(e) => {
            error!("Error connecting to Pinecone index: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to Pinecone index",
            )
            .into());
        }
    };

    let namespace = format!("agent-{}", agent_id);
    let values_f32: Vec<f32> = embed_gemini_text(client, communication.content.clone())
        .await
        .into_iter()
        .map(|v| v as f32)
        .collect();

    let padding: Vec<f32> = vec![0.0; 1024 - values_f32.len()];
    let padded_values: Vec<f32> = values_f32.iter().cloned().chain(padding).collect();

    let content = communication.content.clone();
    let role = communication.role.clone();

    let vector = Vector {
        id: uuid::Uuid::new_v4().to_string(),
        values: padded_values,
        sparse_values: None,
        metadata: Some(pinecone_sdk::models::Metadata {
            fields: BTreeMap::from([
                (
                    "role".to_string(),
                    Value {
                        kind: Some(Kind::StringValue(role.to_string())),
                    },
                ),
                (
                    "content".to_string(),
                    Value {
                        kind: Some(Kind::StringValue(content.to_string())),
                    },
                ),
            ]),
        }),
    };

    index.upsert(&[vector], &namespace.into()).await.unwrap();
    Ok(())
}

pub async fn load_long_term_memory(agent_id: Cow<'static, str>) -> Result<Vec<Communication>> {
    let config = PineconeClientConfig {
        api_key: Some(std::env::var("PINECONE_API_KEY").unwrap()),
        ..Default::default()
    };

    let pinecone_result = config.client();
    let pinecone = match pinecone_result {
        Ok(client) => client,
        Err(e) => {
            error!("Error creating Pinecone client: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create Pinecone client",
            )
            .into());
        }
    };

    let index_result = pinecone
        .index(&std::env::var("PINECONE_INDEX_URL").unwrap_or_default())
        .await;
    let mut index = match index_result {
        Ok(index) => index,
        Err(e) => {
            error!("Error connecting to Pinecone index: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to Pinecone index",
            )
            .into());
        }
    };

    let namespace = format!("agent-{}", agent_id);
    let list = index
        .list(&namespace.clone().into(), None, None, None)
        .await
        .unwrap();

    let ids: Vec<&str> = list.vectors.iter().map(|v| v.id.as_str()).collect();
    let fetched = index.fetch(&ids, &namespace.into()).await.unwrap();

    let communications = fetched
        .vectors
        .values()
        .map(|v| {
            let metadata = v.metadata.as_ref().unwrap();

            Communication {
                role: match metadata.fields.get("role").and_then(|v| v.kind.as_ref()) {
                    Some(Kind::StringValue(val)) => Cow::Owned(val.clone()),
                    _ => Cow::Borrowed("unknown"),
                },
                content: match metadata.fields.get("content").and_then(|v| v.kind.as_ref()) {
                    Some(Kind::StringValue(val)) => Cow::Owned(val.clone()),
                    _ => Cow::Borrowed(""),
                },
            }
        })
        .collect();

    Ok(communications)
}

pub async fn long_term_memory_context(agent_id: Cow<'static, str>) -> String {
    match load_long_term_memory(agent_id).await {
        Ok(comms) => comms
            .iter()
            .map(|c| format!("{}: {}", c.role, c.content))
            .collect::<Vec<_>>()
            .join("\n"),
        Err(_) => String::from(""),
    }
}
