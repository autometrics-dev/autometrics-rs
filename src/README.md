# Autometrics 📈✨

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

**Autometrics is a macro that makes it trivial to add useful metrics to any function in your codebase.**

Easily understand and debug your production system using automatically generated queries. Autometrics adds links to Prometheus charts directly into each function's doc comments.

(Coming Soon!) Autometrics will also generate dashboards ([#15](https://github.com/fiberplane/autometrics-rs/issues/15)) and alerts ([#16](https://github.com/fiberplane/autometrics-rs/issues/16)) from simple annotations in your code. Implementations in other programming languages are also in the works!

### 1️⃣ Add `#[autometrics]` to any function or `impl` block

```rust
#[autometrics]
async fn create_user(Json(payload): Json<CreateUser>) -> Result<Json<User>, ApiError> {
  // ...
}

#[autometrics]
impl Database {
  async fn save_user(&self, user: User) -> Result<User, DbError> {
    // ...
  }
}
```

### 2️⃣ Hover over the function name to see the generated queries

![VS Code Hover Example](vs-code-example)

### 3️⃣ Click a query link to go directly to the Prometheus chart for that function

![Prometheus Chart](prometheus-chart)

### 4️⃣ Go back to shipping features 🚀
