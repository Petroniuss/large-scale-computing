use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ScheduleJobRequestBody<'a> {
    pub(crate) host: &'a str,
    pub(crate) script: &'a str,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct SubmitJobResponseBody {
    pub(crate) job_id: String,
    stdout_path: String,
    stderr_path: String,
    status: Status,

    #[serde(default)]
    tag: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct GetJobInfoResponseBody {
    job_id: String,
    stdout_path: String,
    stderr_path: String,
    pub(crate) status: Status,

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
pub(crate) enum Status {
    QUEUED,
    FINISHED,
    RUNNING,
}

pub(crate) fn submit_job(
    request_body: &ScheduleJobRequestBody,
    client: &Client,
    proxy_header: &str,
) -> SubmitJobResponseBody {
    let request_body_json = serde_json::to_string_pretty(&request_body).unwrap();
    println!("Request to POST: {}", request_body_json);

    let response = client
        .post("https://submit.plgrid.pl/api/jobs")
        .headers(default_headers(proxy_header))
        .body(request_body_json)
        .send()
        .unwrap();

    parse_response(response)
}

pub(crate) fn get_job_info(
    job_id: &str,
    client: &Client,
    proxy_header: &str,
) -> GetJobInfoResponseBody {
    let endpoint = format!("https://submit.plgrid.pl/api/jobs/{job_id}");
    println!("Request to GET: {}", endpoint);

    let response = client
        .get(endpoint)
        .headers(default_headers(proxy_header))
        .send()
        .unwrap();

    parse_response(response)
}

pub(crate) fn download_file(
    remote_file_path: &str,
    local_file_name: &str,
    client: &Client,
    proxy_header: &str,
) {
    let endpoint = format!("https://data.plgrid.pl/download/ares/{remote_file_path}");
    println!("Request to GET: {}", endpoint);

    let response = client
        .get(endpoint)
        .headers(default_headers(proxy_header))
        .send()
        .unwrap();

    let status = response.status();
    if status.is_success() {
        let path = Path::new(local_file_name);
        let mut file = File::create(&path).unwrap();

        let bytes = response.bytes().unwrap();
        file.write_all(bytes.as_ref()).unwrap();
    } else {
        failure::<Value>(response, status);
    }
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

pub(crate) fn default_headers(proxy_header: &str) -> HeaderMap {
    return HeaderMap::from_iter([
        (CONTENT_TYPE, HeaderValue::from_static("application/json")),
        (
            HeaderName::from_str(&"PROXY").unwrap(),
            HeaderValue::from_str(proxy_header).unwrap(),
        ),
    ]);
}
