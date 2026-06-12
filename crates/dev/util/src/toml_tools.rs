pub trait TomlMerge: Sized
{
	fn update_by(&mut self, by: Self,);
	fn into_updated_by(mut self, by: Self,) -> Self
	{
		self.update_by(by,);
		self
	}
}

impl TomlMerge for toml::value::Table
{
	fn update_by(&mut self, by: Self,)
	{
		for (key, val,) in by {
			match val {
				toml::Value::Table(ref object_table,) => {
					match self.get_mut(&key,) {
						Some(toml::Value::Table(subject_table,),) => {
							subject_table.update_by(object_table.clone(),)
						},
						_ => {
							self.insert(key, val,);
						},
					}
				},
				_ => {
					self.insert(key, val,);
				},
			}
		}
	}
}
