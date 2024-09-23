# rust_codestyle
![Minimum Supported Rust Version](https://img.shields.io/badge/nightly-1.83+-ab6000.svg)
[<img alt="crates.io" src="https://img.shields.io/crates/v/rust_codestyle.svg?color=fc8d62&logo=rust" height="20" style=flat-square>](https://crates.io/crates/rust_codestyle)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs&style=flat-square" height="20">](https://docs.rs/rust_codestyle)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/valeratrades/rust_codestyle/ci.yml?branch=master&style=for-the-badge&style=flat-square" height="20">](https://github.com/valeratrades/rust_codestyle/actions?query=branch%3Amaster) <!--NB: Won't find it if repo is private-->
![Lines Of Code](https://img.shields.io/badge/LoC-224-lightblue)

A GA plug to check for properties requiring full `syn` parsing.

Can find missing `//SAFETY` comments on unsafe blocks, for example. For more: `--help`

FUCK: I've spent like an hour on it, so it'll probably not work as expected 50% of the time.

<!-- markdownlint-disable -->
<details>
  <summary>
    <h2>Installation</h2>
  </summary>
	<pre><code class="language-sh">TODO</code></pre>
</details>
<!-- markdownlint-restore -->

## Usage
```sh
rust_codestyle ~/place/with/rust/code --instrument # other flags don't work rn
```

<br>

<sup>
This repository follows <a href="https://github.com/valeratrades/.github/tree/master/best_practices">my best practices</a>.
</sup>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
