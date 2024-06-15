use serde_json;
use std::process::Command;
use std::{fs, io, path::Path};
use tera::{Context, Tera};

const REPO_URL: &str = "https://github.com/FrostemanNeogard/TekkenFramedataAPI.git";
const DATA_DIR: &str = "data/TekkenFramedataAPI/src/__data/tekken8";

struct Character {
    id: String,
    name: String,
    moves: serde_json::Value,
}

fn to_character_name(str: &str) -> String {
    let mut last_char_was_space = true;
    let mut chars:Vec<char> = Vec::new();
    for char in str.chars() {
        if last_char_was_space {
            chars.push(char.to_ascii_uppercase());
        } else {
            chars.push(char);
        }
        last_char_was_space = char == ' ';
    }
    chars.iter().collect()
}

fn parse_frame_data() -> Vec<Character> {
    let mut characters: Vec<Character> = Vec::new();
    let dirs =
        fs::read_dir(DATA_DIR).expect("failed to read dir");
    for file in dirs {
        let file_path_buf = file.expect("File not found").path();
        let file_path = file_path_buf.as_path();
        let file_stem = file_path
            .file_stem()
            .expect("File stem could not be parsed")
            .to_str()
            .expect("File stem could not be converted to string");

        let file_json = fs::read_to_string(&file_path).expect("Unable to read file");

        let moves: serde_json::Value =
            serde_json::from_str(&file_json).expect("JSON was not well-formatted");
        let character = Character {
            id: String::from(file_stem),
            name: to_character_name(file_stem),
            moves,
        };
        characters.push(character);
    }
    characters
}

fn build_templates() {
    let characters = parse_frame_data();
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    // Render character pages
    for character in characters {
        let mut context = Context::new();
        context.insert("data", &character.moves);
        context.insert("name", &character.name);
        let output_file_path = "build/".to_owned() + &character.id + &".html";
        let rendered = tera
            .render("page.html", &context)
            .expect("rendering failed");
        fs::write(&output_file_path, rendered).expect("Character page write failed");
    }

    copy_dir_all("static", "build").expect("Static dir copy failed");
}

fn pull_latest_frame_data() {
    let _ = fs::remove_dir_all("data/");
    let _ = fs::create_dir("data/");
     Command::new("git")
        .arg("clone")
        .arg("--depth=1")
        .arg(REPO_URL)
        .current_dir("data/")
        .spawn()
        .expect("git clone failed")
        .wait().expect("git clone never ran");
}

// https://stackoverflow.com/a/65192210
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    let cmd = std::env::args().nth(1).expect("no command given");
    if cmd == "build" {
        build_templates();
    } else if cmd == "pull" {
        pull_latest_frame_data();
    } else {
        print!("invalid command")
    }
}
