use std::{
    ops::{Bound, RangeBounds},
    path::Path,
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{Context, anyhow};
use git2::{DiffLineType, DiffOptions, Oid, Repository};
use mdbook::{BookItem, preprocess::Preprocessor};
use regex::{Captures, Regex};

#[derive(Default, Debug)]
pub struct Git {}

impl Preprocessor for Git {
    fn name(&self) -> &str {
        "git"
    }

    // {{ #git diff [<options>] [commit_old] [commit_new] [file][:start:end] }}
    // {{ #git show [commit]:[file][:start:end] }}
    fn run(
        &self,
        ctx: &mdbook::preprocess::PreprocessorContext,
        mut book: mdbook::book::Book,
    ) -> anyhow::Result<mdbook::book::Book> {
        let default_repo = ctx
            .config
            .get_preprocessor(self.name())
            .and_then(|cfg| cfg.get("path"))
            .and_then(|val| val.as_str())
            .map(Path::new)
            .and_then(|path| ctx.root.join(path).canonicalize().ok())
            .map(|path| {
                Repository::open(&path)
                    .with_context(|| format!("Could not find repository at {:?}", path))
            })
            .transpose()?;

        let src_dir = ctx.root.join(&ctx.config.book.src);

        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                if let Some(ref chapter_path) = ch.path {
                    let base = chapter_path
                        .parent()
                        .map(|dir| src_dir.join(dir))
                        .expect("All book items have a parent");

                    let content =
                        replace_all(&ch.content, base, chapter_path, default_repo.as_ref());
                    ch.content = content;
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn replace_all(
    s: &str,
    _path: impl AsRef<Path>,
    _source: impl AsRef<Path>,
    default_repo: Option<&Repository>,
) -> String {
    let Some(repo) = default_repo else {
        return s.to_owned();
    };

    let mut replaced = String::new();
    let mut previous_end_index = 0;

    // simply skip invalid cmds
    for cmd in find_git_cmds(s).filter_map(|captures| GitCmd::try_from(captures).ok()) {
        let GitCmd { typ, start, end } = cmd;

        replaced.push_str(&s[previous_end_index..start]);

        match typ {
            GitType::Show { id, path, range } => {
                if let Ok(contents) = git_show(id, path, range, repo) {
                    replaced.push_str(&contents);
                    previous_end_index = end;
                }
            }
            GitType::Diff {
                old,
                new,
                path,
                range,
                options,
            } => {
                if let Ok(contents) = git_diff(old, new, path, range, options, repo) {
                    replaced.push_str(&contents);
                    previous_end_index = end;
                }
            }
        }
    }

    replaced.push_str(&s[previous_end_index..]);
    replaced
}

fn git_show(
    id: &str,
    path: &str,
    range: impl RangeBounds<usize>,
    repo: &Repository,
) -> anyhow::Result<String> {
    let id = Oid::from_str(id)?;
    let commit = repo.find_commit(id)?;

    let tree = commit.tree()?;
    let entry = tree.get_path(std::path::Path::new(path))?;

    let object = entry.to_object(&repo)?;
    let blob = object
        .as_blob()
        .ok_or_else(|| anyhow!("Commit does not contain this file."))?;

    std::str::from_utf8(blob.content())
        .map(|s| take_lines_comment_out_rest(s, range))
        .map_err(Into::into)
}

pub fn take_lines_comment_out_rest(s: &str, range: impl RangeBounds<usize>) -> String {
    let mut lines: Vec<String> = s.lines().map(ToOwned::to_owned).collect();

    for (i, line) in lines.iter_mut().enumerate() {
        if !range.contains(&i) && !line.starts_with("# ") {
            *line = format!("# {line}");
        }
    }

    lines.join("\n")
}

pub fn parse_path_and_range<T: FromStr + Copy>(
    path_and_range: &str,
) -> Option<(&str, (Bound<T>, Bound<T>))> {
    match path_and_range.split(':').collect::<Vec<_>>().as_slice() {
        &[path] => Some((path, (Bound::<T>::Unbounded, Bound::<T>::Unbounded))),
        &[path, line] => {
            let line = line.parse().ok()?;
            Some((path, (Bound::Included(line), Bound::Included(line))))
        }
        &[path, "", ""] => Some((path, (Bound::Unbounded, Bound::<T>::Unbounded))),
        &[path, start, ""] => Some((
            path,
            (Bound::Included(start.parse().ok()?), Bound::<T>::Unbounded),
        )),
        &[path, "", end] => Some((
            path,
            (
                Bound::<T>::Unbounded,
                Bound::<T>::Excluded(end.parse().ok()?),
            ),
        )),
        &[path, start, end] => Some((
            path,
            (
                Bound::<T>::Included(start.parse().ok()?),
                Bound::<T>::Excluded(end.parse().ok()?),
            ),
        )),
        _ => None,
    }
}

fn git_diff(
    old: &str,
    new: &str,
    path: &str,
    range: impl RangeBounds<usize>,
    options: Vec<&str>,
    repo: &Repository,
) -> anyhow::Result<String> {
    let old_commit = repo.find_commit(Oid::from_str(old)?)?;
    let new_commit = repo.find_commit(Oid::from_str(new)?)?;

    let old_tree = old_commit.tree()?;
    let new_tree = new_commit.tree()?;

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(path);

    // handle options
    if let Some(number_context_lines) = options
        .iter()
        .find_map(|item| item.starts_with("-U").then(|| item[2..].parse()))
        .transpose()?
    {
        diff_opts.context_lines(number_context_lines);
    }

    // special non-git option
    let hide_header_and_deletion = options.iter().any(|item| item.starts_with("-h"));

    let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut diff_opts))?;
    let mut str = String::new();

    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let mut content = std::str::from_utf8(line.content())
            .expect("utf8 expected")
            .to_owned();

        content = content
            .lines()
            .map(|content| {
                format!(
                    "{}{}{content}\n",
                    (hide_header_and_deletion
                        && !matches!(
                            line.origin_value(),
                            DiffLineType::Addition | DiffLineType::Context
                        ))
                    .then(|| "# ")
                    .unwrap_or_default(),
                    (matches!(
                        line.origin_value(),
                        DiffLineType::Addition | DiffLineType::Deletion | DiffLineType::Context
                    ))
                    .then(|| line.origin().to_string())
                    .unwrap_or_default()
                )
            })
            .collect::<Vec<String>>()
            .join("");

        str.push_str(&content);

        true
    })?;

    Ok(take_lines_comment_out_rest(&str, range))
}

