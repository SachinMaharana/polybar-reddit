# polybar-reddit

```rust
RUSTFLAGS='-C target-cpu=native' cargo build --release
```

```rust
[dependencies]
anyhow = "1.0.32"
dotenv = "0.15.0"
serde = {version = "1.0.116",  features = ["derive"]}
ureq = { version = "*", features = ["json", "charset"] }
jemallocator = "0.3.2"


[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"
```