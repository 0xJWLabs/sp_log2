# sp_log2

## A simple and easy way to log for Rust's crate

`sp_log2` does not aim to provide a rich set of features, nor to provide the
best logging solution. It aims to be a maintainable, easy to integrate facility
for small to medium sized projects. In those cases `sp_log2` should provide an
easy alternative.

## Concept
`sp_log2` provides a series of logging facilities, that can be easily combined.

- `SimpleLogger` (very basic logger that logs to stderr/out, should never fail)
- `TermLogger` (advanced terminal logger, that splits to stderr/out and has color support) (can be excluded on unsupported platforms)
- `WriteLogger` (logs to a given struct implementing `Write`. e.g. a file)
- `CombinedLogger` (can be used to form combinations of the above loggers)

## Usage
```rust
#[macro_use] extern crate log;
extern crate sp_log2;

use sp_log2::*;

use std::fs::File;

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();

    error!("Bright red error");
    info!("This only appears in the log file");
    debug!("This level is currently not enabled for any logger");
}

```

### Results in
```
$ cargo run --example usage
   Compiling sp_log2 v0.1.0 (file:///home/jo/dev/projects/rust/sp_log2)
     Running `target/debug/examples/usage`
[ERROR] Bright red error
```
and my_rust_binary.log
```
11:13:03 [ERROR] usage: Bright red error
11:13:03 [INFO] usage: This only appears in the log file
```

## Getting Started

Just add
```
[dependencies]
sp_log2 = "^0.1.0"
```
to your `Cargo.toml`

## ANSI color and style support

This crate can internally depend on a [paris](https://github.com/0x20F/paris) crate to provide support for ANSI color and styles.
To use this feature you need to set a _paris_ feature, like this:
```
[dependencies]
sp_log2 = { version = "^0.1.0", features = ["paris"] }
```
in your `Cargo.toml`

After this you can use e.g. the following call:
```rust
info!("I can write <b>bold</b> text or use tags to <red>color it</>");
```

This will automatically generates terminal control sequences for desired styles.

More formatting info: [paris crate documentation](https://github.com/0x20F/paris)
