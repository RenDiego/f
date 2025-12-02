use std::process::{Command, ExitStatus};

use clap::{Args, Parser};
use color_eyre::eyre::Result;
use colored::Colorize;
use scraper::{Html, Selector};

#[derive(Parser)]
#[command(name = "program")]
#[command(about = "A package helper utility", long_about = None)]
struct Cli {
    /// Search for a package
    #[arg(value_name = "SEARCH_PACKAGE")]
    search_package: Option<String>,

    #[command(flatten)]
    actions: Option<Actions>,
}

#[derive(Args, Clone)]
struct Actions {
    /// Install a package, the short name 'S' is inspired by the AUR helper 'yay'. TODO: Change this name if too confusing.
    #[arg(short = 'S', value_name = "INSTALL_PACKAGE")]
    install: Option<String>,

    /// Remove a package
    #[arg(short = 'r', value_name = "REMOVE_PACKAGE")]
    remove: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(ref actions) = cli.actions {
        if let Some(ref package_to_remove) = actions.remove {
            println!("Removing package: {}", package_to_remove);
            Command::new("sudo")
                .arg("dnf")
                .arg("install")
                .arg(package_to_remove)
                .arg("-y")
                .status()?;
        }

        if let Some(ref package_to_install) = actions.install {
            install(package_to_install)?;
        }
    }

    if let Some(ref search_package) = cli.search_package {
        println!("Searching for package: {}\n", search_package);
        let response = reqwest::get(format!(
            "https://packages.fedoraproject.org/search?query={}",
            search_package
        ))
        .await?
        .text()
        .await?
        .replace("\n", "")
        .replace("\t", "");

        let html = Html::parse_document(&response);

        let selector = Selector::parse("div.row > div.col-md-8").unwrap();

        if let Some(col_element) = html.select(&selector).next() {
            /* from HTML response:
             * <div class=\"position-relative\">
             *   <div class=\"h5 m-0 new-block\"><a>PACKAGE_NAME</a> - PACKAGE_DESCRIPTION</div>
             *   <span><a>View other packages from PACKAGE_NAME &raquo;</a></span> // <-- we don't need this
             * </div>
             */
            let content_selector = Selector::parse("div.position-relative div.new-block").unwrap();
            let mut packages: Vec<String> = Vec::new();

            for (index, element) in col_element.select(&content_selector).enumerate() {
                let text = element.text().collect::<String>();
                let names = text.split(" - ").collect::<Vec<&str>>();
                packages.push(names[0].to_owned());
                println!("{}. {} - {}", index + 1, names[0].blue().bold(), names[1]);
            }

            let mut package_to_install = String::new();

            while package_to_install.is_empty() {
                println!("Select the package you want installed: ");
                let mut entered_string = String::new();
                std::io::stdin()
                    .read_line(&mut entered_string)
                    .expect("Reading line failed");

                if let Ok(r) = entered_string.trim().parse::<usize>() {
                    if let Some(package) = packages.get(r - 1) {
                        package_to_install = package.to_owned();
                    }
                }
            }

            install(&package_to_install)?;
        }
    }

    if cli.actions.is_none() && cli.search_package.is_none() {
        println!("Updating the packages");
        Command::new("sudo")
            .arg("dnf")
            .arg("update")
            .arg("-y")
            .status()?;
    }

    Ok(())
}

fn install(package_to_install: &String) -> Result<ExitStatus> {
    println!("Installing package: {}", package_to_install);
    Ok(Command::new("sudo")
        .arg("dnf")
        .arg("install")
        .arg(package_to_install)
        .arg("-y")
        .status()?)
}
