//! # `DesignerGPT` agent.
//!
//! This module provides functionality for creating innovative website designs
//! and user experiences based on prompts using Gemini API. The `DesignerGPT` agent
//! understands user requirements and generates wireframes and user interfaces (UIs)
//! for web applications using the GetIMG API.
//!
//! # Example - Generating website designs:
//!
//! ```rust
//! use autogpt::agents::designer::DesignerGPT;
//! use autogpt::common::utils::Tasks;
//! use autogpt::traits::functions::Functions;
//! use autogpt::traits::functions::AsyncFunctions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut designer_agent = DesignerGPT::new(
//!         "Create innovative website designs",
//!         "UIs",
//!     ).await;
//!
//!     let mut tasks = Tasks {
//!         description: "Design a modern and minimalist homepage design layout for a tech company".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = designer_agent.execute(&mut tasks, true, false, 3).await {
//!         eprintln!("Error executing designer tasks: {:?}", err);
//!     }
//! }
//! ```
//!

use crate::agents::agent::AgentGPT;
use crate::common::utils::{ClientType, Communication, Status, Tasks, similarity};
use crate::prompts::designer::{IMGGET_PROMPT, WEB_DESIGNER_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Functions};
use anyhow::Result;
use colored::*;
use getimg::client::Client as ImgClient;
use getimg::utils::save_image;
use std::borrow::Cow;
use std::env::var;
use tokio::fs;
use tracing::{debug, error, info};

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "oai")]
use {openai_dive::v1::models::FlagshipModel, openai_dive::v1::resources::chat::*};

#[cfg(feature = "gem")]
use gems::{
    messages::Content, messages::Message, traits::CTrait, utils::load_and_encode_image,
    vision::VisionBuilder,
};

use async_trait::async_trait;
#[cfg(feature = "xai")]
use x_ai::{
    chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
    traits::ChatCompletionsFetcher,
};

/// Struct representing a DesignerGPT, which manages design-related tasks using Gemini or OpenAI API.
#[derive(Debug, Clone)]
pub struct DesignerGPT {
    /// Represents the workspace directory path for DesignerGPT.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling design tasks.
    agent: AgentGPT,
    /// Represents a GetIMG client for generating images from text prompts.
    img_client: ImgClient,
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
}

