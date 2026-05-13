use crate::error::HornetError;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

fn compute_hash(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

fn which(cmd: &str) -> bool {
    if cfg!(windows) {
        if let Ok(path) = std::env::var("PATH") {
            for entry in path.split(';') {
                let candidate = Path::new(entry).join(cmd);
                if candidate.exists() {
                    return true;
                }
                let candidate_exe = Path::new(entry).join(format!("{}.exe", cmd));
                if candidate_exe.exists() {
                    return true;
                }
            }
        }
        false
    } else {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

fn find_linker() -> Option<String> {
    for candidate in &[
        "clang",
        "gcc",
        "cc",
        "musl-gcc",
    ] {
        if which(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

fn find_llc() -> Option<String> {
    for candidate in &[
        "llc",
        "llc-18",
        "llc-17",
        "llc-16",
        "llc-15",
    ] {
        if which(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

pub fn build_native(source_path: &str, source: &str, ir: &str, release: bool, emit_ir: bool) -> Result<String, HornetError> {
    let hash = compute_hash(source);
    let hash = format!("{:08x}", hash);

    let tmp_dir = if cfg!(windows) {
        std::env::temp_dir()
    } else {
        PathBuf::from("/tmp")
    };

    let tmp_ll = tmp_dir.join(format!("hornet_{}.ll", hash));
    let tmp_obj = tmp_dir.join(format!("hornet_{}.o", hash));
    let binary_name = Path::new(source_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| HornetError::Other("Invalid filename".into()))?;
    let output_path = PathBuf::from(format!("./{}", binary_name));

    fs::write(&tmp_ll, ir)?;
    if emit_ir {
        let emit_path = Path::new(source_path).with_extension(format!("{}.ll", source_path.split('.').last().unwrap_or("hn")));
        fs::write(&emit_path, ir)?;
    }

    let llc = find_llc().ok_or_else(|| HornetError::Other("llc not found. Install LLVM: https://llvm.org/releases/".into()))?;
    let mut llc_cmd = Command::new(&llc);
    llc_cmd.args(&["-filetype=obj", "-o", tmp_obj.to_str().unwrap(), tmp_ll.to_str().unwrap()]);
    if release {
        llc_cmd.arg("-O2");
    }
    let llc_status = llc_cmd.status()?;
    if !llc_status.success() {
        let _ = fs::remove_file(&tmp_ll);
        return Err(HornetError::Other("llc failed: LLVM IR compilation error".into()));
    }

    let linker = find_linker().ok_or_else(|| HornetError::Other("No C compiler found. Install clang or gcc.".into()))?;
    let mut link_cmd = Command::new(&linker);
    link_cmd.args(&[tmp_obj.to_str().unwrap(), "-o", output_path.to_str().unwrap()]);
    if release {
        link_cmd.args(&["-O2", "-s"]);
    }
    let link_status = link_cmd.status()?;
    if !link_status.success() {
        let _ = fs::remove_file(&tmp_ll);
        let _ = fs::remove_file(&tmp_obj);
        return Err(HornetError::Other("Linking failed".into()));
    }

    let _ = fs::remove_file(&tmp_ll);
    let _ = fs::remove_file(&tmp_obj);
    Ok(output_path.to_str().unwrap().to_string())
}
