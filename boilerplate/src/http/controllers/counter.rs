use std::sync::Arc;

use valar::http::Request;
use valar::http::Response;
use valar::http::Result;

use crate::App;

pub async fn show(request: Request<App>) -> Result {
    let count = 1;
    // let app = request.app();
    // let session = request.session()?;

    // let count: i32 = request
    //     .session()?
    //     .get("count")
    //     .on(&app.cache)
    //     .await
    //     .unwrap_or(String::from("0"))
    //     .parse()?;

    Response::ok()
        .html(format!(
            r#"
                <h1>Counter</h1>
                <p>Count: {count}</p>
                <form method="POST">
                    <button type="submit">Increment</button>
                </form>
            "#
        ))
        .into_ok()
}

pub async fn increment(request: Request<App>) -> Result {
    // let session = request.session()?;

    // let count: i32 = session
    //     .get("count")
    //     .on(&app.cache)
    //     .await
    //     .unwrap_or(String::from("0"))
    //     .parse()?;

    // session
    //     .set("count", (count + 1).to_string())
    //     .on(&app.cache)
    //     .await?;

    Response::redirect("/").into_ok()
}