impl DesignerGPT {
    /// Constructor function to create a new instance of `DesignerGPT`.
    ///
    /// # Arguments
    ///
    /// * `objective` - Objective description for `DesignerGPT`.
    /// * `position` - Position description for `DesignerGPT`.
    ///
    /// # Returns
    ///
    /// (`DesignerGPT`): A new instance of `DesignerGPT`.
    ///
    /// # Business Logic
    ///
    /// - Constructs the workspace directory path for `DesignerGPT`.
    /// - Initializes the GPT agent with the given objective and position.
    /// - Creates clients for generating images and interacting with Gemini or OpenAI API.
    #[allow(unreachable_code)]
    pub async fn new(objective: &'static str, position: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "designer";

        if !fs::try_exists(&workspace).await.unwrap_or(false) {
            match fs::create_dir_all(&workspace).await {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Workspace directory '{}' already exists.", workspace);
        }

        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);
        let getimg_api_key = var("GETIMG_API_KEY").unwrap_or_default().to_owned();
        let getimg_model = var("GETIMG__MODEL")
            .unwrap_or("lcm-realistic-vision-v5-1".to_string())
            .to_owned();

        let img_client = ImgClient::new(&getimg_api_key, &getimg_model);

        let client = ClientType::from_env();

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.position(),)
                .bright_white()
                .bold()
        );

        Self {
            workspace: workspace.into(),
            agent,
            img_client,
            client,
        }
    }
    /// Asynchronously generates an image from a text prompt.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A reference to tasks containing the description for image generation.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of image generation.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in generating the image.
    ///
    /// # Business Logic
    ///
    /// - Constructs a text prompt based on the description from tasks.
    /// - Adds communication logs to the agent memory for traceability.
    /// - Generates an image from the text prompt using the getimg client.
    /// - Saves the generated image to the workspace directory.
    pub async fn generate_image_from_text(&mut self, tasks: &Tasks) -> Result<()> {
        let img_path = self.workspace.to_string() + "/img.jpg";

        let text_prompt: String =
            format!("{}\n\nUser Prompt: {}", IMGGET_PROMPT, tasks.description);
        let negative_prompt = Some("Disfigured, cartoon, blurry");

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: tasks.description.clone(),
        });
        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: tasks.description.clone(),
                })
                .await;
        }

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!("Generating image with prompt: '{}'", text_prompt)),
        });
        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!("Generating image with prompt: '{}'", text_prompt)),
                })
                .await;
        }

        let text_response = self
            .img_client
            .generate_image_from_text(
                &text_prompt,
                1024,
                1024,
                5,
                "jpeg",
                negative_prompt,
                Some(512),
            )
            .await?;

        save_image(&text_response.image, &img_path).unwrap();

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("system"),
            content: Cow::Owned(format!("Image saved at {}", img_path)),
        });
        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("system"),
                    content: Cow::Owned(format!("Image saved at {}", img_path)),
                })
                .await;
        }

        info!(
            "[*] {:?}: Image saved at {}",
            self.agent.position(),
            img_path
        );

        Ok(())
    }

    /// Asynchronously generates text from an image.
    ///
    /// # Arguments
    ///
    /// * `image_path` - Path to the image file for text generation.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the generated text from the image.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in generating text from the image.
    ///
    /// # Business Logic
    ///
    /// - Loads and encodes the image from the specified file path.
    /// - Logs communication between the user, assistant, and system.
    /// - Sends the image data to the Gemini or OpenAI API to generate text.
    /// - Returns the generated text description of the image.
    pub async fn generate_text_from_image(&mut self, image_path: &str) -> Result<String> {
        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Requesting text generation from image at path: {}",
                image_path
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Requesting text generation from image at path: {}",
                        image_path
                    )),
                })
                .await;
        }

        let base64_image_data = match load_and_encode_image(image_path) {
            Ok(data) => data,
            Err(_) => {
                let error_msg = format!("Failed to load or encode image at path: {}", image_path);

                self.agent.add_communication(Communication {
                    role: Cow::Borrowed("system"),
                    content: Cow::Owned(error_msg.clone()),
                });

                #[cfg(feature = "mem")]
                {
                    let _ = self
                        .save_ltm(Communication {
                            role: Cow::Borrowed("system"),
                            content: Cow::Owned(error_msg.clone()),
                        })
                        .await;
                }
                debug!("[*] {:?}: Error loading image!", self.agent.position());
                return Ok("".to_string());
            }
        };

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned("Generating description from uploaded image...".to_string()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Generating description from uploaded image...".to_string(),
                    ),
                })
                .await;
        }
        let response: String = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(gem_client) => {
                let params = VisionBuilder::default()
                    .input(Message::User {
                        content: Content::Text(WEB_DESIGNER_PROMPT.to_string()),
                        name: None,
                    })
                    .image(Message::Tool {
                        content: base64_image_data,
                    })
                    .build()?;

                let result = gem_client.vision().generate(params).await;

                match result {
                    Ok(response) => {
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(format!(
                                "Generated image description: {}",
                                response
                            )),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(format!(
                                        "Generated image description: {}",
                                        response
                                    )),
                                })
                                .await;
                        }

                        debug!(
                            "[*] {:?}: Got Image Description: {:?}",
                            self.agent.position(),
                            response
                        );

                        response
                    }

                    Err(err) => {
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(format!(
                                "Error generating image description: {}",
                                err
                            )),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(format!(
                                        "Error generating image description: {}",
                                        err
                                    )),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }

            #[cfg(feature = "oai")]
            ClientType::OpenAI(oai_client) => {
                let parameters = ChatCompletionParametersBuilder::default()
                    .model(FlagshipModel::Gpt4O.to_string())
                    .messages(vec![
                        ChatMessage::User {
                            content: ChatMessageContent::Text("What is in this image?".to_string()),
                            name: None,
                        },
                        ChatMessage::User {
                            content: ChatMessageContent::ContentPart(vec![
                                ChatMessageContentPart::Image(ChatMessageImageContentPart {
                                    r#type: "image_url".to_string(),
                                    image_url: ImageUrlType {
                                        url: base64_image_data.to_string(),
                                        detail: None,
                                    },
                                }),
                            ]),
                            name: None,
                        },
                    ])
                    .build()?;

                let result = oai_client.chat().create(parameters).await;

                match result {
                    Ok(chat_response) => {
                        let message = &chat_response.choices[0].message;

                        let response_text = match message {
                            ChatMessage::Assistant {
                                content: Some(chat_content),
                                ..
                            } => chat_content.to_string(),
                            ChatMessage::User { content, .. } => content.to_string(),
                            ChatMessage::System { content, .. } => content.to_string(),
                            ChatMessage::Developer { content, .. } => content.to_string(),
                            ChatMessage::Tool { content, .. } => content.clone(),
                            _ => String::from(""),
                        };

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(format!(
                                "Generated image description: {}",
                                response_text
                            )),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(format!(
                                        "Generated image description: {}",
                                        response_text
                                    )),
                                })
                                .await;
                        }

                        debug!(
                            "[*] {:?}: Got Image Description: {:?}",
                            self.agent.position(),
                            response_text
                        );

                        response_text
                    }

                    Err(err) => {
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(format!(
                                "Error generating image description: {}",
                                err
                            )),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(format!(
                                        "Error generating image description: {}",
                                        err
                                    )),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }

            #[cfg(feature = "xai")]
            ClientType::Xai(xai_client) => {
                let messages = vec![XaiMessage {
                    role: "user".into(),
                    content: "What is in this image?".to_string(),
                }];

                let rb = ChatCompletionsRequestBuilder::new(
                    xai_client.clone(),
                    "grok-beta".into(),
                    messages,
                )
                .temperature(0.0)
                .stream(false);

                let req = rb.clone().build()?;
                let resp = rb.create_chat_completion(req).await;

                match resp {
                    Ok(chat) => {
                        let response_text = chat.choices[0].message.content.clone();

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(
                                "Generated image description: ".to_string() + &response_text,
                            ),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(
                                        "Generated image description: ".to_string()
                                            + &response_text,
                                    ),
                                })
                                .await;
                        }

                        #[cfg(debug_assertions)]
                        debug!(
                            "[*] {:?}: Got XAI Output: {:?}",
                            self.agent.position(),
                            response_text
                        );

                        response_text
                    }

                    Err(err) => {
                        let err_msg = format!("ERROR_MESSAGE: {}", err);

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(err_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(err_msg.clone()),
                                })
                                .await;
                        }

                        return Err(anyhow::anyhow!(err_msg));
                    }
                }
            }

            #[allow(unreachable_patterns)]
            _ => {
                return Err(anyhow::anyhow!(
                    "No valid AI client configured. Enable `gem`, `oai`, `cld`, or `xai` feature."
                ));
            }
        };

        Ok(response)
    }

    /// Compares text prompts to determine similarity.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks containing the GetIMG AI prompt.
    /// * `generated_text` - The generated text to compare with the GetIMG AI prompt.
    ///
    /// # Returns
    ///
    /// (`Result<bool>`): Result indicating whether the generated text is similar to the prompt.
    ///
    /// # Business Logic
    ///
    /// - Compares the generated text with the GetIMG AI prompt.
    /// - Returns true if similarity meets a predefined threshold, otherwise false.
    ///
    pub async fn compare_text_and_image_prompts(
        &mut self,
        tasks: &mut Tasks,
        generated_text: &str,
    ) -> Result<bool> {
        let getimg_prompt = &tasks.description;

        let similarity_threshold = 0.8;
        let similarity = similarity(getimg_prompt, generated_text);

        if similarity >= similarity_threshold {
            return Ok(true);
        }

        Ok(false)
    }
}

