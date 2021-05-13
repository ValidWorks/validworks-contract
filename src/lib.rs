#![no_std]

imports!();
derive_imports!();

// Order details
#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Order<BigUint: BigUintApi>{
	gig_id: u64,
	delivery_time: u64,
	price: BigUint,
	status: GigStatus
}

// Change these to adjust deposit ratio
const TOTAL_NUMERATOR: u32 = 12; 
const DENOMINATOR: u32 = 10;

// Smart contract to facillitate
#[elrond_wasm_derive::contract(GigImpl)]
pub trait Gig {

	#[init]
    fn init(&self) {
        let my_address: Address = self.blockchain().get_caller();
        self.set_owner(&my_address);
    }

	/* 	
	SELLER List a gig 
	@param gig_id 
	@param deadline
	@param price
	*/
	#[endpoint]
	fn list(&self, gig_id: u64, delivery_time: u64, price: BigUint) -> SCResult<()> {
        // Get address of seller
		let seller_addr = self.blockchain().get_caller();
		// Store price and deadline
		let listing = Order::<BigUint> {gig_id, delivery_time, price, status: GigStatus::Open};
		self.set_listing(&seller_addr, &gig_id, &listing);
		// TODO: Display price and deadline for buyer
        Ok(())
    }

	/* 	
	SELLER Unlist a gig 
	@param gig_id 
	*/
	#[endpoint]
	fn unlist(&self, gig_id: u64) -> SCResult<()> {
        // Get address of seller
		let seller_addr = self.blockchain().get_caller();
		// Store price and deadline
		let mut listing = self.get_listing(&seller_addr, &gig_id);
		listing.status = GigStatus::Close;
		self.set_listing(&seller_addr, &gig_id, &listing);
        Ok(())
    }

	/* 	
	SELLER Deliver order
	@param gig_id
	*/
	#[endpoint]
	fn deliver(&self, gig_id: u64) -> SCResult<()> {
		// Get address of seller
		let seller_addr = self.blockchain().get_caller();
		// Set deadline to accept delivery 3 days from delivery
		let deadline = self.blockchain().get_block_nonce() + 43200; // nonce equiv of 3 days
		// Store deadline
		self.set_deadline_to_accept_delivery(&seller_addr, &gig_id, &deadline);
		Ok(())
	}

	/* 	
	SELLER Claim payment
	@param gig_id
	*/
	#[endpoint]
	fn claim(&self, gig_id: u64) -> SCResult<()>{
		// Get address of seller
		let seller_addr = self.blockchain().get_caller();
		// Get gig listing
		let mut listing = self.get_listing(&seller_addr, &gig_id);
		// Check status
		if let GigStatus::DeliveryAccepted = listing.status{
			// Get payment amount i.e price
			let payment = &listing.price;
			// Send payment to seller
			self.send().direct_egld(&seller_addr, &payment, b"payment sent successfully");
			// Change status
			listing.status = GigStatus::Open;
			self.set_listing(&seller_addr, &gig_id, &listing);
		} else {
			return sc_error!("delivery has not been accepted");
		}
		Ok(())
	}

	/* 	
	BUYER Place an order for a gig
	@param gig_id
	@param seller's address
	@param payment (inclusive of deposit)
	*/
	#[payable("EGLD")]
	#[endpoint]
	fn order(self, #[payment] payment: BigUint, gig_id: u64, seller_addr: Address) -> SCResult<()> {
		// Get address of buyer
		let buyer_addr = self.blockchain().get_caller();
		// Get gig listing
		let listing = self.get_listing(&seller_addr, &gig_id);
		
		// Get price
		let price = listing.price;
		// Check if payment matches price*TOTAL_NUMERATOR
		if payment != price * BigUint::from(TOTAL_NUMERATOR)/BigUint::from(DENOMINATOR) {
            return sc_error!("wrong payment amount");
        }

		// Check if available to order
		if let GigStatus::Open = listing.status{
			// Get deadline
			let delivery_time = listing.delivery_time;
			// Set deadline for delivery
			let deadline_for_delivery = self.blockchain().get_block_nonce() + delivery_time;

			// Store deadline
			self.set_deadline_for_delivery(&buyer_addr, &seller_addr, &gig_id, &deadline_for_delivery);
			// Store payment
			self.set_payment_for_gig(&buyer_addr, &seller_addr, &gig_id, &payment);
		} else {
			return sc_error!("gig not available to order");
		}

		Ok(()) // If @ok means paid
	}

