fn main() {

}

use server_client::server_client;

server_client!(
	pub Example {
		let a: usize

		fn increment(b: usize) -> usize {
			self.a += b;
			self.a
		}
	}
);