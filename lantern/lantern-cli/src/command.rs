pub trait Command {
    fn execute(&self) -> anyhow::Result<()>;
}
