# mdbook-git
A preprocessor for [mdBook](https://github.com/rust-lang/mdBook) that allows embedding Git commit files and diffs directly into your book.

## Getting started

First, install the `mdbook-git` binary:

```sh
cargo install mdbook-git
```

Then, add the preprocessor configuration to your `book.toml` with the path to the Git repository:

```toml
[preprocessor.git]
# path to the Git repository
path = "path/to/repo"
```

## Embedding Files

Similar to the [built-in mdBook link preprocessor](https://rust-lang.github.io/mdBook/format/mdbook.html#including-files), you can embed a file from a specific commit using the following syntax:

````markdown
```rust
{{ #git show 409a0091e1b14c4a64af91b19dc405ab78f32862:src/main.rs }}
```
````

This displays the contents of `src/main.rs` of the specified commit.

## Embedding Diffs

To show the difference between two commits, use the following syntax:

````markdown
```diff
{{ #git diff c702619b19462b2bff877076a01333fd974613fe 409a0091e1b14c4a64af91b19dc405ab78f32862 src/main.rs }}
```
````

### Diff Options

The `diff` command supports the following options:

* `-h`: Hides the header and removed lines, improving focus on additions and surrounding context.
* ``-U[lines]``: Sets the number of context lines shown around changes. This mirrors Git's -U option.

## Hiding Lines Initially

Similar to [rustdoc_include](https://rust-lang.github.io/mdBook/format/mdbook.html#including-a-file-but-initially-hiding-all-except-specified-lines) feature of mdBook, you can display only specific lines while hiding the rest:

```markdown
# show line 2, hide all others
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs:2 }}

# show all lines ..4
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs:4: }}

# show all lines 4..
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs::4 }}

# show all lines 2..4
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs:2:4 }}
```

Unlike `mdBook`, this preprocessor also supports **multiple ranges**:

```markdown
# show line 2 and 4
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs:[2,4] }}

# show lines 2, 4..8 and 12..
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs:[2,4:8,12:] }}
```

**Note:** Do not include spaces in the array.

**Important:** mdBook anchors are not supported.

## Example

See this [mdBook on ray tracing](https://goldnor.github.io/rt-in-one-weekend/) to view the preprocessor in action:

* [Example chapter using `show` and `diff`](https://goldnor.github.io/rt-in-one-weekend/chapters/the_vec3_class/color_utility_functions.html)
* [Markdown source of that chapter](https://github.com/goldnor/rt-books/blob/main/books/ray-tracing-in-one-weekend/src/chapters/the_vec3_class/color_utility_functions.md)

To preserve Rust syntax highlighting in diffs, a custom highlighting language called `rust-diff` is defined. You can copy the relevant [theme files](https://github.com/goldnor/rt-books/tree/main/books/ray-tracing-in-one-weekend/theme) into the `theme` directory next to your `book.toml`.

To support other languages in a diff context, adapt the `highlight.js` file accordingly.

