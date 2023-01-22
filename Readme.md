# Valar

> **Note**
> This is an experimental, developer-centric async Rust web framework. This is a Proof Of Concept project not meant to be used in any real applications.

> **Warning**
> Under heavy development. APIs are subject to change.

```rs
use serde::Serialize;
use std::sync::Arc;
use tokio_postgres::Error as PGError;
use valar::database::builder::Whereable;
use valar::database::Database;
use valar::database::Executor;
use valar::database::Row;
use valar::http::Server;
use valar::http::Request;
use valar::http::Response;
use valar::http::Result;
use async_trait::async_trait;
use valar::Application;
use valar::Error;
use valar::routing::Routable;
use valar::routing::Router;

pub struct App {
    pub database: Database,
}

#[async_trait]
impl Application for App {
    async fn create() -> Result<Self, Error> {
        let app = Self {
            database: Database::connect("host=localhost user=erik dbname=valar").await?,
        };

        Ok(app)
    }
}

impl Routable for App {
    type Application = App;

    fn router() -> Router<Self::Application> {
        let mut router = Router::default();

        router.get("/", index);
        router.post("/", create);
        router.get("/:id", show);
        router.patch("/:id", update);
        router.delete("/:id", delete);

        router
    }
}

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

async fn index(app: Arc<App>, _request: Request) -> Result {
    let rows: Vec<User> = Database::table("users")
        .select_all()
        .get(&app.database)
        .await?;

    Response::ok().json(&rows)?.produce()
}

async fn show(app: Arc<App>, request: Request) -> Result {
    let id: i32 = request.parameter("id")?;

    let user: User = Database::table("users")
        .select_all()
        .where_equal("id", &id)
        .first(&app.database)
        .await?;

    Response::ok().json(&user)?.produce()
}

async fn create(app: Arc<App>, _request: Request) -> Result {
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

async fn update(app: Arc<App>, request: Request) -> Result {
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

async fn delete(app: Arc<App>, request: Request) -> Result {
    let id: i32 = request.parameter("id")?;

    Database::query("DELETE FROM users WHERE id=$1")
        .with(&id)
        .execute(&app.database)
        .await?;

    Response::no_content().produce()
}

#[tokio::main]
async fn main() {
    let app = App::create().await.unwrap();

    Server::builder()
        .address(([127, 0, 0, 1], 8080))
        .build()
        .start(app)
        .await;
}

#[cfg(test)]
mod tests {
  #[tokio::test]
  async fn text_it_has_json() {
      let app = App::fake().await.unwrap();
      let response = app.get("/").send().await;

      response.assert_ok().assert_is_json();
  }
}
```
