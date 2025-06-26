use anyhow::Context;
use inquire::Confirm;
use minijinja::{
    Environment,
    context,
};
use regex::Regex;
use std::{
    env,
    error::Error,
    fs::write,
    path::PathBuf,
};

// FIXME: Look at error handling

// TODO: Add skip-overwrite flag
// TODO: Add dry-run flag
// TODO: Add interactive flag
const INTERACTIVE: bool = true;
// TODO: Add force flag
const FORCE: bool = false;
// TODO: Iterate over all files in directory

fn main() -> Result<(), Box<dyn Error>> {
    let re = Regex::new(r"S(?<season>\d+)E(?<episode>\d+) (?<title>.+)$").unwrap();

    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);
    let template = env
        .get_template("template.xml")
        .expect("should be able to load embedded template");

    let args: Vec<String> = env::args().collect();
    for path in &args[1..] {
        let path = PathBuf::from(path);
        let mut target_file = path.clone();
        target_file.set_extension("nfo");

        let Some(file_stem) = path.file_stem() else {
            eprintln!(
                "Cannot generate .nfo file for `{}` because the file name is empty, skipping.",
                path.display()
            );
            continue;
        };

        let Some(caps) = re.captures(file_stem.to_str().expect("file name should be valid UTF-8"))
        else {
            eprintln!(
                "Cannot generate .nfo file for `{}` because captures are missing, skipping.",
                path.display()
            );
            continue;
        };
        let context = context! {
            title => caps["title"],
            season => caps["season"],
            episode => caps["episode"],
        };

        if INTERACTIVE {
            eprintln!(
                "Going to create `{}` with the following content:",
                target_file.display()
            );
            eprintln!(
                "{}",
                template.render(&context).with_context(|| format!(
                    "Failed to render template file with context: {context}"
                ))?
            );

            let write = Confirm::new("Write")
                .with_default(true)
                .with_help_message("There is another prompt in case a file would be overwritten")
                .prompt();

            if let Ok(false) | Err(_) = write {
                continue;
            }
        }

        if !FORCE {
            let target_file_exists = target_file.try_exists();

            match target_file_exists {
                Ok(true) => {
                    let overwrite =
                        Confirm::new(&format!("`{}` exists. Overwrite", target_file.display()))
                            .with_default(false)
                            .prompt();

                    if let Ok(false) | Err(_) = overwrite {
                        continue;
                    }
                }
                Err(_) => {
                    let overwrite = Confirm::new(&format!(
                        "Error when detecting whether `{}` exists. Overwrite anyways",
                        target_file.display(),
                    ))
                    .with_default(false)
                    .prompt();

                    if let Ok(false) | Err(_) = overwrite {
                        continue;
                    }
                }
                Ok(false) => {}
            }

            write(target_file, template.render(&context).unwrap())
                .expect("should be able to write file");
        }
    }

    Ok(())
}
