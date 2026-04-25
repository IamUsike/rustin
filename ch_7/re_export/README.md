**hard**

**Config module with re-exports**

Create a config module with sub-modules for database and server settings. Use `pub use` to re-export the most common types at the top level so users can do `use myapp::Config` instead of `use myapp::config::server::Config`.

_Hint: In config/mod.rs: pub use self::server::ServerConfig; pub use self::db::DbConfig;_
