pub trait Command {
    async fn execute(&self) -> anyhow::Result<()>;
}
