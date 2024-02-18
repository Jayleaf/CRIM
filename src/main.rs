pub mod core
{
	pub mod login;
	pub mod messenger;
	pub mod utils;
	pub mod mongo;
}

fn main()
{
	core::utils::clear();
	core::login::login_init();
}