impl Functions for DesignerGPT {
    /// Retrieves a reference to the agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the agent.
    ///
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }
}

/// Implementation of the trait `AsyncFunctions` for `DesignerGPT`.
/// Contains additional methods related to design tasks.
///
/// This trait provides methods for:
///
/// - Retrieving the agent associated with `DesignerGPT`.
/// - Executing tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the agent associated with the `DesignerGPT` instance.
/// - Executes tasks asynchronously based on the current status of the agent.
/// - Handles task execution including image generation, text generation, and comparison.
/// - Manages retries and error handling during task execution.
#[async_trait]
impl AsyncFunctions for DesignerGPT {
    /// Asynchronously executes tasks associated with DesignerGPT.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be executed.
    /// * `execute` - A boolean indicating whether to execute the tasks (TODO).
    /// * `max_tries` - Maximum number of attempts to execute tasks (TODO).
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of task execution.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in executing tasks.
    ///
    /// # Business Logic
    ///
    /// - Executes tasks asynchronously based on the current status of the agent.
    /// - Handles task execution including image generation, text generation, and comparison.
    /// - Manages retries and error handling during task execution.
    ///
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Tasks,
        _execute: bool,
        _browse: bool,
        _max_tries: u64,
    ) -> Result<()> {
        info!(
            "{}",
            format!("[*] {:?}: Executing task:", self.agent.position(),)
                .bright_white()
                .bold()
        );
        for task in tasks.clone().description.clone().split("- ") {
            if !task.trim().is_empty() {
                info!("{} {}", "•".bright_white().bold(), task.trim().cyan());
            }
        }

        let mut _count = 0;
        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::Idle => {
                    debug!("[*] {:?}: Idle", self.agent.position());

                    self.generate_image_from_text(tasks).await?;
                    // let generated_text = self.generate_text_from_image("./img.jpg").await?;

                    // let text_similarity = self
                    //     .compare_text_and_image_prompts(tasks, &generated_text)
                    //     .await?;
                    // debug!(
                    //     "[*] {:?}: Idle: {}",
                    //     self.agent.position(),
                    //     text_similarity
                    // );

                    // if text_similarity || count == max_tries {
                    //     self.agent.update(Status::Completed);
                    // }
                    _count += 1;
                    self.agent.update(Status::Completed);
                }
                _ => {
                    self.agent.update(Status::Completed);
                }
            }
        }

        Ok(())
    }
    /// Saves a communication to long-term memory for the agent.
    ///
    /// # Arguments
    ///
    /// * `communication` - The communication to save, which contains the role and content.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating the success or failure of saving the communication.
    ///
    /// # Business Logic
    ///
    /// - This method uses the `save_long_term_memory` util function to save the communication into the agent's long-term memory.
    /// - The communication is embedded and stored using the agent's unique ID as the namespace.
    /// - It handles the embedding and metadata for the communication, ensuring it's stored correctly.
    #[cfg(feature = "mem")]
    async fn save_ltm(&mut self, communication: Communication) -> Result<()> {
        save_long_term_memory(&mut self.client, self.agent.id.clone(), communication).await
    }

    /// Retrieves all communications stored in the agent's long-term memory.
    ///
    /// # Returns
    ///
    /// (`Result<Vec<Communication>>`): A result containing a vector of communications retrieved from the agent's long-term memory.
    ///
    /// # Business Logic
    ///
    /// - This method fetches the stored communications for the agent by interacting with the `load_long_term_memory` function.
    /// - The function will return a list of communications that are indexed by the agent's unique ID.
    /// - It handles the retrieval of the stored metadata and content for each communication.
    #[cfg(feature = "mem")]
    async fn get_ltm(&self) -> Result<Vec<Communication>> {
        load_long_term_memory(self.agent.id.clone()).await
    }

    /// Retrieves the concatenated context of all communications in the agent's long-term memory.
    ///
    /// # Returns
    ///
    /// (`String`): A string containing the concatenated role and content of all communications stored in the agent's long-term memory.
    ///
    /// # Business Logic
    ///
    /// - This method calls the `long_term_memory_context` function to generate a string representation of the agent's entire long-term memory.
    /// - The context string is composed of each communication's role and content, joined by new lines.
    /// - It provides a quick overview of the agent's memory in a human-readable format.
    #[cfg(feature = "mem")]
    async fn ltm_context(&self) -> String {
        long_term_memory_context(self.agent.id.clone()).await
    }
}

// TODO: Impl Default to make it compatible with AutoGPT
// impl Agent for DesignerGPT {
//     fn new(_objective: Cow<'static, str>, _position: Cow<'static, str>) -> Self {
//         Default::default()
//     }

//     fn update(&mut self, status: Status) {
//         self.agent.update(status);
//     }

//     fn objective(&self) -> &Cow<'static, str> {
//         &self.agent.objective
//     }

//     fn position(&self) -> &Cow<'static, str> {
//         &self.agent.position
//     }

//     fn status(&self) -> &Status {
//         &self.agent.status
//     }

//     fn memory(&self) -> &Vec<Communication> {
//         &self.agent.memory
//     }
// }