#[derive(Debug)]
struct GitCmd<'a> {
    typ: GitType<'a>,
    start: usize,
    end: usize,
}

impl<'a> TryFrom<Captures<'a>> for GitCmd<'a> {
    type Error = ();

    fn try_from(value: Captures<'a>) -> Result<Self, Self::Error> {
        let (start, end) = value.get(0).map(|cmd| (cmd.start(), cmd.end())).ok_or(())?;

        GitType::try_from(value).map(|typ| GitCmd { typ, start, end })
    }
}

#[derive(Debug)]
enum GitType<'a> {
    Show {
        id: &'a str,
        path: &'a str,
        range: (Bound<usize>, Bound<usize>),
    },
    Diff {
        old: &'a str,
        new: &'a str,
        path: &'a str,
        range: (Bound<usize>, Bound<usize>),
        options: Vec<&'a str>,
    },
}

impl<'a> TryFrom<Captures<'a>> for GitType<'a> {
    type Error = ();

    fn try_from(value: Captures<'a>) -> Result<Self, Self::Error> {
        let Some(mut subcmd) = value
            .get(1)
            .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>())
        else {
            return Err(());
        };

        // move all options to the end
        subcmd.sort_unstable_by(|a, b| a.starts_with('-').cmp(&b.starts_with('-')));

        let cmd = match subcmd.as_slice() {
            &["show", id_and_path_and_range, ..] => id_and_path_and_range
                .split_once(":")
                .map(|(id, path_and_range)| {
                    parse_path_and_range(path_and_range).map(|(path, range)| GitType::Show {
                        id,
                        path,
                        range,
                    })
                })
                .flatten(),
            &["diff", old, new, path_and_range, ref options @ ..] => {
                // needs to be owned, sinced they got resorted
                // and would not be contigous in memory
                let options = options.to_owned();

                parse_path_and_range(path_and_range).map(|(path, range)| GitType::Diff {
                    old,
                    new,
                    path,
                    range,
                    options,
                })
            }
            _ => None,
        };

        cmd.ok_or(())
    }
}

fn find_git_cmds(contents: &str) -> impl Iterator<Item = Captures<'_>> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?x)         # insignificant whitespace mode
              \{\{\s*      # link opening bracket and whitespace
              \#git        # git command
              \s+          # seperating whitespace
              ([^}]+)      # everything except the closing bracket
              \}\}         # closing bracket
             ",
        )
        .unwrap()
    });

    RE.captures_iter(contents)
}
