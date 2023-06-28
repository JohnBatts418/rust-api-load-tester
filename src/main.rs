use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use tokio::task;
extern crate redis;
use async_recursion::async_recursion;
use redis::Commands;
#[macro_use]
extern crate rocket;

#[derive(Deserialize)]
struct SastAPIResponse {
    status: String,
}

#[derive(Deserialize, Clone)]
struct Repo {
    target: String,
    hash: String,
}

#[derive(Serialize)]
struct LoadTestStatus {
    success: i32,
    failures: i32,
    total: i32,
}

#[get("/api")]
async fn make_api_call() -> String {
    //read in github.json file
    // let file = File::open("github.json").unwrap();
    // let reader = BufReader::new(file);
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    let mut file = File::open("repo_lists/github.json").unwrap();
    let mut buff = String::new();
    file.read_to_string(&mut buff).unwrap();

    let foo: Vec<Repo> = serde_json::from_str(&buff).unwrap();
    println!("Target is: {}", foo[0].target);
    println!("Hash is : {}", foo[0].hash);

    for api in foo.iter() {
        task::spawn(trigger_analysis(api.clone()));
    }

    String::from("Triggered API calls...")
}

#[get("/status/<id>")]
async fn get_status(id: String) -> Json<LoadTestStatus> {
    // connect to redis
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    let length_of_success: i32 = con.scard("run1Success").unwrap();
    let length_of_failures: i32 = con.scard("run1failures").unwrap();

    let mut total: i32 = 0;
    total += length_of_success;
    total += length_of_failures;

    let status = LoadTestStatus {
        success: length_of_success,
        failures: length_of_failures,
        total,
    };
    Json(status)
}

async fn trigger_analysis(single_repo_req: Repo) {
    let body = poll_api(&single_repo_req).await;

    println!(
        "Finished polling for {}, got status {}",
        single_repo_req.hash, body.status
    );

    // connect to redis
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    if body.status == "SUCCESS" {
        println!("Analysis succeeded for {}", single_repo_req.hash);
        let _: () = con.sadd("run1Success", single_repo_req.target).unwrap();
    } else {
        println!("Analysis failed for {}", single_repo_req.hash);
        let _: () = con.sadd("run1failures", single_repo_req.target).unwrap();
    }
}

#[async_recursion]
async fn poll_api(single_repo_req: &Repo) -> SastAPIResponse {
    let body = reqwest::get(&single_repo_req.target)
        .await
        .unwrap()
        .json::<SastAPIResponse>()
        .await
        .unwrap();

    println!(
        "Got response for hash {} with status {}",
        single_repo_req.hash, body.status
    );

    //if the status is waiting then wait 2 seconds and try again
    if body.status == "WAITING" {
        println!("Waiting for analysis to complete...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        poll_api(single_repo_req).await
    } else {
        body
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![make_api_call])
        .mount("/", routes![get_status])
}
