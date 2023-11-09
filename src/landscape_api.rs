use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_derive::Deserialize;
use sha2::Sha256;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::Read,
    path::PathBuf,
};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Script {
    pub username: String,
    pub time_limit: u32,
    pub attachments: Vec<String>,
    pub title: String,
    pub creator: Creator,
    pub access_group: String,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
pub struct Creator {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ScriptExec {
    pub computer_id: Option<String>,
    pub creation_time: String,
    pub creator: Creator,
    pub id: u32,
    pub parent_id: Option<String>,
    pub summary: String,
    #[serde(rename = "type")]
    pub group_type: String,
}

pub struct Api {
    api_uri: String,
    api_key: String,
    api_secret: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Computer {
    comment: Option<String>,
    total_swap: Option<i32>,
    total_memory: Option<i32>,
    annotations: Option<HashMap<String, String>>,
    title: Option<String>,
    last_ping_time: Option<String>,
    hostname: Option<String>,
    container_info: Option<String>,
    last_exchange_time: Option<String>,
    update_manager_prompt: Option<String>,
    tags: Option<Vec<String>>,
    cloud_instance_metadata: HashMap<String, String>, // Assuming String values. Adjust as needed.
    access_group: Option<String>,
    distribution: Option<String>,
    id: i32,
    reboot_required_flag: bool,
    vm_info: Option<String>,
}

impl Api {
    pub fn new() -> Self {
        let api_uri = std::env::var("LANDSCAPE_API_URI").expect("LANDSCAPE_API_URI");
        let api_key = std::env::var("LANDSCAPE_API_KEY").expect("LANDSCAPE_API_KEY");
        let api_secret = std::env::var("LANDSCAPE_API_SECRET").expect("LANDSCAPE_API_SECRET");
        Self {
            api_uri,
            api_key,
            api_secret,
        }
    }

    //
    // Signing the API request. See the fn create_signature(...) below for more
    // details
    //
    fn sign_api_call(&self, http_method: &str, map: &mut BTreeMap<String, String>) {
        let url_parse = Url::parse(&self.api_uri).unwrap();
        let host = url_parse.host().unwrap();
        let uri = url_parse.path();

        map.insert("access_key_id".to_string(), self.api_key.clone());
        map.insert("signature_method".to_string(), "HmacSHA256".to_string());
        map.insert("signature_version".to_string(), "2".to_string());
        map.insert("version".to_string(), "2011-08-01".to_string());
        // map.insert("version".to_string(), "2013-11-04".to_string());

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        // map.insert("timestamp".to_string(), encode(&now).into_owned());
        map.insert("timestamp".to_string(), encode_rfc3986(&now));

        let signature = Api::create_signature(
            self.api_secret.as_bytes(),
            map.clone(),
            http_method,
            &host.to_string(),
            uri,
        )
        .unwrap();

        map.insert("signature".to_string(), encode_rfc3986(&signature));
    }

    //
    // See CreateScriptAttachment at https://ubuntu.com/landscape/docs/api-scripts
    //
    pub fn create_script_attachment(&self, scriptname: &str, path: &PathBuf) -> String {
        // Read the file to a byte array
        let mut content = Vec::new();
        let mut the_file = File::open(path).expect("Unable to read file");
        the_file
            .read_to_end(&mut content)
            .expect("Unable to load file to the memory");

        // let content = std::fs::read_to_string(path).expect("Unable to read file");

        let encoded = general_purpose::STANDARD.encode(&content);

        if let Some(script_id) = self.get_script(scriptname) {
            let mut map = BTreeMap::new();

            map.insert("action".to_string(), "CreateScriptAttachment".to_string());
            map.insert("script_id".to_string(), script_id.id.to_string());
            let filename = path.file_name().unwrap().to_str().unwrap();
            map.insert(
                "file".to_string(),
                encode_rfc3986(&format!("{}$${}", filename, encoded)),
            );

            self.sign_api_call("POST", &mut map);

            let mut req = minreq::post(&self.api_uri);
            for (key, value) in map {
                req = req.with_param(&key, &value);
            }

            req.send().unwrap().as_str().unwrap().to_string()
        } else {
            panic!("Script not found")
        }
    }

    //
    // See RemoveScriptAttachment at https://ubuntu.com/landscape/docs/api-scripts
    //
    pub fn remove_script_attachment(&self, scriptname: &str, path: PathBuf) -> String {
        // Find the script
        if let Some(script_id) = self.get_script(scriptname) {
            let mut map = BTreeMap::new();

            map.insert("action".to_string(), "RemoveScriptAttachment".to_string());
            map.insert("script_id".to_string(), script_id.id.to_string());
            let filename = path.file_name().unwrap().to_str().unwrap();
            map.insert("filename".to_string(), filename.to_string());

            self.sign_api_call("POST", &mut map);

            let mut req = minreq::post(&self.api_uri);
            for (key, value) in map {
                req = req.with_param(&key, &value);
            }

            req.send().unwrap().as_str().unwrap().to_string()
        } else {
            panic!("Script not found")
        }
    }

    //
    // See GetScriptAttachments at https://ubuntu.com/landscape/docs/api-scripts
    //
    pub fn get_script_attachments(&self, scriptname: &str) -> Vec<String> {
        if let Some(script) = &self.get_script(scriptname) {
            script.attachments.iter().map(|a| a.to_string()).collect()
        } else {
            panic!("Script not found")
        }
    }

    //
    // API does not allow query a single script. As we already can query all scripts
    // we are iterating to find a particular script we are interested in.
    //
    pub fn get_script(&self, name: &str) -> Option<Script> {
        let scripts = self.get_scripts();

        if let Some(s) = scripts.iter().find(|s| s.title.starts_with(name)) {
            Some(Script {
                username: s.username.clone(),
                title: s.title.clone(),
                time_limit: s.time_limit,
                attachments: s.attachments.clone(),
                creator: Creator {
                    id: s.creator.id,
                    name: s.creator.name.clone(),
                    email: s.creator.email.clone(),
                },
                access_group: s.access_group.clone(),
                id: s.id,
            })
        } else {
            panic!("Script not found")
        }
    }

    //
    // See GetScripts at https://ubuntu.com/landscape/docs/api-scripts
    //
    pub fn get_scripts(&self) -> Vec<Script> {
        let mut map = BTreeMap::new();

        map.insert("action".to_string(), "GetScripts".to_string());

        self.sign_api_call("POST", &mut map);

        let mut req = minreq::post(&self.api_uri);
        for (key, value) in map {
            req = req.with_param(&key, &value);
        }

        let res = req.send().unwrap();

        // res.as_str().unwrap().to_string()
        res.json::<Vec<Script>>().unwrap()
    }

    //
    // See ExecuteScript at https://ubuntu.com/landscape/docs/api-scripts
    //
    pub fn execute_script(&self, host_query: &str, script_name: &str) -> ScriptExec {
        let scripts = self.get_scripts();
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        if let Some(s) = scripts.iter().find(|s| s.title.starts_with(script_name)) {
            let script_id = s.id;
            map.insert("action".to_string(), "ExecuteScript".to_string());
            map.insert("query".to_string(), host_query.to_string());
            map.insert("script_id".to_string(), script_id.to_string());

            self.sign_api_call("POST", &mut map);

            let mut req = minreq::post(&self.api_uri);
            for (key, value) in map {
                req = req.with_param(&key, &value);
            }
            // dbg!(&req);
            let res = req.send().unwrap();

            res.json::<ScriptExec>().unwrap()
        } else {
            panic!("Script not found")
        }
    }

    //
    // Every call to the Landscape API must be signed. Signing protocol is defined at
    // https://ubuntu.com/landscape/docs/low-level-http-requests
    //
    fn create_signature(
        secret_key: &[u8],
        params: BTreeMap<String, String>,
        http_verb: &str,
        host: &str,
        uri: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Step 1: Create Canonicalized Query String
        let mut canonical_query = String::new();
        for (key, value) in &params {
            canonical_query.push_str(&encode_rfc3986(key));
            canonical_query.push('=');
            if key.starts_with("timestamp") || key.starts_with("file") {
                canonical_query.push_str(value)
            } else {
                canonical_query.push_str(&encode_rfc3986(value));
            }
            canonical_query.push('&');
        }
        canonical_query.pop(); // remove trailing '&'

        // Step 2: Create String to Sign
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            http_verb,
            host.to_lowercase(),
            uri,
            canonical_query
        );

        // Step 3: Calculate HMAC
        let mut hmac = Hmac::<Sha256>::new_from_slice(secret_key).unwrap();
        hmac.update(string_to_sign.as_bytes());
        let content = hmac.finalize().into_bytes();

        // Step 4: Convert to Base64
        let signature = general_purpose::STANDARD.encode(content);

        Ok(signature)
    }

    //
    // See GetComputers at https://ubuntu.com/landscape/docs/api-computers
    //
    pub fn get_all_hosts(&self) -> Vec<Computer> {
        let mut map = BTreeMap::new();

        map.insert("action".to_string(), "GetComputers".to_string());

        self.sign_api_call("POST", &mut map);

        let mut req = minreq::post(&self.api_uri);
        for (key, value) in map {
            req = req.with_param(&key, &value);
        }

        let res = req.send().unwrap();

        // res.as_str().unwrap().to_string()
        res.json::<Vec<Computer>>().unwrap()
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

// urlencode::encode() will encode characters indiscriminately, including
// the ones that we should not encode for the Landscape
// this custom function solves the problem
fn encode_rfc3986(input: &str) -> String {
    let reserved_chars = "abcdefghijklmnopqrstuvwxyz\
                          ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                          0123456789\
                          -._~";
    let mut result = String::with_capacity(input.len() * 3);
    for character in input.chars() {
        if reserved_chars.contains(character) {
            result.push(character);
        } else if character == ' ' {
            result.push_str("%20");
        } else {
            result.push_str(&format!("%{:02X}", character as u32));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_api_creation() {
        let api = Api::new();
        assert_eq!(api.api_uri, env::var("LANDSCAPE_API_URI").unwrap());
        assert_eq!(api.api_key, env::var("LANDSCAPE_API_KEY").unwrap());
        assert_eq!(api.api_secret, env::var("LANDSCAPE_API_SECRET").unwrap());
    }

    #[test]
    #[should_panic(expected = "Script not found")]
    fn test_get_script_not_found() {
        let api = Api::new();
        api.get_script("nonexistent");
    }

    #[test]
    #[should_panic(expected = "Unable to read file")]
    fn test_create_script_attachment_invalid_file() {
        let api = Api::new();
        api.create_script_attachment("test_script", &PathBuf::from("invalid_path"));
    }

    #[test]
    #[should_panic(expected = "Script not found")]
    fn test_remove_script_attachment_script_not_found() {
        let api = Api::new();
        api.remove_script_attachment("nonexistent", PathBuf::from("valid_path"));
    }

    #[test]
    #[should_panic(expected = "Script not found")]
    fn test_get_script_attachments_script_not_found() {
        let api = Api::new();
        api.get_script_attachments("nonexistent");
    }

    #[test]
    #[should_panic(expected = "Script not found")]
    fn test_execute_script_script_not_found() {
        let api = Api::new();
        api.execute_script("valid_query", "nonexistent");
    }

    #[test]
    fn test_encode_rfc3986() {
        let encoded = encode_rfc3986("test string with spaces");
        assert_eq!(encoded, "test%20string%20with%20spaces");
    }

    #[test]
    fn test_encode_rfc3986_empty_string() {
        let encoded = encode_rfc3986("");
        assert_eq!(encoded, "");
    }
}
