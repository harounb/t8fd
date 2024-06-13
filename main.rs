use std::{fs, path::Path, io};
use std::process::Command;
use tera::{Context, Tera};
use serde_json;

fn build_templates() {
    let dirs = fs::read_dir("data/TekkenFramedataAPI/src/__data/tekken8").expect("failed to read dir");
    // Use globbing
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    for file in dirs {
        let mut context = Context::new();
        let mut file_path:String = "data/TekkenFramedataAPI/src/__data/tekken8/".to_owned();
        let file_name = file.unwrap().file_name().into_string().unwrap();
        file_path = file_path + &file_name;
        context.insert("title",&("Tekken 8 Frame Data - ".to_owned() + &file_name));
        let file_json = fs::read_to_string(&file_path).expect("Unable to read file");
        let data: serde_json::Value = serde_json::from_str(&file_json).expect("JSON was not well-formatted");
        context.insert("data", &data);
        context.insert("displayCharacter", &file_name);
        let output_file_path = "build/".to_owned() + &file_name + &".html";
        let rendered = tera.render("page.html", &context).expect("rendering failed");
        let _ = fs::write(&output_file_path, rendered);
    }
   
    let _ = copy_dir_all("static", "build");
}

fn pull_latest_frame_data() {
let _ = fs::remove_dir_all("data/");
let _ = fs::create_dir("data/");
Command::new("git")
.arg("clone")
.arg("--depth=1")
.arg("https://github.com/FrostemanNeogard/TekkenFramedataAPI.git")
.current_dir("data/")
.spawn()
.expect("git clone failed");
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
    } else if cmd =="pull" {
        pull_latest_frame_data();
    } else {
        print!("invalid command")
    }
}