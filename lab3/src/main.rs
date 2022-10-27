mod rimrock_client;

extern crate core;

use std::{env, thread, time};
use reqwest::blocking::Client;
use crate::rimrock_client::{ScheduleJobRequestBody, Status};

// todo fix the script, add modules.
// todo upload the file to ares.

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
        let submit_job_response = rimrock_client::submit_job(
            &ScheduleJobRequestBody {
                host: "ares.cyfronet.pl",
                script: sample_script,
            },
            &client,
            &proxy,
        );

        loop {
            let response = rimrock_client::get_job_info(&submit_job_response.job_id, &client, &proxy);

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
