use {
	crate::{cli_interface::CompileOpt, decl_manage::crate_::Crate},
	poison_girl_dev_error::PoisonGirlB,
	std::path::PathBuf,
};

pub struct BuildArtifact
{
	path: PathBuf,
}

impl BuildArtifact
{
	pub fn new(path: PathBuf,) -> Self
	{
		Self { path, }
	}

	pub fn path(&self,) -> PathBuf
	{
		self.path.clone()
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

#[cfg(test)]
mod tests
{
	use {super::*, std::path::PathBuf};

	#[test]
	fn build_artifact_path_is_resolved_value()
	{
		let path = PathBuf::from("resolved-target",)
			.join("aarch64-unknown-uefi",)
			.join("debug",)
			.join("loader.efi",);

		let artifact = BuildArtifact::new(path.clone(),);

		assert_eq!(artifact.path(), path);
	}
}
