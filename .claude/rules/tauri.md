---
paths: "src/**/*"
---

# Tauri Plugin Rules

### Rust Commands
- **Async for all I/O**: Use async for file/network ops, sync only for fast CPU ops
- **Error handling**: Use thiserror + Serialize for all command errors
- **State**: `app.manage(Mutex::new(...))`, access via `State<'_, T>`

### Error Pattern
```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
```

### Performance
```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Security
- Principle of least privilege per window
- Validate all command args in Rust
- Use scopes for fs access
