use std::{env, fs, io::Write, path::Path};

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    args.next();
    let len = args
        .next()
        .map(|s| fs::read_to_string(s).expect("file not found"))
        .expect("missing 1 required argument")
        .len();
    let mut writer = args
        .next()
        .map(|s| {
            Box::new(
                fs::File::create(Path::new::<str>(s.as_ref()).join("size"))
                    .expect("unable to open file for writing"),
            ) as Box<dyn Write>
        })
        .unwrap_or_else(|| Box::new(std::io::stdout()) as Box<dyn Write>);
    writer.write(format!("{len}").as_bytes())?;
    Ok(())
}
