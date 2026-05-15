use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("mihomo.bin");

    if dest.exists() && dest.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
        return; // already cached
    }

    let target = env::var("TARGET").unwrap();
    let (platform, ext, is_zip) = if target.contains("windows") {
        ("windows-amd64", ".zip", true)
    } else if target.contains("linux") && target.contains("aarch64") {
        ("linux-arm64", ".gz", false)
    } else if target.contains("linux") {
        ("linux-amd64", ".gz", false)
    } else {
        println!("cargo:warning=unsupported target: {}", target);
        fs::write(&dest, b"").ok();
        return;
    };

    let version = "v1.18.10";
    let asset = format!("mihomo-{}-{}{}", platform, version, ext);
    let url = format!(
        "https://github.com/MetaCubeX/mihomo/releases/download/{}/{}",
        version, asset
    );

    println!("cargo:warning=Downloading {}...", asset);
    let archive_path = out_dir.join(&asset);

    let dl_ok = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                    url,
                    archive_path.display()
                ),
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        Command::new("curl")
            .args(["-sL", &url, "-o"])
            .arg(&archive_path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };

    if !dl_ok {
        println!(
            "cargo:warning=Download failed. Place mihomo binary at {:?}",
            dest
        );
        fs::write(&dest, b"").ok();
        return;
    }

    if is_zip {
        let file = match fs::File::open(&archive_path) {
            Ok(f) => f,
            Err(_) => {
                fs::write(&dest, b"").ok();
                return;
            }
        };
        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => {
                fs::write(&dest, b"").ok();
                return;
            }
        };
        for i in 0..archive.len() {
            if let Ok(mut entry) = archive.by_index(i) {
                let name = entry.name().to_lowercase();
                if name.ends_with(".exe") || name == "mihomo" {
                    let mut buf = Vec::new();
                    if std::io::copy(&mut entry, &mut buf).is_ok() && !buf.is_empty() {
                        fs::write(&dest, buf).ok();
                        println!(
                            "cargo:warning=Extracted mihomo.exe ({} bytes)",
                            dest.metadata().map(|m| m.len()).unwrap_or(0)
                        );
                        let _ = fs::remove_file(&archive_path);
                        return;
                    }
                }
            }
        }
    } else {
        // gz
        let data = match fs::read(&archive_path) {
            Ok(d) => d,
            Err(_) => {
                fs::write(&dest, b"").ok();
                return;
            }
        };
        let mut decoder = flate2::read::GzDecoder::new(&data[..]);
        let mut buf = Vec::new();
        if std::io::Read::read_to_end(&mut decoder, &mut buf).is_ok() && !buf.is_empty() {
            fs::write(&dest, buf).ok();
            println!(
                "cargo:warning=Extracted mihomo ({} bytes)",
                dest.metadata().map(|m| m.len()).unwrap_or(0)
            );
            let _ = fs::remove_file(&archive_path);
            return;
        }
    }

    println!("cargo:warning=Extraction failed");
    fs::write(&dest, b"").ok();
}
