Wayk Now Rust
=============

Collection of Rust crates for Wayk Now.

## Crates

### wayk_proto

Provides basic Wayk Now packet encoder-decoder and sequencing utilities.

### wayk_proto_derive

Provides derive macros for Encode and Decode traits from wayk_proto.

## wayk_core

Provides data structures and other utilities used by Wayk products not specific to protocol itself.

### wayk_cli_client

A basic Wayk Now CLI client to demonstrate wayk_proto usage.

## Contributing guidelines

- Make a separate branch/fork for your modifications.

- Do your modifications.

- Make sure your code builds and tests are green:

    ```
    $ cargo test
    ```

- Check clippy lints for code quality:

    ```
    $ cargo clippy
    ```

    See instructions [here](https://github.com/rust-lang/rust-clippy) if you don't have clippy yet.

- Reformat your code using rustfmt:

    ```
    $ cargo fmt
    ```

    See instructions [here](https://github.com/rust-lang/rustfmt) if you don't have rustfmt yet.

- **Make sure you pull latest modifications** for master.

- Rebase on top of master so we can
    [fast forward](https://help.github.com/en/github/administering-a-repository/about-merge-methods-on-github#rebasing-and-merging-your-commits)
    merge:

    ```
    $ git rebase origin/master
    ```

    Assuming your origin remote points on the main repository.

- Send your pull request! Thanks!
