fn main() -> anyhow::Result<()> {
    vergen::EmitBuilder::builder().git_sha(false).emit()
}
