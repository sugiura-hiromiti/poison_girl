#![feature(exit_status_error)]
use {
	colored::Colorize,
	poison_girl_dev_error::{PoisonGirlB, X},
	std::{
		ffi::OsStr,
		process::{Command, Stdio},
	},
};

pub trait Run
{
	/// コマンドを実行し、出力を親プロセスにバイパスする
	fn run(&mut self,) -> PoisonGirlB<(),>;
}

impl Run for Command
{
	fn run(&mut self,) -> PoisonGirlB<(),>
	{
		// Format the command display string with program and arguments
		let cmd_dsply = format!(
			"{} {}",
			self.get_program().display(),
			self.get_args()
				.collect::<Vec<&OsStr,>>()
				.join(OsStr::new(" "))
				.display()
		);

		// Display the command in bold blue for visibility
		println!("\n{}", cmd_dsply.bold().blue());

		// Configure stdio inheritance and execute the command
		let out = self
			.stdout(Stdio::inherit(),)  // Inherit stdout for real-time output
			.stderr(Stdio::inherit(),)  // Inherit stderr for error messages
			.stdin(Stdio::inherit(),)   // Inherit stdin for interactive commands
			.status()?; // Execute and get exit status

		// Check exit status and convert to Result
		out.exit_ok()?; // This will return an error if exit code != 0
		X((),)
	}
}
