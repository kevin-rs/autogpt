extern crate proc_macro;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(AutoNet)]
pub fn auto_net(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
            #[async_trait::async_trait]
            impl Network for #name {
                async fn heartbeat(&self) {
                    let clients = self.clients.clone();
                    let peer_addresses = self.peer_addresses.clone();
                    let signer = self.signer.clone();
                    let id = self.id.to_string();
                    let interval = self.heartbeat_interval;

                    tokio::spawn(async move {
                        loop {
                            for (peer_id, client) in &clients {
                                let msg = Message::ping(&id, peer_id, 0);
                                let result = {
                                    let client = client.lock().await;
                                    client.send(msg).await
                                };

                                if let Err(e) = result {
                                    debug!("Heartbeat failed to {peer_id}: {e}");

                                    if let Some(addr) = peer_addresses.get(peer_id) {
                                        debug!("Attempting to reconnect to {peer_id} at {addr}...");

                                        match Client::connect(addr, signer.clone()).await {
                                            Ok(new_client) => {
                                                debug!("Reconnected to {peer_id}");
                                                let mut locked = client.lock().await;
                                                *locked = new_client;
                                            }
                                            Err(err) => {
                                                debug!("Failed to reconnect to {peer_id}: {err}");
                                            }
                                        }
                                    } else {
                                        debug!("No known address for {peer_id}, cannot reconnect.");
                                    }
                                }
                            }

                            tokio::time::sleep(interval).await;
                        }
                    });
                }

                async fn broadcast(&self, payload: &str) -> anyhow::Result<()> {
                    let tasks = self.clients.iter().map(|(peer_id, client)| {
                        let mut msg = Message::broadcast(&self.id, payload, 0);
                        msg.to = peer_id.clone();
                        let client = client.clone();
                        async move {
                            let send_result = {
                                let client_guard = client.lock().await;
                                let client_ref = client_guard.clone();
                                client_ref
                            }.send(msg).await;

                            if let Err(e) = send_result {
                                debug!("Broadcast to {peer_id} failed: {e}");
                            } else {
                                debug!("Broadcast to {peer_id} succeeded");
                            }
                        }
                    });

                    futures::future::join_all(tasks).await;
                    Ok(())
                }
            }
        };

    proc_macro::TokenStream::from(expanded)
}
