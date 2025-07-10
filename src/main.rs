mod preprocessor;

use clap::{Arg, ArgMatches, Command};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use preprocessor::Git;
use semver::{Version, VersionReq};
use std::io;
use std::process;

fn main() {
    let matches = make_app().get_matches();

    let pre = Git::default();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&pre, sub_args);
    } else if let Err(e) = handle_preprocessing(&pre) {
        eprintln!("{e:?}");
        process::exit(1);
    }
}

pub fn make_app() -> Command {
    Command::new("mdbook-git")
        .about("mdbook preprocessor for displaying git diffs")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn handle_supports(pre: &impl Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);

    process::exit(if supported { 0 } else { 1 });
}

fn handle_preprocessing(pre: &impl Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
