use core::fmt;
use std::thread;

use serde::Deserialize;
use tokio::task;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

// We get the response from this one, but it is blocking.
#[get("/api2")]
async fn make_api_call2() -> String {
    let ops = vec![1, 2, 3];
    let mut tasks = Vec::with_capacity(ops.len());
    for op in ops {
        // This call will make them start running in the background
        // immediately.
        tasks.push(tokio::spawn(my_background_op(op)));
    }

    let mut outputs = Vec::with_capacity(tasks.len());
    for task in tasks {
        outputs.push(task.await.unwrap());
    }
    println!("my outputs {:?}", outputs);

    "Done with API calls 2".to_string()
}
async fn my_background_op(id: i32) -> String {
    let s = format!("Starting background task {}.", id);
    //sleep for 5 seconds
    thread::sleep(std::time::Duration::from_secs(5));
    println!("{}", s);
    s
}

//This one doesnt block, but we dont get the response
#[get("/api")]
async fn make_api_call() -> String {
    let list_of_apis = [
        String::from("https://dummyjson.com/products/1"),
        String::from("https://dummyjson.com/products/2"),
        String::from("https://dummyjson.com/products/3"),
    ];

    let resp1 = task::spawn(request(5));
    let resp2 = task::spawn(request(1));

    //If you uncomment these, it blocks the resp until completion
    // let _ = resp1.await.unwrap();
    // let _ = resp2.await.unwrap();

    "Done with API calls".to_string()
}

fn slowwly(delay_s: u32) -> reqwest::Url {
    let url = format!("https://hub.dummyapis.com/delay?seconds={}", delay_s,);
    reqwest::Url::parse(&url).unwrap()
}
async fn request(n: u32) {
    reqwest::get(slowwly(n)).await.unwrap();
    info!("Got response {}", n);
}

#[derive(Deserialize)]
struct APIResponse {
    id: i32,
    title: String,
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![make_api_call])
        .mount("/", routes![make_api_call2])
}
