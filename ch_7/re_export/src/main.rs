pub mod config;

use crate::config::db;
use crate::config::server;

fn main() {
    db::Config::db_config();
    server::Config::srv_config();
}