	/* 	
	BUYER Refund if deadline not met
	@param seller_addr
	@param gig_id
	*/
	#[endpoint]
	fn refund(self, gig_id: &u64, seller_addr: &Address) -> SCResult<()> {
		// Get address of buyer
		let buyer_addr = self.blockchain().get_caller();
		// Check deadline for delivery
		let deadline = self.get_deadline_for_delivery(&buyer_addr, &seller_addr, &gig_id);
		// If past deadline send back monies
		if self.blockchain().get_block_nonce() > deadline {
			// Get monies to send back
			let refund = self.get_payment_for_gig(&buyer_addr, &seller_addr, &gig_id);
			// Send back
			self.send().direct_egld(&buyer_addr, &refund, b"payment sent successfully");
		} else{
		// If not send sc_error say cannot
			return sc_error!("seller still has time to delivery");
		}

		Ok(())
	}


	/* 	
	BUYER Dispute delivery
	@param gig_id
	@param seller address
	*/
	#[endpoint]
	fn dispute(&self, gig_id: &u64, seller_addr: &Address) -> SCResult<()> {
		// Get address of buyer
		let buyer_addr = self.blockchain().get_caller();
		// Verify if buyer who paid for this gig
		let mut this_listing = self.get_listing(&seller_addr, &gig_id);
		let payment = self.get_payment_for_gig(&buyer_addr, &seller_addr, &gig_id);
		if payment == &this_listing.price * &BigUint::from(TOTAL_NUMERATOR)/BigUint::from(DENOMINATOR) {
			// Send back payment minus deposit
			self.send().direct_egld(&buyer_addr, &this_listing.price, b"refunded price of gig; deposit withheld");
			// Change status to Open
			this_listing.status = GigStatus::Open;
			self.set_listing(&seller_addr, &gig_id, &this_listing);
		} else {
			return sc_error!("you are not allowed to dispute this order");
		}
		Ok(())
	}

	/* 	
	BUYER Accept and release payment
	@param gig_id
	*/
	#[endpoint]
	fn accept(self, gig_id: &u64, seller_addr: &Address) -> SCResult<()> {
		// Get address of buyer
		let buyer_addr = self.blockchain().get_caller();
		// Verify if buyer who paid for this gig
		let mut this_listing = self.get_listing(&seller_addr, &gig_id);
		let payment = self.get_payment_for_gig(&buyer_addr, &seller_addr, &gig_id);
		if payment == &this_listing.price * &BigUint::from(TOTAL_NUMERATOR)/BigUint::from(DENOMINATOR) {
			// Send back deposit to buyer
			let deposit = &payment - &this_listing.price;
			self.send().direct_egld(&buyer_addr, &deposit, b"delivery accepted; deposit refunded");
			// Change status to DeliveryAccepted
			this_listing.status = GigStatus::DeliveryAccepted;
			self.set_listing(&seller_addr, &gig_id, &this_listing);
		} else {
			return sc_error!("you are not allowed to accept this order");
		}
		Ok(())
	}

	// GETTERS & SETTERS

	#[storage_set("payment_for_gig")]
    fn set_payment_for_gig(&self, buyer_addr: &Address, seller_addr: &Address, gig_id: &u64, payment: &BigUint);

    #[view]
    #[storage_get("payment_for_gig")]
    fn get_payment_for_gig(&self, buyer_addr:&Address, seller_addr: &Address, gig_id: &u64) -> BigUint;

	#[storage_set("deadline_for_delivery")]
    fn set_deadline_for_delivery(&self, buyer_addr: &Address, seller_addr: &Address, gig_id: &u64, deadline: &u64);

    #[view]
    #[storage_get("deadline_for_delivery")]
    fn get_deadline_for_delivery(&self, buyer_addr:&Address, seller_addr: &Address, gig_id: &u64) -> u64;

	#[storage_set("deadline_to_accept_delivery")]
    fn set_deadline_to_accept_delivery(&self, seller_addr: &Address, gig_id: &u64, deadline: &u64);

    #[view]
    #[storage_get("deadline_to_accept_delivery")]
    fn get_deadline_to_accept_delivery(&self, seller_addr: &Address, gig_id: &u64) -> u64;

	#[storage_set("listing")]
    fn set_listing(&self, seller_addr: &Address, gig_id: &u64, listing: &Order<BigUint>);

    #[view]
    #[storage_get("listing")]
    fn get_listing(&self, seller_addr: &Address, gig_id: &u64) -> Order<BigUint>;

	#[storage_set("owner")]
    fn set_owner(&self, address: &Address);

    #[view]
    #[storage_get("owner")]
    fn get_owner(&self) -> Address;
}



#[derive(TopEncode, TopDecode, TypeAbi, NestedEncode, NestedDecode)]
enum GigStatus {
	Close,
	Open,
	InOrder,
	DeliveryAccepted
}
