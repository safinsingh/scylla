use clap::Clap;

#[derive(Clap)]
#[clap(version = "1.0", author = "Safin S. <safin.singh@gmail.com>")]
pub struct Opts {
	#[clap(subcommand)]
	pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
	#[clap(version = "1.0", author = "Safin S. <safin.singh@gmail.com>")]
	/// Prepare SQL database for Scylla
	Prepare,
	#[clap(version = "1.0", author = "Safin S. <safin.singh@gmail.com>")]
	/// Start the engine
	Start,
}
