pub mod core
{
	pub mod login;
	pub mod messenger;
	pub mod mongo;
	pub mod utils;
}

fn main()
{
	core::utils::clear();
	core::login::login_init();
}
