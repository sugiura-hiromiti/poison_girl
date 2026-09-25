use {
	crate::{
		CliCommandDiscriminants, Policy,
		cli_interface::{CargoInvocation, RenderCargoInvocation, TargetKind},
		decl_manage::crate_::PoisonGirlCrateChart,
		policy::{
			build_std_features_policy::{
				BuildStdFeaturesPolicies, BuildStdFeaturesPolicy,
			},
			build_std_policy::{BuildStdPolicies, BuildStdPolicy},
			execution_policy::ExecutionPolicy,
			package_policy::PackagePolicy,
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

	fn resolve(self,) -> PoisonGirlB<Vec<CargoInvocation,>,>
	{
		self.execution_policies()?
			.into_iter()
			.map(|execution| self.resolve_one(execution,),)
			.try_collect()
	}

	fn resolve_one(
		&self,
		execution: ExecutionPolicy,
	) -> PoisonGirlB<CargoInvocation,>
	{
		let target = self.target_policy(&execution,);
		let build_std = self.build_std_policies(&execution,);
		let build_std_features = self.build_std_features_policies(&execution,);
		let package = self.package_policy();

		let mut invocation = CargoInvocation::default();

		invocation.extend(execution.render(),);
		invocation.extend(package.render(),);
		invocation.extend(target.render(),);
		invocation.extend(build_std.render(),);
		invocation.extend(build_std_features.render(),);

		X(invocation,)
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

	fn build_target_runtime(&self,) -> Runtime
	{
		match self.chart {
			PoisonGirlCrateChart::KERNEL => Runtime::PoisonGirl,
			PoisonGirlCrateChart::LOADER => Runtime::Efi,
			_ => Runtime::Host,
		}
	}

	pub(super) fn build_target_tuple_representation(&self,) -> PathBuf
	{
		TargetPolicy::new(self.policy.arch(), self.build_target_runtime(),)
			.target_tuple()
			.map(PathBuf::from,)
			.unwrap_or_default()
	}

	fn uses_custom_target(&self, execution: &ExecutionPolicy,) -> bool
	{
		!matches!(execution.runtime(), Runtime::Host)
	}

	pub(super) fn target_policy(
		&self,
		execution: &ExecutionPolicy,
	) -> TargetPolicy
	{
		TargetPolicy::new(self.policy.arch(), execution.runtime(),)
	}

	pub(super) fn build_std_policies(
		&self,
		execution: &ExecutionPolicy,
	) -> BuildStdPolicies
	{
		let policies = if self.uses_custom_target(execution,) {
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
		execution: &ExecutionPolicy,
	) -> BuildStdFeaturesPolicies
	{
		let policies = if self.uses_custom_target(execution,)
			&& self.chart.uses_custom_runtime()
		{
			vec![BuildStdFeaturesPolicy::CompilerBuiltinsMem]
		} else {
			vec![]
		};

		BuildStdFeaturesPolicies::from(policies,)
	}

	pub(in crate::decl_manage) fn execution_policies(
		&self,
	) -> PoisonGirlB<Vec<ExecutionPolicy,>,>
	{
		if self.splits_clippy_targets() {
			return X(vec![
				ExecutionPolicy::new(
					self.build_target_runtime(),
					TargetKind::Lib,
				),
				ExecutionPolicy::new(Runtime::Host, TargetKind::Test,),
			],);
		}

		X(vec![ExecutionPolicy::new(
			if self.command() == CliCommandDiscriminants::Test {
				Runtime::Host
			} else {
				self.build_target_runtime()
			},
			TargetKind::Auto,
		)],)
	}

	fn package_policy(&self,) -> PackagePolicy
	{
		PackagePolicy::new(self.chart,)
	}

	fn splits_clippy_targets(&self,) -> bool
	{
		self.command() == CliCommandDiscriminants::Clippy
			&& self.chart.uses_custom_runtime()
			&& self.policy.clippy_lints_all_targets()
	}
}
