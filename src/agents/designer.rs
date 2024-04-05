use crate::agents::agent::AgentGPT;
use crate::common::utils::{similarity, Status, Tasks};
use crate::prompts::designer::{STABILITY_PROMPT, WEB_DESIGNER_PROMPT};
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

#[derive(Debug, Clone)]
pub struct DesignerGPT {
    workspace: Cow<'static, str>,
    agent: AgentGPT,
    img_client: ImgClient,
    client: Client,
}

impl DesignerGPT {
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

    pub async fn generate_image_from_text(&mut self, tasks: &Tasks) -> Result<()> {
        let img_path = self.workspace.to_string() + "/img.jpg";

        let text_prompt: String =
            format!("{}\n\nUser Prompt: {}", STABILITY_PROMPT, tasks.description);
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

    pub async fn generate_text_from_image(&mut self, image_path: &str) -> Result<String> {
        let base64_image_data = match load_and_encode_image(&image_path) {
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

    pub async fn compare_text_and_image_prompts(
        &mut self,
        tasks: &mut Tasks,
        generated_text: &str,
    ) -> Result<bool> {
        let stability_ai_prompt = &tasks.description;

        let similarity_threshold = 0.2;
        let similarity = similarity(&stability_ai_prompt, &generated_text);

        if similarity >= similarity_threshold {
            return Ok(true);
        }

        Ok(false)
    }
}

impl Functions for DesignerGPT {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool, _max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.position(),
            tasks.clone()
        );
        let mut _count = 0;
        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::InDiscovery => {
                    debug!("[*] {:?}: InDiscovery", self.agent.position());

                    let _generated_image = self.generate_image_from_text(tasks).await?;
                    // let generated_text = self.generate_text_from_image("./img.jpg").await?;

                    // let text_similarity = self
                    //     .compare_text_and_image_prompts(tasks, &generated_text)
                    //     .await?;
                    // debug!(
                    //     "[*] {:?}: InDiscovery: {}",
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
