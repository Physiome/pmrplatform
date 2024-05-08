use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let cwd = env::current_dir()?;
    print!("{{");
    print!(r#""args":{args:?},"#);
    print!(r#""cwd":{:?}"#, cwd.display());
    print!("}}");
    Ok(())
}
