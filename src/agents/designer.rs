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
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut designer_agent = DesignerGPT::new(
//!         "Create innovative website designs",
//!         "UIs",
//!     );
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
//!     if let Err(err) = designer_agent.execute(&mut tasks, true, 3).await {
//!         eprintln!("Error executing designer tasks: {:?}", err);
//!     }
//! }
//! ```
//!

use crate::agents::agent::AgentGPT;
use crate::common::utils::{similarity, Status, Tasks};
use crate::prompts::designer::{IMGGET_PROMPT, WEB_DESIGNER_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use gems::utils::load_and_encode_image;
use gems::Client;
use getimg::client::Client as ImgClient;
use getimg::utils::save_image;
use std::borrow::Cow;
use std::env::var;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info};

/// Struct representing a DesignerGPT, which manages design-related tasks using Gemini API.
#[derive(Debug, Clone)]
pub struct DesignerGPT {
    /// Represents the workspace directory path for DesignerGPT.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling design tasks.
    agent: AgentGPT,
    /// Represents a GetIMG client for generating images from text prompts.
    img_client: ImgClient,
    /// Represents a Gemini client for interacting with Gemini API.
    client: Client,
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
    /// - Creates clients for generating images and interacting with Gemini API.
    ///
    pub fn new(objective: &'static str, position: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "designer";

        if !Path::new(&workspace).exists() {
            match fs::create_dir_all(workspace.clone()) {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Directory '{}' already exists.", workspace);
        }

        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);
        let getimg_api_key = var("GETIMG_API_KEY").unwrap_or_default().to_owned();
        let getimg_model = var("GETIMG__MODEL")
            .unwrap_or("lcm-realistic-vision-v5-1".to_string())
            .to_owned();

        let img_client = ImgClient::new(&getimg_api_key, &getimg_model);

        let model = var("GEMINI_MODEL")
            .unwrap_or("gemini-pro-vision".to_string())
            .to_owned();
        let api_key = var("GEMINI_API_KEY").unwrap_or_default().to_owned();
        let client = Client::new(&api_key, &model);

        info!("[*] {:?}: {:?}", position, agent);

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
    /// - Generates an image from the text prompt using the getimg client.
    /// - Saves the generated image to the workspace directory.
    ///
    pub async fn generate_image_from_text(&mut self, tasks: &Tasks) -> Result<()> {
        let img_path = self.workspace.to_string() + "/img.jpg";

        let text_prompt: String =
            format!("{}\n\nUser Prompt: {}", IMGGET_PROMPT, tasks.description);
        let negative_prompt = Some("Disfigured, cartoon, blurry");

        // Generate image from text prompt
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

        // Save text response image to file
        save_image(&text_response.image, &img_path).unwrap();

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
    /// - Sends the image data to the Gemini API to generate text.
    /// - Returns the generated text description of the image.
    ///
    pub async fn generate_text_from_image(&mut self, image_path: &str) -> Result<String> {
        let base64_image_data = match load_and_encode_image(image_path) {
            Ok(data) => data,
            Err(_) => {
                debug!("[*] {:?}: Error loading image!", self.agent.position());
                "".to_string()
            }
        };

        let response = self
            .client
            .generate_content_with_image(WEB_DESIGNER_PROMPT, &base64_image_data)
            .await
            .unwrap();

        debug!(
            "[*] {:?}: Got Image Description: {:?}",
            self.agent.position(),
            response
        );

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

/// Implementation of the trait Functions for `DesignerGPT`.
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
///
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
    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool, _max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.position(),
            tasks.clone()
        );
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
}
