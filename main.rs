use serde::{Deserialize, Serialize};
use serde_json;
use std::process::{exit, Command};
use std::{fs, io, path::Path};
use tera::{Context, Tera};

const DATA_DIR: &str = "data";
const TEKKEN_8_DATA_DIRECTORY: &str = "data/TekkenFramedataAPI/src/__data/tekken8";
const BUILD_FOLDER: &str = "build";
const REPO_URL: &str = "https://github.com/FrostemanNeogard/TekkenFramedataAPI.git";

struct Character {
    id: String,
    name: String,
    moves: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct CharacterLink {
    url: String,
    name: String,
}

fn to_character_name(str: &str) -> String {
    let mut last_char_was_space = true;
    let mut chars: Vec<char> = Vec::new();
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
    let dirs = fs::read_dir(TEKKEN_8_DATA_DIRECTORY).expect("failed to read dir");
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
    let mut character_links: Vec<CharacterLink> = Vec::new();
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    // Render character pages
    for character in &characters {
        let mut context = Context::new();
        context.insert("data", &character.moves);
        context.insert("name", &character.name);
        let output_file_path = String::from("") + BUILD_FOLDER + &character.id + &".html";
        let rendered = tera
            .render("[character].html", &context)
            .expect("Character page rendering failed");
        fs::write(&output_file_path, rendered).expect("Character page write failed");
    }

    let mut context = Context::new();
    // Render index page
    for character in characters {
        let character_link = CharacterLink {
            url: String::from("") + &character.id,
            name: character.name,
        };
        character_links.push(character_link)
    }
    context.insert("data", &character_links);
    let rendered = tera
        .render("index.html", &context)
        .expect("Index page rendering failed");
    fs::write(String::from("") + BUILD_FOLDER + "/index.html", rendered)
        .expect("Index page write failed");

    copy_dir_all("static", BUILD_FOLDER).expect("Static dir copy failed");
}

fn pull_latest_frame_data() {
    let _ = fs::remove_dir_all(DATA_DIR);
    let _ = fs::create_dir(DATA_DIR);
    Command::new("git")
        .arg("clone")
        .arg("--depth=1")
        .arg(REPO_URL)
        .current_dir(DATA_DIR)
        .spawn()
        .expect("git clone failed")
        .wait()
        .expect("git clone never ran");
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

    match cmd.as_str() {
        "build" => build_templates(),
        "pull" => pull_latest_frame_data(),
        _ => {
            eprintln!("invalid command");
            exit(1);
        }
    }
}
