use gig::*;
use elrond_wasm_debug::*;

fn main() {
	let contract = GigImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
