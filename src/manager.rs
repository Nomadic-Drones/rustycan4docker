/*
 * Filename: manager.rs
 * Created Date: Tuesday, October 18th 2022, 5:15:15 pm
 * Author: Jonathan Haws
 *
 * Copyright (c) 2022 WiTricity
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use crate::endpoint::Endpoint;
use crate::network::{JoinResponse, Network};
use bollard::network::ListNetworksOptions;
use bollard::Docker;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct NetworkManager {
    network_list: Arc<RwLock<HashMap<String, Network>>>,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
            network_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn network_load(&self) {
        let connection = Docker::connect_with_unix_defaults().unwrap();

        let list_networks_filters: HashMap<&str, Vec<&str>> = HashMap::new();
        let config = ListNetworksOptions {
            filters: list_networks_filters,
        };
        match connection.list_networks(Some(config)).await {
            Ok(networks) => {
                for n in networks {
                    match (n.driver, n.options, n.id) {
                        (Some(driver), Some(options), Some(nid)) => {
                            if driver.eq("rustyvxcan") {
                                let device = if options.contains_key("vxcan.dev") {
                                    options["vxcan.dev"].clone()
                                } else {
                                    String::from("vcan")
                                };
                                let peer = if options.contains_key("vxcan.peer") {
                                    options["vxcan.peer"].clone()
                                } else {
                                    String::from("vcan")
                                };
                                let canid = if options.contains_key("vxcan.id") {
                                    options["vxcan.id"].clone()
                                } else {
                                    String::from("0")
                                };

                                let nw = Network::new(device, peer, canid);
                                self.network_list.write().insert(nid, nw);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => eprintln!(" !! Unable to get docker networks: {}", e),
        }
    }

    pub fn network_create(&self, uid: String, options: String) {
        // Print the options and extract the right values
        // Add the network to the hashmap
        println!(
            " -> Adding network with id '{}' with options '{}'",
            uid, options
        );

        match self.options_parse(options) {
            Ok((d, p, c)) => {
                let nw = Network::new(d, p, c);
                self.network_list.write().insert(uid, nw);
            }
            Err(_) => {}
        }
    }

    pub fn network_delete(&self, uid: String) {
        let mut map = self.network_list.write();
        if map.contains_key(&uid) {
            println!(" -> Network {uid} exists...removing!");
            map.remove(&uid);
        }
    }

    pub fn endpoint_create(&self, nuid: String, epuid: String) {
        // Create the endpoint
        let ep = Endpoint::new(epuid);

        // Lock the network list
        let mut map = self.network_list.write();
        match map.get_mut(&nuid) {
            Some(n) => {
                // Add the endpoint to the network
                n.endpoint_add(ep)
            }
            None => (),
        };
    }

    pub fn endpoint_delete(&self, nuid: String, epuid: String) {
        // Lock the network list
        let mut map = self.network_list.write();
        match map.get_mut(&nuid) {
            Some(n) => {
                // Remove the endpoint from the network
                n.endpoint_remove(epuid)
            }
            None => (),
        };
    }

    pub fn endpoint_attach(
        &self,
        nuid: String,
        epuid: String,
        _sbox: String,
        options: String,
    ) -> Result<JoinResponse, Error> {
        // Lock the network list
        let mut map = self.network_list.write();
        match map.get_mut(&nuid) {
            Some(n) => {
                let peer = match serde_json::from_str::<serde_json::Value>(&options) {
                    Ok(v) => match v["vxcan.peer"].as_str() {
                        Some(u) => u.to_string(),
                        None => String::new(),
                    },
                    Err(_) => String::new(),
                };

                let namespace = String::new();

                // Add the endpoint to the network
                let rsp = n.endpoint_attach(epuid, namespace, peer)?;
                Ok(rsp)
            }
            None => Err(Error),
        }
    }

    pub fn endpoint_detach(&self, nuid: String, epuid: String) {
        // Lock the network list
        let mut map = self.network_list.write();
        match map.get_mut(&nuid) {
            Some(n) => {
                // Detach the endpoint from the network
                n.endpoint_detach(epuid)
            }
            None => (),
        };
    }

    fn options_parse(&self, options: String) -> Result<(String, String, String), Error> {
        match serde_json::from_str::<serde_json::Value>(&options) {
            Ok(v) => {
                let device = match v["vxcan.dev"].as_str() {
                    Some(u) => u.to_string(),
                    None => {
                        println!(" !! Error parsing vxcan.dev option: {}", v["vxcan.dev"]);
                        String::from("vcan")
                    }
                };
                let peer = match v["vxcan.peer"].as_str() {
                    Some(u) => u.to_string(),
                    None => {
                        println!(" !! Error parsing vxcan.peer option: {}", v["vxcan.peer"]);
                        String::from("vcanp")
                    }
                };
                let canid = match v["vxcan.id"].as_str() {
                    Some(u) => u.to_string(),
                    None => {
                        println!(" !! Error parsing vxcan.dev option: {}", v["vxcan.dev"]);
                        String::from("0")
                    }
                };

                // Return the tuple of options
                Ok((device, peer, canid))
            }
            Err(_) => Err(Error),
        }
    }
}
