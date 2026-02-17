use std::{collections::HashMap, io::Read, sync::Arc};

use anyhow::{anyhow, Result};
use reqwest::cookie::Jar;
use serde::{Deserialize, Serialize};
use redb::ReadableTable;

// ===== Data Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub password: String,
    pub strategy: String,
    pub server: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_timestamp")]
    pub last_used: Option<u64>,
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    
    let opt: Option<f64> = Option::deserialize(deserializer)?;
    Ok(opt.map(|f| f as u64))
}

// ===== Database Engine =====

pub struct StoreEngine {
    db: redb::Database,
}

const TABLE: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("users");

impl StoreEngine {
    pub fn create(path: &str) -> Result<Self> {
        let db = redb::Database::create(path)?;
        
        // Garantir que a tabela existe
        let write_txn = db.begin_write()?;
        {
            let _table = write_txn.open_table(TABLE)?;
        }
        write_txn.commit()?;
        
        Ok(Self { db })
    }

    pub fn get_user(&self, uuid: &uuid::Uuid) -> Option<UserInfo> {
        let read_txn = self.db.begin_read().ok()?;
        let table = read_txn.open_table(TABLE).ok()?;
        let key = uuid.to_string();
        let value = table.get(key.as_str()).ok()??;
        let json = value.value();
        serde_json::from_str(json).ok()
    }

    pub fn users(&self) -> Vec<(uuid::Uuid, UserInfo)> {
        let mut users = Vec::new();
        
        // Return empty if table doesn't exist yet
        let read_txn = match self.db.begin_read() {
            Ok(txn) => txn,
            Err(_) => return users,
        };
        
        let table = match read_txn.open_table(TABLE) {
            Ok(t) => t,
            Err(_) => return users,
        };
        
        if let Ok(mut iter) = table.iter() {
            while let Some((key, value)) = iter.next() {
                let key_str: &str = key.value();
                let value_str: &str = value.value();
                
                if let (Ok(uuid), Ok(user)) = (
                    uuid::Uuid::parse_str(key_str),
                    serde_json::from_str::<UserInfo>(value_str)
                ) {
                    users.push((uuid, user));
                }
            }
        }
        
        users
    }

    pub fn insert(&mut self, uuid: &uuid::Uuid, user: &UserInfo) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let key = uuid.to_string();
            let json = serde_json::to_string(user)?;
            table.insert(key.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn remove(&mut self, uuid: &uuid::Uuid) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let key = uuid.to_string();
            table.remove(key.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}

// ===== Strategy System =====

#[derive(Default)]
pub struct Strategy {
    scripts: HashMap<String, String>,
}

impl Strategy {
    pub fn new(pattern: &str) -> Self {
        let mut strategy = Strategy::default();
        strategy.load(pattern);
        strategy
    }

    /// Load scripts from path that match a glob pattern.
    pub fn load(&mut self, pattern: &str) {
        for entry in glob::glob(pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                let script = std::fs::read_to_string(path).unwrap();
                self.scripts.insert(file_name, script);
            }
        }
    }

    /// Get a vector that lists all strategy name.
    pub fn list(&self) -> Vec<String> {
        self.scripts.keys().map(|key| key.to_owned()).collect()
    }

    pub fn get(&self, name: &str) -> Result<String> {
        let script = self
            .scripts
            .get(name)
            .ok_or_else(|| anyhow!(format!("stratrgy {} do not exist", name)))?
            .to_owned();

        Ok(script)
    }
}

/// Execute a strategy by name
pub fn execute_strategy(
    script: &str,
    username: &str,
    password: &str,
    server: &str,
) -> Result<String> {
    let lua = mlua::Lua::new();
    let globals = lua.globals();

    let agent_constructor = lua.create_function(|_, ()| Ok(Agent::new()))?;
    globals.set("agent", agent_constructor)?;

    let crypto_rs = lua.create_table()?;
        let md5_func = lua.create_function(|_, input: String| {
            let digest = md5::compute(input.as_bytes());
            Ok(format!("{:x}", digest))
        })?;

    crypto_rs.set("md5", md5_func)?;
    globals.set("crypto", crypto_rs)?;

    let cowv2_func =
        lua.create_function(|_, (url, re, title): (String, String, String)| {
            let result = get_cookie_by_cowv2(url, re, title).unwrap();
            Ok(result)
        })?;
    globals.set("get_cookie_by_cowv2", cowv2_func)?;

    lua.load(script).exec()?;
    let login_function: mlua::Function = globals.get("login")?;
    let result = login_function.call::<_, String>((username, password, server))?;

    Ok(result)
}

struct Agent {
    client: reqwest::blocking::Client,
    cookie_jar: Arc<Jar>,
}

impl Agent {
    fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1; .NET CLR 1.0.3705;)"
                .parse()
                .unwrap(),
        );

        let cookie_jar: Arc<Jar> = Default::default();
        let cookie_jar1 = cookie_jar.clone();

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .cookie_provider(cookie_jar1)
            .build()
            .unwrap();

        Self { client, cookie_jar }
    }
}

impl mlua::UserData for Agent {
    fn add_methods<'lua, T: mlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("get", |_, agent, (url,): (String,)| {
            let response = agent.client.get(url).send().unwrap().text().unwrap();
            Ok(response)
        });

        methods.add_method("get_with", |_, agent, (url,): (String,)| {
            let response = agent.client.get(url).send().unwrap();
            let url = response.url();
            let url = format!("{}://{}/", url.scheme(), url.host().unwrap());
            let text = response.text().unwrap();
            Ok((text, url))
        });

        methods.add_method("post", |_, agent, (url, form): (String, mlua::Table)| {
            let form: std::collections::HashMap<String, String> = form
                .pairs::<String, String>()
                .into_iter()
                .map(|pair| {
                    let (k, v) = pair.unwrap();
                    (k, v)
                })
                .collect();

            let response = agent.client.post(url).form(&form).send().unwrap();
            let response_text = response.text().unwrap();

            Ok(response_text)
        });

        methods.add_method(
            "load_cookie",
            |_, agent, (url, cookies): (String, String)| {
                let url: reqwest::Url = url.parse().unwrap();

                let jar1 = agent.cookie_jar.clone();
                for cookie in cookies.split(';').map(|x| x.trim()) {
                    jar1.add_cookie_str(cookie, &url);
                }

                Ok("".to_owned())
            },
        );
    }
}

fn get_cookie_by_cowv2(url: String, re: String, title: String) -> Result<String> {
    let mut cowv2 = std::process::Command::new("cowv2")
        .args(["-u", &url, "-r", &re, "-t", &title])
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let status = cowv2.wait()?;
    if status.success() {
        let mut stdout = cowv2.stdout.take().unwrap();
        let mut cookies = String::new();
        stdout.read_to_string(&mut cookies)?;
        Ok(cookies)
    } else {
        Err(anyhow::anyhow!("cowv2 exit with no cookie"))
    }
}
