use aws_sdk_s3::Client;
use color_eyre::Result;
use clap::Parser;
use aws_smithy_http::byte_stream::ByteStream;

static S3_BUCKET_NAME: &'static str = "some-random-bucket-name-1231241-q234scvx";


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_path: String,

    #[arg(short, long)]
    object_name: String
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = Args::parse();
    let config = aws_config::from_env().region("us-east-1").load().await;
    let client = Client::new(&config);

    upload_file(&client, &args).await?;

    Ok(())
}

async fn upload_file(client: &Client, args: &Args) -> Result<()> {
    let stream = ByteStream::from_path(&args.file_path).await?;

    client.put_object()
        .bucket(S3_BUCKET_NAME)
        .key(&args.object_name)
        .body(stream)
        .send().await?;

    Ok(())
}