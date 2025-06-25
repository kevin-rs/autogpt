extern crate proc_macro;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Auto)]
pub fn derive_agent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl Agent for #name {
            fn new(objective: std::borrow::Cow<'static, str>, position: std::borrow::Cow<'static, str>) -> Self {
                Self {
                    objective,
                    position,
                    ..Default::default()
                }
            }

            fn update(&mut self, status: Status) {
                self.status = status;
            }

            fn objective(&self) -> &std::borrow::Cow<'static, str> {
                &self.objective
            }

            fn position(&self) -> &std::borrow::Cow<'static, str> {
                &self.position
            }

            fn status(&self) -> &Status {
                &self.status
            }

            fn memory(&self) -> &Vec<Communication> {
                &self.memory
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
                tasks: &'a mut Tasks,
                execute: bool,
                browse: bool,
                max_tries: u64,
            ) -> Result<()> {
                <#name as AgentExecutor>::execute(self, tasks, execute, browse, max_tries).await
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
    };

    proc_macro::TokenStream::from(expanded)
}
