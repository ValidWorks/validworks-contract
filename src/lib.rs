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

// Change this to adjust time to accept delivery
const TIME_TO_ACCEPT_DELIVERY: u64 = 50; // nonce equiv of 5 minutes

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
		// Check if gig_id has been used
		if !!!self.is_empty_listing(&seller_addr, &gig_id){
			return sc_error!("Gig_id has been used by this address");
		}
		// Store price and deadline
		let listing = Order::<BigUint> {gig_id, delivery_time, price, status: GigStatus::Open};
		self.set_listing(&seller_addr, &gig_id, &listing);
		// TODO: Display price and deadline for buyer
        Ok(())
    }

	/* 	
	SELLER Unlist a gig 
	@param gig_id 
	@condition listing must be currently open
	*/
	#[endpoint]
	fn unlist(&self, gig_id: u64) -> SCResult<()> {
        // Get address of seller
		let seller_addr = self.blockchain().get_caller();
		// Check if there's a listing to unlist
		if self.is_empty_listing(&seller_addr, &gig_id){
			return sc_error!("No listing with this gig_id");
		}
		// Check if the listing is in Open status
		let listing = self.get_listing(&seller_addr, &gig_id);
		if let GigStatus::Open = listing.status {
			// Delete listing
			self.delete_listing(&seller_addr, &gig_id);
		} else {
			return sc_error!("Listing is currently in order; you may not unlist now");
		}	
        Ok(())
    }

	/* 	
	SELLER Deliver order
	@param gig_id
	@condition called after buyer order
	@condition cannot be called if past deadline to deliver
	*/
	#[endpoint]
	fn deliver(&self, gig_id: u64) -> SCResult<()> {
		// Get address of seller
		let seller_addr = self.blockchain().get_caller();
		// Get gig listing
		let mut listing = self.get_listing(&seller_addr, &gig_id);

		// TODO: If want to disallow delivery past deadline to delivery would require 
		// seller to have buyer's address

		match listing.status {
			// Can deliver
			GigStatus::InOrder => {
				// Set deadline to accept delivery 3 days from delivery
				let deadline = self.blockchain().get_block_nonce() + TIME_TO_ACCEPT_DELIVERY; 
				// Store deadline
				self.set_deadline_to_accept_delivery(&seller_addr, &gig_id, &deadline);
				// Change status to Delivered
				listing.status = GigStatus::Delivered;
				self.set_listing(&seller_addr, &gig_id, &listing);
			}
			GigStatus::Delivered => {
				return sc_error!("Already delivered for this order");
			}
			_ => return sc_error!("Cannot deliver now"),
		}
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
		match listing.status {
			// If delivery accepted then accept payment
			GigStatus::DeliveryAccepted => {
				// Get payment amount i.e price
				let payment = &listing.price;
				// Send payment to seller
				self.send().direct_egld(&seller_addr, &payment, b"payment sent successfully");
				// Change status
				listing.status = GigStatus::Open;
				self.set_listing(&seller_addr, &gig_id, &listing);
			}
			// If delivery still in Delivered state, check if deadline to accept has been past
			GigStatus::Delivered => {
				let deadline_to_accept_delivery = self.get_deadline_to_accept_delivery(&seller_addr, &gig_id);
				if self.blockchain().get_block_nonce() > deadline_to_accept_delivery {
					// Get payment amount i.e price
					let payment = &listing.price;
					// Send payment to seller
					self.send().direct_egld(&seller_addr, &payment, b"payment sent successfully");
					// Change status
					listing.status = GigStatus::Open;
					self.set_listing(&seller_addr, &gig_id, &listing);
				} else {
					return sc_error!("buyer still has time to accept delivery");
				}
			}
			// If no order has been placed
			_ => return sc_error!("you can not claim now"),
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
		let mut listing = self.get_listing(&seller_addr, &gig_id);
		
		// Get price
		let price = &listing.price;
		// Check if payment matches price*TOTAL_NUMERATOR
		if payment != price * &BigUint::from(TOTAL_NUMERATOR)/BigUint::from(DENOMINATOR) {
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

			// Change status to InOrder
			listing.status = GigStatus::InOrder;
			self.set_listing(&seller_addr, &gig_id, &listing);
		} else {
			return sc_error!("gig not available to order");
		}

		Ok(()) // If @ok means paid
	}

	/* 	
	BUYER Refund if deadline not met
	@param seller_addr
	@param gig_id
	@condition can only be called when InOrder
	*/
	#[endpoint]
	fn refund(self, gig_id: &u64, seller_addr: &Address) -> SCResult<()> {
		// Get address of buyer
		let buyer_addr = self.blockchain().get_caller();

		let mut this_listing = self.get_listing(&seller_addr, &gig_id);

		// Verify if listing is InOrder
		match this_listing.status {
			GigStatus::InOrder => {}
			_ => return sc_error!("Not allowed to refund now")
		}

		// Check deadline for delivery
		let deadline = self.get_deadline_for_delivery(&buyer_addr, &seller_addr, &gig_id);
		// If past deadline send back monies
		if self.blockchain().get_block_nonce() > deadline {
			// Get monies to send back
			let refund = self.get_payment_for_gig(&buyer_addr, &seller_addr, &gig_id);
			// Send back
			self.send().direct_egld(&buyer_addr, &refund, b"payment sent successfully");

			// Change status to Open
			this_listing.status = GigStatus::Open;
			self.set_listing(&seller_addr, &gig_id, &this_listing);
			// Remove payment from storage
			self.payment_ok(&buyer_addr, &seller_addr, &gig_id);
		} else{
		// If not send sc_error say cannot
			return sc_error!("seller still has time to deliver");
		}

		Ok(())
	}


	/* 	
	BUYER Dispute/cancel delivery
	@param gig_id
	@param seller address
	@condition gig must be InOrder/Delivered
	*/
	#[endpoint]
	fn dispute(&self, gig_id: &u64, seller_addr: &Address) -> SCResult<()> {
		// Get address of buyer
		let buyer_addr = self.blockchain().get_caller();
		
		let mut this_listing = self.get_listing(&seller_addr, &gig_id);
		
		// Verify if listing is InOrder
		match this_listing.status {
			GigStatus::InOrder | GigStatus::Delivered => {}
			_ => return sc_error!("Not allowed to dispute/cancel now")
		}

		// Verify if buyer who paid for this gig
		if self.has_paid_for_gig(&buyer_addr, &seller_addr, &gig_id) {
			// Send back payment minus deposit
			self.send().direct_egld(&buyer_addr, &this_listing.price, b"refunded price of gig; deposit withheld");
			// Change status to Open
			this_listing.status = GigStatus::Open;
			self.set_listing(&seller_addr, &gig_id, &this_listing);
			// Remove payment from storage
			self.payment_ok(&buyer_addr, &seller_addr, &gig_id);
		} else {
			return sc_error!("You are not allowed to dispute/cancel this order");
		}
		Ok(())
	}

	/* 	
	BUYER Accept and release payment
	@param gig_id
	@condition must be before deadline_to_accept
	*/
	#[endpoint]
	fn accept(self, gig_id: &u64, seller_addr: &Address) -> SCResult<()> {
		// Get address of buyer
		let buyer_addr = self.blockchain().get_caller();
		let mut this_listing = self.get_listing(&seller_addr, &gig_id);

		// Verify if listing is Delivered
		match this_listing.status {
			GigStatus::Delivered => {}
			_ => return sc_error!("no delivery to accept"),
		}
		
		// Verify if past deadline_to_accept
		let deadline = self.get_deadline_to_accept_delivery(&seller_addr, &gig_id);
		// If past deadline say too late
		if self.blockchain().get_block_nonce() > deadline {
			return sc_error!("deadline past to accept delivery");
		}

		// Verify if buyer who paid for this gig
		let payment = self.get_payment_for_gig(&buyer_addr, &seller_addr, &gig_id);
		if payment == &this_listing.price * &BigUint::from(TOTAL_NUMERATOR)/BigUint::from(DENOMINATOR) {
			// Send back deposit to buyer
			let deposit = &payment - &this_listing.price;
			self.send().direct_egld(&buyer_addr, &deposit, b"delivery accepted; deposit refunded");
			// Change status to DeliveryAccepted
			this_listing.status = GigStatus::DeliveryAccepted;
			self.set_listing(&seller_addr, &gig_id, &this_listing);
			// Remove payment from storage
			self.payment_ok(&buyer_addr, &seller_addr, &gig_id);
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

	#[view]
    #[storage_is_empty("payment_for_gig")]
    fn has_paid_for_gig(&self, buyer_address: &Address, seller_addr: &Address, gig_id: &u64) -> bool;

    #[storage_clear("payment_for_gig")]
    fn payment_ok(&self, buyer_addr: &Address, seller_addr: &Address, gig_id: &u64);

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

	#[view]
    #[storage_is_empty("listing")]
    fn is_empty_listing(&self, seller_addr: &Address, gig_id: &u64) -> bool;

    #[storage_clear("listing")]
    fn delete_listing(&self, seller_addr: &Address, gig_id: &u64);

	#[storage_set("owner")]
    fn set_owner(&self, address: &Address);

    #[view]
    #[storage_get("owner")]
    fn get_owner(&self) -> Address;
}



#[derive(TopEncode, TopDecode, TypeAbi, NestedEncode, NestedDecode)]
enum GigStatus {
	Open,
	InOrder,
	Delivered,
	DeliveryAccepted
}
