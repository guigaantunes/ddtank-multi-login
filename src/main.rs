// Inspector habilitado - remover windows_subsystem para permitir debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ddtank_rs::{StoreEngine, UserInfo};

use sciter::{make_args, Value};
use sciter::window::{Builder, Options};
use std::thread;
use std::sync::{Arc, Mutex};
use std::process::Child;

struct DDTankHandler {
    strategy: ddtank_rs::Strategy,
    db: StoreEngine,
    child_processes: Arc<Mutex<Vec<Child>>>,
}

impl DDTankHandler {
    fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        
        let scripts_pattern = exe_dir.join("scripts").join("*.lua");
        let db_path = exe_dir.join("userdata.redb");
        
        let strategy = ddtank_rs::Strategy::new(scripts_pattern.to_str().unwrap());
        let db = StoreEngine::create(db_path.to_str().unwrap()).unwrap();
        Self { 
            strategy, 
            db,
            child_processes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn login(
        &mut self,
        strategy: String,
        username: String,
        password: String,
        server: String,
        done_callback: Value,
    ) -> bool {
        let script = self.strategy.get(&strategy).unwrap();
        thread::spawn(move || {
            let result = ddtank_rs::execute_strategy(&script, &username, &password, &server);
            let result = match result {
                Ok(url) => {
                    println!("Login bem-sucedido! URL retornada: {}", url);
                    url
                },
                Err(err) => {
                    println!("Erro no login: {}", err);
                    format!("error{}", err.to_string())
                },
            };
            done_callback.call(None, &make_args!(result), None).unwrap();
        });
        true
    }

    fn get_all_strategy(&self) -> Value {
        let strategy_list = self.strategy.list();
        Value::from_iter(strategy_list)
    }

    fn play_flash(&self, url: String) -> anyhow::Result<()> {
        println!("Tentando abrir: {}", url);
        
        // Se for URL do 337.com, usar comando do Windows para abrir
        if url.contains("337.com") || url.starts_with("http://s") {
            println!("Detectada URL do 337.com - abrindo com navegador padrão");
            std::process::Command::new("cmd")
                .args(&["/c", "start", "", &url])
                .spawn()?;
            return Ok(());
        }
        
        // Para URLs SWF, usar flashplayer
        let flashplayer = if cfg!(target_os = "windows") {
            "./flashplayer_sa.exe"
        } else {
            "./flashplayer"
        };

        println!("Abrindo com flashplayer: {}", flashplayer);
        std::process::Command::new(flashplayer).arg(url).output()?;
        Ok(())
    }

    fn open_reguinha(&self) -> bool {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        
        let reguinha_path = exe_dir.join("reguinha.exe");
        
        match std::process::Command::new(&reguinha_path).spawn() {
            Ok(child) => {
                println!("Reguinha aberto com sucesso! PID: {}", child.id());
                if let Ok(mut processes) = self.child_processes.lock() {
                    processes.push(child);
                }
                true
            },
            Err(e) => {
                eprintln!("Erro ao abrir reguinha.exe: {}", e);
                false
            }
        }
    }

    fn database_get(&self, user_id: String) -> Value {
        let uuid = match uuid::Uuid::parse_str(&user_id) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Error parsing UUID: {:?}", e);
                return Value::new();
            }
        };
        
        let user = match self.db.get_user(&uuid) {
            Some(u) => u,
            None => {
                eprintln!("User not found: {}", user_id);
                return Value::new();
            }
        };
        
        // Manually construct Value
        let mut user_obj = Value::new();
        user_obj.set_item("username", user.username);
        user_obj.set_item("password", user.password);
        user_obj.set_item("strategy", user.strategy);
        user_obj.set_item("server", user.server);
        if let Some(nickname) = user.nickname {
            user_obj.set_item("nickname", nickname);
        }
        if let Some(last_used) = user.last_used {
            user_obj.set_item("last_used", last_used as f64);
        }
        
