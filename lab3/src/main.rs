extern crate core;

use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::{env, thread, time};

#[derive(Deserialize, Serialize, Debug)]
struct ScheduleJobRequestBody<'a> {
    host: &'a str,
    script: &'a str,
}

#[derive(Deserialize, Serialize, Debug)]
struct SubmitJobResponseBody {
    job_id: String,
    stdout_path: String,
    stderr_path: String,
    status: Status,

    #[serde(default)]
    tag: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct GetJobInfoResponseBody {
    job_id: String,
    stdout_path: String,
    stderr_path: String,
    status: Status,

    #[serde(default)]
    tag: Option<String>,

    #[serde(default)]
    nodes: Option<String>,

    #[serde(default)]
    cores: Option<String>,

    #[serde(default)]
    start_time: Option<String>,

    #[serde(default)]
    end_time: Option<String>,

    #[serde(default)]
    wall_time: Option<String>,

    #[serde(default)]
    queue_time: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
enum Status {
    QUEUED,
    FINISHED,

    RUNNING,
}

fn submit_job(
    request_body: &ScheduleJobRequestBody,
    client: &Client,
    proxy_header: &str,
) -> SubmitJobResponseBody {
    let request_body_json = serde_json::to_string_pretty(&request_body).unwrap();
    println!("Request to /api/jobs: {}", request_body_json);

    let response = client
        .post("https://submit.plgrid.pl/api/jobs")
        .headers(default_headers(proxy_header))
        .body(request_body_json)
        .send()
        .unwrap();

    parse_response(response)
}

fn get_job_info(job_id: &str, client: &Client, proxy_header: &str) -> GetJobInfoResponseBody {
    let endpoint = format!("https://submit.plgrid.pl/api/jobs/{job_id}");
    println!("Request to /api/jobs: {}", endpoint);

    let response = client
        .get(endpoint)
        .headers(default_headers(proxy_header))
        .send()
        .unwrap();

    parse_response(response)
}

fn parse_response<T>(response: Response) -> T
where
    T: DeserializeOwned,
{
    let status = response.status();
    if status.is_success() {
        success(response)
    } else {
        failure(response, status)
    }
}

fn success<T>(response: Response) -> T
where
    T: DeserializeOwned,
{
    let success_body = response.json::<Value>().unwrap();
    println!("Request success: {:#?}", success_body);

    serde_json::from_value(success_body).unwrap()
}

fn failure<T>(response: Response, status: StatusCode) -> T
where
    T: DeserializeOwned,
{
    let failure_body = response.json::<Value>().unwrap();
    println!(
        "Request failure - {:#?} : {:#?}",
        status.to_string(),
        failure_body
    );

    panic!();
}

fn default_headers(proxy_header: &str) -> HeaderMap {
    return HeaderMap::from_iter([
        (CONTENT_TYPE, HeaderValue::from_static("application/json")),
        (
            HeaderName::from_str(&"PROXY").unwrap(),
            HeaderValue::from_str(proxy_header).unwrap(),
        ),
    ]);
}

fn main() {
    if let Some(proxy) = env::var_os("PROXY") {
        let proxy = proxy.to_str().unwrap();

        let sample_script = "\
            #!/bin/bash \n\
            #SBATCH -A plglscclass-cpu \n\
            #SBATCH -p plgrid \n\
            #SBATCH -N 1\n\
            #SBATCH --ntasks-per-node=1 \n\
            #SBATCH -t 00:01:00 \n\
            sleep 5 \n\
            exit 0\
        ";

        let client = Client::new();
        let submit_job_response = submit_job(
            &ScheduleJobRequestBody {
                host: "ares.cyfronet.pl",
                script: sample_script,
            },
            &client,
            &proxy,
        );

        loop {
            let response = get_job_info(&submit_job_response.job_id, &client, &proxy);

            if response.status == Status::FINISHED {
                println!("{:#?}", response);
                break;
            }

            thread::sleep(time::Duration::from_secs(2));
        }
    } else {
        panic!("$PROXY variable must be set to a valid certificate.");
    }
}
