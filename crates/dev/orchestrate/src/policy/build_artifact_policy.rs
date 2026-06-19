// TODO: refactor `BuildArtifact` datatype more decralative and separate
// policies

use {
	crate::{
		cli_interface::CompileOpt,
		decl_manage::crate_::{Crate, PoisonGirlCrateChart},
	},
	poison_girl_dev_error::PoisonGirlB,
	std::path::PathBuf,
};

pub struct BuildArtifact
{
	target_dir:                  PathBuf,
	/// maybe should be Option
	target_tuple_representation: PathBuf,
	profile:                     PathBuf,
	artifact_name:               PathBuf,
}

impl BuildArtifact
{
	pub fn new(
		target_dir: PathBuf,
		target_tuple_representation: PathBuf,
		profile: PathBuf,
		artifact_name: PathBuf,
	) -> Self
	{
		Self {
			target_dir, target_tuple_representation, profile, artifact_name,
		}
	}

	pub fn path(&self,) -> PathBuf
	{
		let Self {
			target_dir,
			target_tuple_representation,
			profile,
			artifact_name,
		} = self;

		let workspace_root = PoisonGirlCrateChart::XTASK.to_path_buf();

		workspace_root
			.join(target_dir,)
			.join(target_tuple_representation,)
			.join(profile,)
			.join(artifact_name,)
	}
}

/// orchestration functionality which is finally resolved.
/// this is required over any traits in this crate because finally resolved
/// orchestration graph is not statically determined e.g. by env var, and this
/// trait also connect to existing cargo orchestration as needed
pub trait BuildArtifactPolicyResolver
{
	fn build_artifact_policy(&self,) -> PoisonGirlB<BuildArtifact,>;
	fn resolve_target_dir(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_target_triple_representation(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_profile(&self,) -> PathBuf;
	fn resolve_artifact_name(&self,) -> PoisonGirlB<PathBuf,>;

	fn as_crate(&self,) -> &impl Crate;
	fn as_opts(&self,) -> &impl CompileOpt;
}
