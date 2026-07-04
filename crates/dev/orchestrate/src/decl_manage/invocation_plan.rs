use {
	crate::{
		CliCommandDiscriminants, Policy,
		decl_manage::crate_::PoisonGirlCrateChart,
		policy::{
			build_std_features_policy::{
				BuildStdFeaturesPolicies, BuildStdFeaturesPolicy,
			},
			build_std_policy::{BuildStdPolicies, BuildStdPolicy},
			target_policy::TargetPolicy,
		},
	},
	poison_girl_dev_cargo::Runtime,
	poison_girl_dev_error::{PoisonGirlB, X},
	std::path::PathBuf,
};

pub(in crate::decl_manage) struct CargoInvocationPlan
{
	chart:  PoisonGirlCrateChart,
	policy: Policy,
}

impl CargoInvocationPlan
{
	pub(in crate::decl_manage) fn new(
		chart: PoisonGirlCrateChart,
		policy: Policy,
	) -> Self
	{
		Self { chart, policy, }
	}

	pub(super) fn with_supported_features(mut self,) -> Self
	{
		self.policy = self.policy.with_features_supported_by(&self.chart,);
		self
	}

	pub(super) fn chart(&self,) -> &PoisonGirlCrateChart
	{
		&self.chart
	}

	pub(super) fn policy(&self,) -> &Policy
	{
		&self.policy
	}

	pub(super) fn command(&self,) -> CliCommandDiscriminants
	{
		self.policy.command_discriminant()
	}

	fn target_runtime(&self,) -> Runtime
	{
		if self.command() == CliCommandDiscriminants::Test
			|| self.policy.clippy_uses_host_target()
		{
			return Runtime::Host;
		}

		match self.chart {
			PoisonGirlCrateChart::KERNEL => Runtime::PoisonGirl,
			PoisonGirlCrateChart::LOADER => Runtime::Efi,
			_ => Runtime::Host,
		}
	}

	fn build_target_runtime(&self,) -> Runtime
	{
		match self.chart {
			PoisonGirlCrateChart::KERNEL => Runtime::PoisonGirl,
			PoisonGirlCrateChart::LOADER => Runtime::Efi,
			_ => Runtime::Host,
		}
	}

	pub(super) fn target_policy(&self,) -> TargetPolicy
	{
		TargetPolicy::new(self.policy.arch(), self.target_runtime(),)
	}

	pub(super) fn build_target_tuple_representation(&self,) -> PathBuf
	{
		TargetPolicy::new(self.policy.arch(), self.build_target_runtime(),)
			.target_tuple()
			.map(PathBuf::from,)
			.unwrap_or_default()
	}

	fn uses_custom_target(&self,) -> bool
	{
		!matches!(self.target_runtime(), Runtime::Host)
	}

	pub(super) fn build_std_policies(&self,) -> BuildStdPolicies
	{
		let policies = if self.uses_custom_target() {
			match self.chart {
				PoisonGirlCrateChart::KERNEL => vec![BuildStdPolicy::Core],
				PoisonGirlCrateChart::LOADER => vec![
					BuildStdPolicy::Core,
					BuildStdPolicy::Alloc,
					BuildStdPolicy::CompilerBuiltins,
				],
				_ => vec![],
			}
		} else {
			vec![]
		};

		BuildStdPolicies::from(policies,)
	}

	pub(super) fn build_std_features_policies(
		&self,
	) -> BuildStdFeaturesPolicies
	{
		let policies =
			if self.uses_custom_target() && self.chart.uses_custom_runtime() {
				vec![BuildStdFeaturesPolicy::CompilerBuiltinsMem]
			} else {
				vec![]
			};

		BuildStdFeaturesPolicies::from(policies,)
	}

	pub(in crate::decl_manage) fn invocation_policies(
		&self,
	) -> PoisonGirlB<Vec<Policy,>,>
	{
		if self.splits_clippy_targets() {
			return X(vec![
				self.policy.clone().with_clippy_custom_target_lib()?,
				self.policy.clone().with_clippy_host_tests()?,
			],);
		}

		X(vec![self.policy.clone()],)
	}

	fn splits_clippy_targets(&self,) -> bool
	{
		self.command() == CliCommandDiscriminants::Clippy
			&& self.chart.uses_custom_runtime()
			&& self.policy.clippy_lints_all_targets()
	}
}
