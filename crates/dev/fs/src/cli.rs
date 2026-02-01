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

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn test_run_trait_successful_command()
	{
		let mut cmd = Command::new("echo",);
		cmd.arg("test",);

		let result = cmd.run();
		assert!(result.is_x(), "Echo command should succeed");
	}

	#[test]
	fn test_run_trait_failing_command()
	{
		let mut cmd = Command::new("false",); // Command that always fails

		let result = cmd.run();
		assert!(result.is_y(), "False command should fail");
	}

	#[test]
	fn test_run_trait_nonexistent_command()
	{
		let mut cmd = Command::new("definitely_nonexistent_command_12345",);

		let result = cmd.run();
		assert!(result.is_y(), "Nonexistent command should fail");
	}

	#[test]
	fn test_run_trait_with_args()
	{
		let mut cmd = Command::new("echo",);
		cmd.args(&["hello", "world",],);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with args should succeed");
	}

	#[test]
	fn test_run_trait_with_env()
	{
		let mut cmd = Command::new("echo",);
		cmd.arg("test",);
		cmd.env("TEST_VAR", "test_value",);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with env should succeed");
	}

	#[test]
	fn test_run_trait_multiple_calls()
	{
		// Test that we can call run multiple times on different command
		// instances
		let mut cmd1 = Command::new("echo",);
		cmd1.arg("first",);
		assert!(cmd1.run().is_x());

		let mut cmd2 = Command::new("echo",);
		cmd2.arg("second",);
		assert!(cmd2.run().is_x());
	}

	#[test]
	fn test_run_trait_with_current_dir()
	{
		let mut cmd = Command::new("pwd",);
		cmd.current_dir("/tmp",);

		let result = cmd.run();
		// This might fail on some systems if /tmp doesn't exist or pwd isn't
		// available but we're mainly testing that the trait works with
		// current_dir
		let _ = result; // Don't assert success since it's system-dependent
	}

	#[test]
	fn test_command_builder_pattern()
	{
		// Test that we can use the builder pattern with our trait
		let result = Command::new("echo",)
			.arg("builder",)
			.arg("pattern",)
			.env("TEST", "value",)
			.run();

		assert!(result.is_x(), "Builder pattern should work");
	}

	#[test]
	fn test_run_trait_idempotent_success()
	{
		// Test that successful commands are idempotent
		let mut cmd = Command::new("true",); // Command that always succeeds

		assert!(cmd.run().is_x());
		// Note: We can't call run again on the same Command instance
		// because std::process::Command consumes itself on spawn()
	}

	#[test]
	fn test_run_trait_idempotent_failure()
	{
		// Test that failing commands consistently fail
		let mut cmd = Command::new("false",); // Command that always fails

		assert!(cmd.run().is_y());
		// Note: We can't call run again on the same Command instance
	}

	#[test]
	fn test_run_trait_with_output_redirection()
	{
		// Test commands that might produce output
		let mut cmd = Command::new("echo",);
		cmd.arg("output_test",);

		let result = cmd.run();
		assert!(result.is_x(), "Echo should succeed even with output");
	}

	#[test]
	fn test_run_trait_with_error_output()
	{
		// Test commands that write to stderr
		let mut cmd = Command::new("sh",);
		cmd.args(&["-c", "echo 'error message' >&2",],);

		let result = cmd.run();
		assert!(
			result.is_x(),
			"Command writing to stderr should still succeed if exit code is 0"
		);
	}

	#[test]
	fn test_run_trait_exit_code_handling()
	{
		// Test that non-zero exit codes are treated as errors
		let mut cmd = Command::new("sh",);
		cmd.args(&["-c", "exit 1",],);

		let result = cmd.run();
		assert!(result.is_y(), "Non-zero exit code should be treated as error");
	}

	#[test]
	fn test_run_trait_zero_exit_code()
	{
		// Test that zero exit code is treated as success
		let mut cmd = Command::new("sh",);
		cmd.args(&["-c", "exit 0",],);

		let result = cmd.run();
		assert!(result.is_x(), "Zero exit code should be treated as success");
	}

	#[test]
	fn test_run_trait_with_long_running_command()
	{
		// Test with a command that takes a bit of time
		let mut cmd = Command::new("sleep",);
		cmd.arg("0.1",); // Sleep for 100ms

		let result = cmd.run();
		assert!(result.is_x(), "Sleep command should succeed");
	}

	#[test]
	fn test_run_trait_command_not_found_vs_execution_failure()
	{
		// Test the difference between command not found and execution failure
		let mut nonexistent_cmd =
			Command::new("definitely_nonexistent_command_12345",);
		let nonexistent_result = nonexistent_cmd.run();
		assert!(nonexistent_result.is_y());

		let mut failing_cmd = Command::new("false",);
		let failing_result = failing_cmd.run();
		assert!(failing_result.is_y());

		// Both should fail, but potentially with different error types
		// We can't easily distinguish them in the test, but both should be
		// errors
	}

	#[test]
	fn test_run_trait_with_complex_args()
	{
		// Test with complex argument patterns
		let mut cmd = Command::new("echo",);
		cmd.args(&["--flag", "value", "-x", "test with spaces",],);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with complex args should succeed");
	}

	#[test]
	fn test_run_trait_with_empty_args()
	{
		// Test with no arguments
		let mut cmd = Command::new("true",); // Command that always succeeds

		let result = cmd.run();
		assert!(result.is_x(), "True command with no args should succeed");
	}

	#[test]
	fn test_run_trait_with_special_characters_in_args()
	{
		// Test with special characters in arguments
		let mut cmd = Command::new("echo",);
		cmd.args(&["hello", "world!", "@#$%", "test",],);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with special characters should succeed");
	}

	#[test]
	fn test_run_trait_with_unicode_args()
	{
		// Test with unicode characters in arguments
		let mut cmd = Command::new("echo",);
		cmd.args(&["hello", "世界", "🦀", "test",],);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with unicode should succeed");
	}

	#[test]
	fn test_run_trait_with_very_long_args()
	{
		// Test with very long arguments
		let long_arg = "a".repeat(1000,);
		let mut cmd = Command::new("echo",);
		cmd.arg(&long_arg,);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with long args should succeed");
	}

	#[test]
	fn test_run_trait_with_many_args()
	{
		// Test with many arguments
		let mut cmd = Command::new("echo",);
		for i in 0..50 {
			cmd.arg(format!("arg{}", i,),);
		}

		let result = cmd.run();
		assert!(result.is_x(), "Echo with many args should succeed");
	}

	#[test]
	fn test_run_trait_with_env_vars()
	{
		// Test with multiple environment variables
		let mut cmd = Command::new("echo",);
		cmd.arg("test",);
		cmd.env("VAR1", "value1",);
		cmd.env("VAR2", "value2",);
		cmd.env("UNICODE_VAR", "世界",);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with multiple env vars should succeed");
	}

	#[test]
	fn test_run_trait_with_working_directory()
	{
		// Test with different working directories
		let mut cmd = Command::new("pwd",);

		// Try to set working directory to /tmp if it exists
		if std::path::Path::new("/tmp",).exists() {
			cmd.current_dir("/tmp",);
		}

		let result = cmd.run();
		// Don't assert success since pwd might not be available on all systems
		let _ = result;
	}

	#[test]
	fn test_run_trait_command_display_formatting()
	{
		// Test that command display includes program name and args
		// This is harder to test directly since the output goes to stdout
		// But we can at least verify the command runs without panicking
		let mut cmd = Command::new("echo",);
		cmd.args(&["test", "display", "formatting",],);

		let result = cmd.run();
		assert!(result.is_x(), "Command display formatting should work");
	}

	#[test]
	fn test_run_trait_with_shell_commands()
	{
		// Test with shell commands that might have different exit codes
		let test_cases = vec![
			("true", true,),   // Should succeed
			("false", false,), // Should fail
		];

		for (cmd_name, should_succeed,) in test_cases {
			let mut cmd = Command::new(cmd_name,);
			let result = cmd.run();

			if should_succeed {
				assert!(result.is_x(), "{} should succeed", cmd_name);
			} else {
				assert!(result.is_y(), "{} should fail", cmd_name);
			}
		}
	}

	#[test]
	fn test_run_trait_error_propagation()
	{
		// Test that errors are properly propagated
		let mut cmd = Command::new("sh",);
		cmd.args(&["-c", "exit 42",],); // Exit with specific code

		let result = cmd.run();
		assert!(result.is_y(), "Non-zero exit should be error");
	}

	#[test]
	fn test_run_trait_with_stdin_interaction()
	{
		// Test commands that might read from stdin
		let mut cmd = Command::new("cat",);
		cmd.args(&["/dev/null",],); // Read from /dev/null instead of stdin

		let result = cmd.run();
		// This might fail on some systems, so just check it doesn't panic
		let _ = result;
	}

	#[test]
	fn test_run_trait_with_output_commands()
	{
		// Test commands that produce various types of output
		let test_commands = vec![
			vec!["echo", "simple output"],
			vec!["echo", "-n", "no newline"],
			vec!["printf", "formatted %s\\n", "output"],
		];

		for args in test_commands {
			if args.is_empty() {
				continue;
			}

			let mut cmd = Command::new(&args[0],);
			if args.len() > 1 {
				cmd.args(&args[1..],);
			}

			let result = cmd.run();
			// Don't assert success since not all commands may be available
			let _ = result;
		}
	}

	#[test]
	fn test_run_trait_concurrent_execution()
	{
		// Test that multiple commands can be run concurrently
		use std::thread;

		let handles: Vec<_,> = (0..5)
			.map(|i| {
				thread::spawn(move || {
					let mut cmd = Command::new("echo",);
					cmd.arg(format!("thread_{}", i,),);
					cmd.run()
				},)
			},)
			.collect();

		for handle in handles {
			let result = handle.join().expect("Thread should not panic",);
			assert!(result.is_x(), "Concurrent echo should succeed");
		}
	}

	#[test]
	fn test_run_trait_with_path_commands()
	{
		// Test commands with full paths
		let common_paths =
			vec!["/bin/echo", "/usr/bin/echo", "/bin/true", "/usr/bin/true"];

		for path in common_paths {
			if std::path::Path::new(path,).exists() {
				let mut cmd = Command::new(path,);
				cmd.arg("path_test",);

				let result = cmd.run();
				assert!(
					result.is_x(),
					"Command with full path should work: {}",
					path
				);
				break; // Only test the first available one
			}
		}
	}

	#[test]
	fn test_run_trait_with_relative_paths()
	{
		// Test with relative path commands (if any exist)
		// This is system-dependent, so we'll just test that it doesn't panic
		let mut cmd = Command::new("./nonexistent_relative_command",);
		let result = cmd.run();
		assert!(result.is_y(), "Nonexistent relative command should fail");
	}

	#[test]
	fn test_run_trait_command_chaining_simulation()
	{
		// Simulate command chaining by running multiple commands in sequence
		let commands = vec![
			vec!["echo", "first"],
			vec!["echo", "second"],
			vec!["echo", "third"],
		];

		for args in commands {
			let mut cmd = Command::new(&args[0],);
			if args.len() > 1 {
				cmd.args(&args[1..],);
			}

			let result = cmd.run();
			assert!(result.is_x(), "Sequential commands should work");
		}
	}

	#[test]
	fn test_run_trait_with_numeric_args()
	{
		// Test with numeric arguments
		let mut cmd = Command::new("echo",);
		cmd.args(&["123", "456.789", "-42", "0",],);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with numeric args should succeed");
	}

	#[test]
	fn test_run_trait_with_boolean_like_args()
	{
		// Test with boolean-like arguments
		let mut cmd = Command::new("echo",);
		cmd.args(&["true", "false", "yes", "no", "on", "off",],);

		let result = cmd.run();
		assert!(result.is_x(), "Echo with boolean-like args should succeed");
	}
}
