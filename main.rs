use std::{fs, path::Path, io};
use tera::{Context, Tera};
use serde_json;

fn build_templates() {
    // Use globbing
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let mut context = Context::new();
    context.insert("title",&"Tekken 8 Frame Data - Hwoarang");
    let file = fs::read_to_string("data/hwoarang.json").expect("Unable to read file");
    let data: serde_json::Value = serde_json::from_str(&file).expect("JSON was not well-formatted");
    context.insert("data", &data);
    context.insert("displayCharacter", "Hwoarang");
    let rendered = tera.render("page.html", &context).expect("rendering failed");
    let _ = fs::write("build/page.html", rendered);
    let _ = copy_dir_all("static", "build");
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
// let data = octocrab::instance().pulls("FrostemanNeogard", "TekkenFramedataAPI").get(5).await?;
    if cmd == "build" {
        build_templates();
    } else {
        print!("invalid command")
    }
}