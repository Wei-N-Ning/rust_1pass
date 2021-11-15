use lib_rust_1pass::make_session;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sess = make_session("immu").await?;
    let values = sess.item_fields(
        "shared-aws-nonprod",
        &["Access key ID", "Secret access key"],
    )?;
    println!("{:?}", values);
    Ok(())
}
