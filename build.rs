use embed_manifest::{embed_manifest, new_manifest};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/ui");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=scripts");
    println!("cargo:rerun-if-changed=C:/sciter-js-sdk-main/bin/windows/x64/sciter.dll");

    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let packfolder = r"C:\sciter-js-sdk-main\bin\windows\packfolder.exe";
        let sciter_dll = PathBuf::from(r"C:\sciter-js-sdk-main\bin\windows\x64\sciter.dll");
        let status = Command::new(packfolder)
            .arg("./src/ui")
            .arg("./src/ui.rc")
            .arg("-binary")
            .status()
            .expect("failed to execute packfolder.exe. Verify C:\\sciter-js-sdk-main\\bin\\windows\\packfolder.exe exists");

        if !status.success() {
            panic!("packfolder.exe failed to generate src/ui.rc");
        }

        if !sciter_dll.exists() {
            panic!("sciter.dll not found at C:\\sciter-js-sdk-main\\bin\\windows\\x64\\sciter.dll");
        }

        let profile = std::env::var("PROFILE").expect("PROFILE env var is not set");
        let target_root = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var is not set")).join("target")
            });
        let target_profile_dir = target_root.join(&profile);
        fs::create_dir_all(&target_profile_dir).expect("failed to create target profile directory");
        fs::copy(&sciter_dll, target_profile_dir.join("sciter.dll"))
            .expect("failed to copy sciter.dll to target profile directory");

        // Copiar pasta scripts para o diretório de saída se existir
        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let scripts_src = manifest_dir.join("scripts");
        if scripts_src.exists() {
            let scripts_dst = target_profile_dir.join("scripts");
            let _ = fs::remove_dir_all(&scripts_dst); // limpar primeiro
            copy_dir_recursive(&scripts_src, &scripts_dst)
                .expect("failed to copy scripts directory");
        }

        // Copiar ui.rc para o diretório de saída
        let ui_rc_src = manifest_dir.join("src").join("ui.rc");
        if ui_rc_src.exists() {
            fs::copy(&ui_rc_src, target_profile_dir.join("ui.rc"))
                .expect("failed to copy ui.rc to target directory");
        }

        // Copiar reguinha.exe para o diretório de saída
        let reguinha_src = manifest_dir.join("reguinha.exe");
        if reguinha_src.exists() {
            fs::copy(&reguinha_src, target_profile_dir.join("reguinha.exe"))
                .expect("failed to copy reguinha.exe to target directory");
        }

        // Inicializar banco de dados com conta de teste
        let db_path = target_profile_dir.join("userdata.redb");
        // Remover banco existente se for build release para sempre ter conta de teste fresca
        if profile == "release" && db_path.exists() {
            let _ = fs::remove_file(&db_path);
        }
        if !db_path.exists() {
            println!("Criando banco de dados com conta de teste: {:?}", db_path);
            if let Err(e) = initialize_database(&db_path) {
                eprintln!("Aviso: Erro ao criar banco de dados: {}", e);
            }
        }

        embed_manifest(new_manifest("Contoso.Sample"))
            .expect("unable to embed manifest file");
    }
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn initialize_database(db_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use redb::TableDefinition;
    
    const TABLE: TableDefinition<&str, &str> = TableDefinition::new("users");
    
    let db = redb::Database::create(db_path)?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        
        // Adicionar conta de teste
        let test_uuid = "00000000-0000-0000-0000-000000000001";
        let test_account = r#"{
            "username": "usuario_teste",
            "password": "senha123",
            "strategy": "337.lua",
            "server": "10000",
            "nickname": "Conta Teste"
        }"#;
        
        table.insert(test_uuid, test_account)?;
        println!("Conta de teste adicionada ao banco de dados!");
    }
    write_txn.commit()?;
    
    println!("Banco de dados inicializado com sucesso!");
    Ok(())
}
