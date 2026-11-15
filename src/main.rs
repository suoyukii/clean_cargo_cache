use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

type Item = (u64, PathBuf);

fn parse_crate_name(file_name: &str) -> String {
    let stem = file_name.strip_suffix(".crate").unwrap_or(file_name);
    match stem.rsplit_once('-') {
        Some((name, ver)) if ver.chars().next().map_or(false, |c| c.is_ascii_digit()) => {
            name.to_string()
        }
        _ => stem.to_string(),
    }
}

#[cfg(windows)]
fn file_time(p: &Path) -> u64 {
    use std::os::windows::fs::MetadataExt;
    fs::metadata(p).unwrap().creation_time()
}

#[cfg(unix)]
fn file_time(p: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(p).unwrap().ctime() as u64
}

fn clean_dir(root: &Path, is_target: impl Fn(&Path) -> bool, remove: impl Fn(&Path)) {
    let rd = fs::read_dir(root).unwrap();
    for sub in rd {
        let sub_dir = sub.unwrap().path();
        if !sub_dir.is_dir() {
            continue;
        }
        let mut map: HashMap<String, Vec<Item>> = HashMap::new();
        for e in fs::read_dir(&sub_dir).unwrap() {
            let p = e.unwrap().path();
            if !is_target(&p) {
                continue;
            }
            let name = p.file_name().unwrap().to_str().unwrap();
            let crate_name = parse_crate_name(name);
            let time = file_time(&p);
            map.entry(crate_name).or_default().push((time, p));
        }
        for (_, mut items) in map {
            if items.len() <= 1 {
                continue;
            }
            items.sort_by(|a, b| b.0.cmp(&a.0));
            for (_, p) in &items[1..] {
                remove(p);
            }
        }
    }
}

fn main() {
    #[cfg(windows)]
    let home_path = env::var("USERPROFILE").unwrap();
    #[cfg(unix)]
    let home_path = env::var("HOME").unwrap();

    let reg = Path::new(&home_path).join(".cargo").join("registry");

    clean_dir(
        &reg.join("cache"),
        |p| p.is_file(),
        |p| {
            fs::remove_file(p).unwrap();
        },
    );

    clean_dir(
        &reg.join("src"),
        |p| p.is_dir(),
        |p| {
            fs::remove_dir_all(p).unwrap();
        },
    );

    println!("Cleanup done");
}
