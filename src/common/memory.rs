use crate::common::utils::ClientType;
use crate::common::utils::Communication;
use anyhow::Result;
use pinecone_sdk::models::{Kind, Value, Vector};
use pinecone_sdk::pinecone::PineconeClientConfig;
use std::borrow::Cow;
use std::collections::BTreeMap;
use tracing::error;

async fn embed_text(client: &mut ClientType, content: Cow<'static, str>) -> Vec<f64> {
    match client {
        #[cfg(feature = "gem")]
        ClientType::Gemini(gem_client) => {
            use gems::embed::EmbeddingBuilder;
            use gems::messages::Content;
            use gems::messages::Message;
            use gems::models::Model;
            use gems::traits::CTrait;

            let params = EmbeddingBuilder::default()
                .model(Model::Embedding)
                .input(Message::User {
                    content: Content::Text(content.into()),
                    name: None,
                })
                .build()
                .unwrap_or_default();
            gem_client.set_model(Model::Embedding);
            let response = gem_client.embeddings().create(params).await;
            gem_client.set_model(Model::Flash20);
            match response {
                Ok(embed_response) => {
                    if let Some(embedding) = embed_response.embedding {
                        embedding.values
                    } else {
                        error!("Gemini: No embedding returned.");
                        vec![]
                    }
                }
                Err(err) => {
                    error!("Gemini: Failed to embed content: {}", err);
                    vec![]
                }
            }
        }
        #[cfg(feature = "oai")]
        ClientType::OpenAI(oai_client) => {
            use openai_dive::v1::models::EmbeddingModel;
            use openai_dive::v1::resources::embedding::{
                EmbeddingEncodingFormat, EmbeddingInput, EmbeddingOutput,
                EmbeddingParametersBuilder,
            };

            let parameters = EmbeddingParametersBuilder::default()
                .model(EmbeddingModel::TextEmbedding3Small.to_string())
                .input(EmbeddingInput::String(content.to_string()))
                .encoding_format(EmbeddingEncodingFormat::Float)
                .build()
                .unwrap();

            match oai_client.embeddings().create(parameters).await {
                Ok(response) => {
                    if let Some(embedding) = response.data.first() {
                        match &embedding.embedding {
                            EmbeddingOutput::Float(vec) => vec.clone(),
                            EmbeddingOutput::Base64(_) => {
                                error!("OpenAI: Expected embedding as Float, found Base64.");
                                vec![]
                            }
                        }
                    } else {
                        error!("OpenAI: No embedding returned.");
                        vec![]
                    }
                }
                Err(err) => {
                    error!("OpenAI: Failed to embed content: {}", err);
                    vec![]
                }
            }
        }

        #[allow(unreachable_patterns)]
        _ => {
            error!("Unsupported AI client for embedding.");
            vec![]
        }
    }
}

pub async fn save_long_term_memory(
    client: &mut ClientType,
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
    let values_f32: Vec<f32> = embed_text(client, communication.content.clone())
        .await
        .into_iter()
        .map(|v| v as f32)
        .collect();

    let padding: Vec<f32> = vec![0.0; 1024 - values_f32.len()];
    let padded_values: Vec<f32> = values_f32.into_iter().chain(padding).collect();

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
