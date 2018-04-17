# Yet Another Derive Error (Yade)
[![](https://img.shields.io/crates/v/yade.svg)](https://crates.io/crates/yade) [![](https://travis-ci.org/lholden/yade.svg?branch=master)](https://travis-ci.org/lholden/yade)

Yade is a set of derive macros designed to generate the boilerplate for the `Error` trait in Rust. This crate supports `enum`, `struct`, unit, tuple struct, and ErrorKind styled Error types.

Yade was designed to work with the `Error` trait in the Rust standard library and does not set out to solve any of the problems with said trait. This allows Yade to be drop in compatible with the current ecosystem. If you are interested in an alternative trait for error handling that attempts to address the shortcomings of `Error` by replacing it with a brand new trait, please check out the [Failure](https://github.com/withoutboats/failure) project.

## Usage

Be sure to add the yade crate to your `Cargo.toml`:

```toml
[dependencies]
yade = "*"
```

### Derive Macros
Yade provides two derive macros, `YadeError` ad `YadeKind`.

`YadeError` is the primary macro and can be used to automatically derive the `Error` and `Display` traits for many kinds of error structs.

For example:

```rust
#[macro_use] extern crate yade;
use std::fs::File;
use std::error::Error as StdError;

#[derive(Debug, YadeError)]
pub enum Error {
    #[display(msg = "File could not be found: {}", _0)]
    FileNotFound(String, #[cause] std::io::Error),

    #[display(msg = "Bad request: {}", params)]
    BadRequest {
        params: String,
    },
}

fn causes_error() -> Result<(), Error> {
    File::open("a_missing_file.txt")
        .map_err(|e| Error::FileNotFound("a_missing_file.txt".into(), e))?;

    Ok(())
}

fn main() {
    match causes_error() {
      Err(err) => {
          eprintln!("An error occured: {}", err);

          let mut err: &StdError = &err;
          while let Some(cause) = err.cause() {
              eprintln!(" - Cause: {}", cause);
              err = cause;
          }
      },
      _ => println!("Great success"),
    }
}
```

`YadeKind` is provided to assist in building error types using an ErrorKind enum.

```rust
#[macro_use] extern crate yade;
use std::fs::File;
use std::error::Error as StdError;

#[derive(Debug, YadeError)]
#[display(msg = "{}", kind)]
pub struct Error {
    pub kind: ErrorKind,

    #[cause]
    pub cause: Option<Box<StdError>>,
}

#[derive(Debug, YadeKind)]
pub enum ErrorKind {
    #[display(msg = "File could not be found: {}", _0)]
    FileNotFound(String),

    #[display(msg = "World fell apart, oops")]
    AnotherError,
}

fn causes_error() -> Result<(), Error> {
    File::open("a_missing_file.txt")
        .map_err(|e| Error { kind: ErrorKind::FileNotFound("a_missing_file.txt".into()), cause: Some(Box::new(e)) })?;

    Ok(())
}

fn main() {
    match causes_error() {
        Err(err) => {
            eprintln!("An error occured: {}", err);

            let mut err: &StdError = &err;
            while let Some(cause) = err.cause() {
                eprintln!(" - Cause: {}", cause);
                err = cause;
            }
        },

        _ => println!("Great success"),
    }
}
```


## FAQ
##### Why not just use 'failure'?
[failure](https://github.com/withoutboats/failure) by withoutboats tries to solve some of the problems with Rusts `Error` trait by introducing a new trait. Unlike 'failure', Yade is directly compatible with the standard `Error` trait in Rust. As such, Yade is designed to be a drop-in replacement that maintains compatibility with existing code.

##### Why not just use 'error-chain'?
I have used [error-chain](https://github.com/rust-lang-nursery/error-chain) in several projects since it's inception and while I do think it's a useful tool for building applications, I do not feel it fits in well for libraries.

My main complaint is that 'chain-chain' complicates pattern matching. I also find the macro based DSLs like the one employed by 'chain-error' are confusing to the error when dealing with syntax errors. (There is the [derive-error-chain](https://github.com/Arnavion/derive-error-chain) project, but I feel that it suffers from some of it's own issues.)

## Similar Libraries

* [failure](https://github.com/withoutboats/failure) - A new error management story.
* [derive-error](https://github.com/rushmorem/derive-error) - Derive macro for Error using macros 1.1.
* [error-chain](https://github.com/rust-lang-nursery/error-chain) - Yet another error boilerplate library.

## License

Yade is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
    http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
    http://opensource.org/licenses/MIT)

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

Please see the [CONTRIBUTING](CONTRIBUTING.md) file for more information.
