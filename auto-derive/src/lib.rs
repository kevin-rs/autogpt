extern crate proc_macro;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Auto)]
pub fn derive_agent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl Agent for #name {
            fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
                let mut agent = Self::default();
                agent.agent.objective = objective;
                agent.agent.position = position;
                agent
            }

            fn update(&mut self, status: Status) {
                self.agent.update(status);
            }

            fn objective(&self) -> &std::borrow::Cow<'static, str> {
                &self.agent.objective
            }

            fn position(&self) -> &std::borrow::Cow<'static, str> {
                &self.agent.position
            }

            fn status(&self) -> &Status {
                &self.agent.status
            }

            fn memory(&self) -> &Vec<Communication> {
                &self.agent.memory
            }

            fn tools(&self) -> &Vec<Tool> {
                &self.agent.tools
            }

            fn knowledge(&self) -> &Knowledge {
                &self.agent.knowledge
            }

            fn planner(&self) -> Option<&Planner> {
                self.agent.planner.as_ref()
            }

            fn persona(&self) -> &Persona {
                &self.agent.persona
            }

            fn collaborators(&self) -> &Vec<Collaborator> {
                &self.agent.collaborators
            }

            fn reflection(&self) -> Option<&Reflection> {
                self.agent.reflection.as_ref()
            }

            fn scheduler(&self) -> Option<&TaskScheduler> {
                self.agent.scheduler.as_ref()
            }

            fn capabilities(&self) -> &std::collections::HashSet<Capability> {
                &self.agent.capabilities
            }

            fn context(&self) -> &ContextManager {
                &self.agent.context
            }

            fn tasks(&self) -> &Vec<Task> {
                &self.agent.tasks
            }

            fn memory_mut(&mut self) -> &mut Vec<Communication> {
                &mut self.agent.memory
            }

            fn planner_mut(&mut self) -> Option<&mut Planner> {
                self.agent.planner.as_mut()
            }

            fn context_mut(&mut self) -> &mut ContextManager {
                &mut self.agent.context
            }
        }

        impl Functions for #name {
            fn get_agent(&self) -> &AgentGPT {
                &self.agent
            }
        }

        #[async_trait::async_trait]
        impl AsyncFunctions for #name {
            async fn execute<'a>(
                &'a mut self,
                tasks: &'a mut Task,
                execute: bool,
                browse: bool,
                max_tries: u64,
            ) -> Result<()> {
                <#name as Executor>::execute(self, tasks, execute, browse, max_tries).await
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

            #[cfg(any(feature = "oai", feature = "gem", feature = "cld", feature = "xai"))]
            async fn send_request(&mut self, request: &str) -> Result<String> {
                match &mut self.client {
                    #[cfg(feature = "gem")]
                    ClientType::Gemini(gem_client) => {
                        let parameters = ChatBuilder::default()
                            .messages(vec![Message::User {
                                content: Content::Text(request.to_string()),
                                name: None,
                            }])
                            .build()?;

                        let result = gem_client.chat().generate(parameters).await;
                        Ok(result.unwrap_or_default())
                    }

                    #[cfg(feature = "oai")]
                    ClientType::OpenAI(oai_client) => {
                        let parameters = ChatCompletionParametersBuilder::default()
                            .model(FlagshipModel::Gpt4O.to_string())
                            .messages(vec![ChatMessage::User {
                                content: ChatMessageContent::Text(request.to_string()),
                                name: None,
                            }])
                            .response_format(ChatCompletionResponseFormat::Text)
                            .build()?;

                        let result = oai_client.chat().create(parameters).await?;
                        let message = &result.choices[0].message;

                        Ok(match message {
                            ChatMessage::Assistant {
                                content: Some(chat_content),
                                ..
                            } => chat_content.to_string(),
                            ChatMessage::User { content, .. } => content.to_string(),
                            ChatMessage::System { content, .. } => content.to_string(),
                            ChatMessage::Developer { content, .. } => content.to_string(),
                            ChatMessage::Tool { content, .. } => content.clone(),
                            _ => String::new(),
                        })
                    }

                    #[cfg(feature = "cld")]
                    ClientType::Anthropic(client) => {
                        let body = CreateMessageParams::new(RequiredMessageParams {
                            model: "claude-3-7-sonnet-latest".to_string(),
                            messages: vec![AnthMessage::new_text(Role::User, request.to_string())],
                            max_tokens: 1024,
                        });

                        let chat_response = client.create_message(Some(&body)).await?;
                        Ok(chat_response
                            .content
                            .iter()
                            .filter_map(|block| match block {
                                ContentBlock::Text { text, .. } => Some(text.as_str()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("\n"))
                    }

                    #[cfg(feature = "xai")]
                    ClientType::Xai(xai_client) => {
                        let messages = vec![XaiMessage {
                            role: "user".into(),
                            content: request.to_string(),
                        }];

                        let rb = ChatCompletionsRequestBuilder::new(
                            xai_client.clone(),
                            "grok-beta".into(),
                            messages,
                        )
                        .temperature(0.0)
                        .stream(false);

                        let req = rb.clone().build()?;
                        let chat = rb.create_chat_completion(req).await?;
                        Ok(chat.choices[0].message.content.clone())
                    }


                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(anyhow::anyhow!(
                            "No valid AI client configured. Enable `gem`, `oai`, `cld`, or `xai` feature."
                        ));
                    }
                }
            }

        }
    };

    proc_macro::TokenStream::from(expanded)
}
