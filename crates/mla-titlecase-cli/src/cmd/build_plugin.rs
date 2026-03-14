use crate::{
    cli::{BuildPluginArgs, OutputFormat},
    error::Result,
    manifest::load_prepared,
    plugin_build,
};

pub(crate) fn run(args: BuildPluginArgs) -> Result<()> {
    let prepared = load_prepared(&args.input)?;
    let plugin = plugin_build::build_plugin(&prepared);

    match args.format {
        OutputFormat::Json => mla_titlecase::json_store::save_json_plugin(&args.output, &plugin)?,
        OutputFormat::Fst => mla_titlecase::fst_store::save_fst_plugin(&args.output, &plugin)?,
    }

    println!(
        "built {} plugin with {} entries at {}",
        match args.format {
            OutputFormat::Json => "json",
            OutputFormat::Fst => "fst",
        },
        plugin.payload.len(),
        args.output.display()
    );
    Ok(())
}
