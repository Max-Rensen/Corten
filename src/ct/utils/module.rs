use ct::utils::predefs::Predefs;

pub trait Module {
	fn extend(&self, predefs: &mut Predefs);
}