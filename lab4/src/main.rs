use color_eyre::Result;
use std::time::Duration;

static AMI_ID: &'static str = "ami-05bfb9c51e8c55557";

use aws_sdk_ec2::{
    model::{Instance, InstanceStateName, InstanceStatus, InstanceType, SummaryStatus},
    Client,
};
use color_eyre::eyre::eyre;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = aws_config::from_env().region("us-east-1").load().await;
    let client = Client::new(&config);

    // see what we've got.
    show_all_instances(&client).await?;

    // spawn the instance
    let spawned_instance_id = spawn_instance(&client).await?;
    println!("Spawned instance: {}", spawned_instance_id);
    poll_until(&client, &spawned_instance_id, |e| {
        let state = e.instance_state().and_then(|x| x.name());
        let status = e.instance_status().and_then(|x| x.status());

        state == Some(&InstanceStateName::Running) && status == Some(&SummaryStatus::Ok)
    })
    .await?;

    // fetch index
    let spawned_instance = describe_instance(&client, &spawned_instance_id).await?;
    let ip_address = spawned_instance.public_ip_address().unwrap();
    let url = format!("http://{}", ip_address);
    let response_body = reqwest::get(&url).await?.text().await?;
    println!("GET {}", url);
    println!("{}", response_body);

    // at this point we shuld terminate the instance
    terminate_instance(&client, &spawned_instance_id).await?;
    poll_until(&client, &spawned_instance_id, |e| {
        let state = e.instance_state().and_then(|x| x.name());

        state == Some(&InstanceStateName::Terminated)
    })
    .await?;

    // see what we've got.
    show_all_instances(&client).await?;

    Ok(())
}

async fn terminate_instance(client: &Client, instance_id: &str) -> Result<()> {
    client
        .terminate_instances()
        .instance_ids(instance_id)
        .send()
        .await?;

    Ok(())
}

async fn poll_until(
    client: &Client,
    instance_id: &str,
    predicate: fn(&InstanceStatus) -> bool,
) -> Result<Duration> {
    let mut measured_time = Duration::ZERO;
    loop {
        let sleep_time = Duration::from_secs(5);
        println!("Sleeping for {}s..", sleep_time.as_secs());
        sleep(sleep_time).await;
        measured_time += sleep_time;

        let instance_status = describe_instance_status(&client, instance_id).await?;

        let instance_id = instance_status.instance_id().unwrap();
        let state = instance_status.instance_state().unwrap().name().unwrap();
        let status = instance_status.instance_status().unwrap().status().unwrap();

        println!(
            r#"
            Instance ID: {},
            State      : {},
            Status     : {},
        "#,
            instance_id,
            state.as_str(),
            status.as_str()
        );

        if predicate(&instance_status) {
            println!("Took {}s.", measured_time.as_secs());
            break;
        }
    }

    Ok(measured_time)
}

async fn spawn_instance(client: &Client) -> Result<String> {
    client
        .run_instances()
        .image_id(AMI_ID)
        .instance_type(InstanceType::T2Micro)
        .min_count(1)
        .max_count(1)
        .send()
        .await?
        .instances()
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .instance_id()
        .map(|x| x.to_string())
        .ok_or(eyre!("Failed to retrieve instance_id"))
}

async fn describe_instance_status(client: &Client, instance_id: &str) -> Result<InstanceStatus> {
    let result = client
        .describe_instance_status()
        .instance_ids(instance_id)
        .send()
        .await?;

    let foo = result
        .instance_statuses();

    println!("{:?}", foo);

    let bar = foo
        .unwrap()
        .into_iter()
        .next()
        .clone()
        .unwrap()
        .clone();

    Ok(bar)
}

async fn describe_instance(client: &Client, instance_id: &str) -> Result<Instance> {
    Ok(client
        .describe_instances()
        .instance_ids(instance_id)
        .send()
        .await?
        .reservations()
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .instances()
        .into_iter()
        .next()
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .clone())
}

async fn show_all_instances(client: &Client) -> Result<()> {
    let resp = client.describe_instances().send().await?;

    println!("EC2 Instances:");
    for reservation in resp.reservations().unwrap_or_default() {
        for instance in reservation.instances().unwrap_or_default() {
            println!("Instance ID: {}", instance.instance_id().unwrap());
            println!(
                "State:       {:?}",
                instance.state().unwrap().name().unwrap()
            );
            println!();
        }
    }
    println!("--------------------------------------------------");

    Ok(())
}
