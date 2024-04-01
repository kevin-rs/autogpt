use crate::agents::agent::AgentGPT;
use crate::common::utils::{Status, Tasks};
use crate::prompts::designer::{STABILITY_PROMPT, WEB_DESIGNER_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use gems::utils::load_and_encode_image;
use gems::Client;
use stabilityai::{
    types::{ClipGuidancePreset, Sampler, StylePreset, TextToImageRequestBodyArgs},
    Client as StabilityAIClient,
};
use std::env::var;
use tracing::info;

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }

    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, char1) in s1.chars().enumerate() {
        for (j, char2) in s2.chars().enumerate() {
            let cost = if char1 == char2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

fn similarity(s1: &str, s2: &str) -> f64 {
    let distance = levenshtein_distance(s1, s2) as f64;
    let max_length = s1.chars().count().max(s2.chars().count()) as f64;
    1.0 - distance / max_length
}

#[derive(Debug)]
pub struct DesignerGPT {
    agent: AgentGPT,
    stab_client: StabilityAIClient,
    client: Client,
}

impl DesignerGPT {
    pub fn new(objective: &'static str, position: &'static str) -> Self {
        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);
        let _stability_api_key = var("STABILITY_API_KEY").unwrap_or_default().to_owned();
        let stab_client = StabilityAIClient::new();
        let model = var("GEMINI_MODEL")
            .unwrap_or("gemini-pro-vision".to_string())
            .to_owned();
        let api_key = var("GEMINI_API_KEY").unwrap_or_default().to_owned();
        let client = Client::new(&api_key, &model);

        info!("[*] {:?}: {:?}", position, agent);

        Self {
            agent,
            stab_client,
            client,
        }
    }

    pub async fn generate_image_from_text(&mut self, tasks: &Tasks) -> Result<()> {
        let text_prompt: String =
            format!("{}\n\nUser Prompt: {}", STABILITY_PROMPT, tasks.description);
        let request = TextToImageRequestBodyArgs::default()
            .text_prompts(text_prompt)
            .samples(1)
            .steps(30_u32)
            .clip_guidance_preset(ClipGuidancePreset::FastBlue)
            .sampler(Sampler::KDpmpp2sAncestral)
            .width(1216_u16)
            .height(832_u16)
            .style_preset(StylePreset::ThreeDModel)
            .build()?;

        let artifacts = self
            .stab_client
            .generate("stable-diffusion-xl-1024-v1-0")
            .text_to_image(request)
            .await?;

        let paths = artifacts.save("images/").await?;

        paths.iter().for_each(|path| {
            info!(
                "[*] {:?}: Image saved at {}",
                self.agent.position(),
                path.display()
            )
        });

        Ok(())
    }

    pub async fn generate_text_from_image(&mut self, image_path: &str) -> Result<String> {
        let base64_image_data = match load_and_encode_image(&image_path) {
            Ok(data) => data,
            Err(_) => {
                info!("[*] {:?}: Error loading image!", self.agent.position());
                "".to_string()
            }
        };

        let response = self
            .client
            .generate_content_with_image(WEB_DESIGNER_PROMPT, &base64_image_data)
            .await
            .unwrap();

        info!(
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
        let gems_response = self.generate_text_from_image(generated_text).await?;

        let similarity_threshold = 0.8;
        let similarity = similarity(&stability_ai_prompt, &gems_response);

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

    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool) -> Result<()> {
        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::InDiscovery => {
                    info!("[*] {:?}: InDiscovery", self.agent.position());

                    let generated_image_path = self.generate_image_from_text(tasks).await?;
                    // let generated_text = self.generate_image_from_text(tasks).await?;
                    info!(
                        "[*] {:?}: InDiscovery: generated_image_path {:?}",
                        self.agent.position(),
                        generated_image_path
                    );

                    // let text_similarity = self.compare_text_and_image_prompts(tasks, &generated_text).await?;

                    // if text_similarity {
                    //     self.agent.update(Status::Completed);
                    // }
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
