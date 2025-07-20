# mdbook-git
Insert git commit files and diffs into [mdBook](https://github.com/rust-lang/mdBook).

## Getting started

First install the mdbook-git binary

```sh
cargo install mdbook-git
```

and add the preprocessor to your `book.toml` with the path to the repository:

```toml
[preprocessor.git]
# path to git repo
path = "path/to/repo"
```

Now, similar to the [mdBook buildin link preprocessor](https://rust-lang.github.io/mdBook/format/mdbook.html#including-files), the preprocessor looks for files to include

````markdown
```rust
{{ #git show 409a0091e1b14c4a64af91b19dc405ab78f32862:src/main.rs }}
```
````

Here the content of `src/main.rs` of the specified commit is displayed.
If you want to show the diff between two commits, you can use this instead

````markdown
```diff
{{ #git diff c702619b19462b2bff877076a01333fd974613fe 409a0091e1b14c4a64af91b19dc405ab78f32862 src/main.rs }}
```
````

This command allows for two options:
* `-h` to hide the header and deletion; allows for better readability of the additions and the context around it
* `-U[lines]` sets the number of lines of the context around additions and deletions; this is a standard git commnand

## Hide lines initially

Similar to [rustdoc_include](https://rust-lang.github.io/mdBook/format/mdbook.html#including-a-file-but-initially-hiding-all-except-specified-lines) of `mdBook`, the lines to show initially (and hide the others) can be annotated with

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

In contrast to `mdBook`, however, you can specify arrays of ranges:

```markdown
# show line 2 and 4
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs:[2,4] }}
# show lines 2, 4..8 and 12..
{{ #git show c702619b19462b2bff877076a01333fd974613fe:src/main.rs:[2,4:8,12:] }}
```

**NOTE:** Do not use spaces in the array.

**ALSO NOTE:** Anchors are not supported!

## Example

See [this](https://goldnor.github.io/rt-in-one-weekend/) mdBook about raytracing for ray tracing to see the preprocessor in action:

* [Example chapter displaying show and diff](https://goldnor.github.io/rt-in-one-weekend/chapters/the_vec3_class/color_utility_functions.html)
* [Markdown of that chapter](https://github.com/goldnor/rt-books/blob/main/books/ray-tracing-in-one-weekend/src/chapters/the_vec3_class/color_utility_functions.md)

I defined the custom highlighting language `rust-diff`, so that Rust code is still highlighted while beeing in a diff context. You can these [files](https://github.com/goldnor/rt-books/tree/main/books/ray-tracing-in-one-weekend/theme) to the `theme` directory next to the `book.toml` to introduce better highlighting. For other languages you have to can follow my implementation in the `highlight.js` file.



