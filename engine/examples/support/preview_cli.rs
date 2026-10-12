#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub baseline: String,
    pub seed: u64,
    pub autoplay: bool,
    pub trace: bool,
}
pub fn parse(args: &[String]) -> Result<Option<Options>, String> {
    if args == ["--help"] {
        return Ok(None);
    }
    let mut baseline = None;
    let mut seed = None;
    let mut autoplay = false;
    let mut trace = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--baseline" if baseline.is_none() => {
                baseline = Some(args.next().ok_or("missing baseline path")?.clone())
            }
            "--seed" if seed.is_none() => {
                seed = Some(
                    args.next()
                        .ok_or("missing seed")?
                        .parse::<u64>()
                        .map_err(|_| "seed must be an unsigned integer")?,
                )
            }
            "--autoplay" if !autoplay => autoplay = true,
            "--trace" if !trace => trace = true,
            _ => return Err(format!("unknown or repeated option: {arg}")),
        }
    }
    Ok(Some(Options {
        baseline: baseline.ok_or("--baseline PATH is required")?,
        seed: seed.unwrap_or(1),
        autoplay,
        trace,
    }))
}
