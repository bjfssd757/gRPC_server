use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    std::env::var("OUT_DIR")?;

    tonic_build::compile_protos("proto/control.proto")?;
    tonic_build::compile_protos("proto/calendar.proto")?;

    Ok(())
}