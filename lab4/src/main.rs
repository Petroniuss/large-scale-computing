use color_eyre::eyre::eyre;
use color_eyre::Result;


#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Hello, world!");

    let x = None;
    let y = x.ok_or(eyre!("foo"))?;

    Ok(())
}
