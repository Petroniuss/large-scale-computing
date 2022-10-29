mod ares_client;

use crate::ares_client::{download_file, ScheduleJobRequestBody, Status};
use reqwest::blocking::Client;
use std::time::Duration;
use std::{env, thread, time};

fn main() {
    if let Some(proxy) = env::var_os("PROXY") {
        let proxy = proxy.to_str().unwrap();

        let sample_script = "\
            #!/bin/bash
            #SBATCH -A plglscclass-cpu \n\
            #SBATCH -p plgrid \n\
            #SBATCH -N 1 \n\
            #SBATCH --ntasks-per-node=4 \n\
            module add xvfb blender \n\
            xvfb-run -a blender \
                --background -noaudio ./large-scale-computing/lab3/halloween_spider.blend \
                --render-output ./frame_1.png --render-frame 1 \n\
            exit 0\
        ";

        let client = Client::new();
        let submit_job_response = ares_client::submit_job(
            &ScheduleJobRequestBody {
                host: "ares.cyfronet.pl",
                script: sample_script,
            },
            &client,
            &proxy,
        );

        let mut total_time_waiting = Duration::from_secs(0);
        loop {
            let next_duration = time::Duration::from_secs(30);
            thread::sleep(next_duration);
            total_time_waiting += next_duration;

            println!("Waited for: {}s", total_time_waiting.as_secs());
            let response = ares_client::get_job_info(&submit_job_response.job_id, &client, &proxy);

            match response.status {
                Status::FINISHED | Status::ERROR => {
                    println!("{:#?}", response);
                    break;
                }
                _ => { }
            }
        }

        let local_file_name = "frame.png";
        let remote_file_path = "/net/people/plgrid/plgpwojtyczek/frame_1.png0001.png";
        download_file(remote_file_path, local_file_name, &client, &proxy);
        println!("Downloaded file: {} as {}", remote_file_path, local_file_name);
    } else {
        panic!("$PROXY variable must be set to a valid certificate.");
    }
}