        user_obj
    }

    fn database_get_all(&self) -> Value {
        let mut users_vec: Vec<(String, UserInfo)> = self
            .db
            .users()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        
        // Sort by last_used descending (most recent first)
        users_vec.sort_by(|a, b| {
            let a_time = a.1.last_used.unwrap_or(0);
            let b_time = b.1.last_used.unwrap_or(0);
            b_time.cmp(&a_time)
        });
        
        // Convert to a simple object structure that Sciter understands
        let mut result = Value::new();
        for (uuid, user) in users_vec {
            let mut user_obj = Value::new();
            user_obj.set_item("username", user.username);
            user_obj.set_item("password", user.password);
            user_obj.set_item("strategy", user.strategy);
            user_obj.set_item("server", user.server);
            if let Some(nickname) = user.nickname {
                user_obj.set_item("nickname", nickname);
            }
            if let Some(last_used) = user.last_used {
                user_obj.set_item("last_used", last_used as f64);
            }
            result.set_item(uuid, user_obj);
        }
        
        result
    }

    fn database_add(&mut self, user: Value) -> bool {
        let uuid = uuid::Uuid::new_v4();
        let mut user = user.clone();
        user.isolate();
        let user: UserInfo = match sciter_serde::from_value(&user) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Error deserializing user: {:?}", e);
                return false;
            }
        };
        match self.db.insert(&uuid, &user) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Error inserting user: {:?}", e);
                false
            }
        }
    }

    fn database_replace(&mut self, uuid: String, user: Value) -> bool {
        let uuid = match uuid::Uuid::parse_str(&uuid) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Error parsing UUID: {:?}", e);
                return false;
            }
        };
        let mut user = user.clone();
        user.isolate();
        let user: UserInfo = match sciter_serde::from_value(&user) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Error deserializing user: {:?}", e);
                return false;
            }
        };
        match self.db.insert(&uuid, &user) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Error replacing user: {:?}", e);
                false
            }
        }
    }

    fn database_delete(&mut self, uuid: String) -> bool {
        let uuid = match uuid::Uuid::parse_str(&uuid) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Error parsing UUID: {:?}", e);
                return false;
            }
        };
        match self.db.remove(&uuid) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Error deleting user: {:?}", e);
                false
            }
        }
    }
}

impl Drop for DDTankHandler {
    fn drop(&mut self) {
        println!("Encerrando processos filhos...");
        if let Ok(mut processes) = self.child_processes.lock() {
            for mut child in processes.drain(..) {
                match child.kill() {
                    Ok(_) => println!("Processo {} encerrado", child.id()),
                    Err(e) => eprintln!("Erro ao encerrar processo: {}", e),
                }
            }
        }
    }
}

impl sciter::EventHandler for DDTankHandler {
    sciter::dispatch_script_call! {
        fn login(String, String, String, String, Value);
        fn get_all_strategy();
        fn play_flash(String);
        fn open_reguinha();
        fn database_get(String);
        fn database_get_all();
        fn database_add(Value);
        fn database_replace(String, Value);
        fn database_delete(String);
    }
}

fn main() {
    println!("=== DDTank-RS ===");
    println!("Inspector habilitado! Use Ctrl+Shift+I para abrir o inspetor do Sciter.");
    println!("Ou clique com botão direito e selecione 'Inspect Element'");
    println!("=================\n");
    
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    
    let ddtank_handler = DDTankHandler::new();

    let mut frame = Builder::main().create();
    let _ = frame.set_options(Options::DebugMode(true));

    // Tentar carregar recursos empacotados (`src/ui.rc` ou `ui.rc`), senão carregar arquivos do sistema de arquivos
    let resources = std::fs::read(exe_dir.join("src").join("ui.rc"))
        .or_else(|_| std::fs::read(exe_dir.join("ui.rc")));

    match resources {
        Ok(bytes) => {
            if let Err(_) = frame.archive_handler(&bytes) {
                eprintln!("Invalid archive found in ui.rc, falling back to filesystem UI");
                let index = exe_dir.join("src").join("ui").join("index.htm");
                let index_path = index.to_str().unwrap().replace("\\", "/");
                frame.event_handler(ddtank_handler);
                frame.load_file(&format!("file:///{index}", index = index_path));
                frame.run_app();
                return;
            }
            frame.event_handler(ddtank_handler);
            frame.load_file("this://app/index.htm");
            frame.run_app();
        }
        Err(_) => {
            // ui.rc não existe — carregar diretamente do disco (modo dev)
            let index = exe_dir.join("src").join("ui").join("index.htm");
            let index_path = index.to_str().unwrap().replace("\\", "/");
            frame.event_handler(ddtank_handler);
            frame.load_file(&format!("file:///{index}", index = index_path));
            frame.run_app();
        }
    }
}
