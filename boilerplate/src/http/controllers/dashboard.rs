use std::sync::Arc;

use serde::Serialize;
use tokio_postgres::Error as PGError;
use valar::database::builder::Whereable;
use valar::database::Database;
use valar::database::Executor;
use valar::database::Row;
use valar::drivers::cache::Cache;
use valar::http::Request;
use valar::http::Response;
use valar::http::Result;

use crate::App;

#[derive(Debug, Serialize)]
struct User {
    id: i32,
    name: String,
}

impl TryFrom<Row> for User {
    type Error = PGError;

    fn try_from(row: Row) -> std::result::Result<Self, Self::Error> {
        let user = Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
        };

        Ok(user)
    }
}

pub async fn index(app: Arc<App>, _request: Request) -> Result {
    app.cache
        .map_or_default("views", |views: u16| views + 1)
        .await?;

    let rows: Vec<User> = Database::table("users")
        .select_all()
        .get(&app.database)
        .await?;

    Response::ok().json(&rows)?.produce()
}

pub async fn show(app: Arc<App>, request: Request) -> Result {
    let id: i32 = request.parameter("id")?;

    let user: User = Database::table("users")
        .select_all()
        .where_equal("id", &id)
        .first(&app.database)
        .await?;

    Response::ok().json(&user)?.produce()
}

pub async fn create(app: Arc<App>, _request: Request) -> Result {
    let user = User {
        id: 3,
        name: "Hello".to_string(),
    };

    Database::query("INSERT INTO users (id, name) VALUES ($1, $2)")
        .parameters([&user.id, &user.name])
        .execute(&app.database)
        .await?;

    Response::created().json(&user)?.produce()
}

pub async fn update(app: Arc<App>, request: Request) -> Result {
    let user = User {
        id: request.parameter("id")?,
        name: "Super".to_string(),
    };

    Database::query("UPDATE users SET name=$1 WHERE id=$2")
        .parameters([&user.name, &user.id])
        .execute(&app.database)
        .await?;

    Response::ok().json(&user)?.produce()
}

pub async fn delete(app: Arc<App>, request: Request) -> Result {
    let id: i32 = request.parameter("id")?;

    Database::query("DELETE FROM users WHERE id=$1")
        .with(&id)
        .execute(&app.database)
        .await?;

    Response::no_content().produce()
}
