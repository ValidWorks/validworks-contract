OWNER_PEM="${USERS}/alice.pem"
SELLER_PEM="${USERS}/bob.pem"
BUYER_PEM="${USERS}/carol.pem"

CONTRACT_ADDRESS=$(erdpy data load --key=address-devnet)
OWNER_ADDRESS_HEX=$(erdpy data load --key=owner-address-hex)
SELLER_ADDRESS_HEX=$(erdpy data load --key=seller-address-hex)

DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)
PROXY="https://devnet-gateway.elrond.com"

deploy() {
  erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${OWNER_PEM} --metadata-not-upgradeable \
        --gas-limit=1400000000 --outfile="deploy-devnet.interaction.json" --send --proxy=${PROXY} --chain=D || return

  TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['hash']")
  CONTRACT_ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['address']")
  OWNER_ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['sender']")

  erdpy data store --key=address-devnet --value=${CONTRACT_ADDRESS}
  erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}
  erdpy data store --key=owner-address --value=${OWNER_ADDRESS}

  echo "Smart contract address: ${CONTRACT_ADDRESS}"
}

GIG_ID="2"
DEADLINE="50"
PRICE="2000000000000000000"
PAYMENT="2400000000000000000"

seller_list() {

    erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${SELLER_PEM} --gas-limit=50000000 --function="list" --arguments ${GIG_ID} ${DEADLINE} ${PRICE} --outfile="list-devnet.interaction.json" --send --proxy=${PROXY} --chain=D

    SELLER_ADDRESS_BECH32=$(erdpy data parse --file="list-devnet.interaction.json" --expression="data['emitted_tx']['tx']['sender']")

    SELLER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode ${SELLER_ADDRESS_BECH32})"

    erdpy data store --key=seller-address-hex --value=${SELLER_ADDRESS_HEX}
    erdpy data store --key=seller-address-bech32 --value=${SELLER_ADDRESS_BECH32}

    echo "Seller address (BECH32): ${SELLER_ADDRESS_BECH32}"
    echo "Seller address (HEX): ${SELLER_ADDRESS_HEX}"

}

buyer_order() {
    erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${BUYER_PEM} --value ${PAYMENT} --gas-limit=50000000 --function="order" --arguments ${GIG_ID} ${SELLER_ADDRESS_HEX} --outfile="order-devnet.interaction.json" --send --proxy=${PROXY} --chain=D

    BUYER_ADDRESS_BECH32=$(erdpy data parse --file="order-devnet.interaction.json" --expression="data['emitted_tx']['tx']['sender']")

    BUYER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode ${BUYER_ADDRESS_BECH32})"

    erdpy data store --key=buyer-address-hex --value=${BUYER_ADDRESS_HEX}
    erdpy data store --key=buyer-address-bech32 --value=${BUYER_ADDRESS_BECH32}

    echo "Buyer address (BECH32): ${BUYER_ADDRESS_BECH32}"
    echo "Buyer address (HEX): ${BUYER_ADDRESS_HEX}"
}

seller_deliver() {
    erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${SELLER_PEM} --gas-limit=50000000 --function="deliver" --arguments ${GIG_ID} --send --proxy=${PROXY} --chain=D
}

buyer_accept() {
    erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${BUYER_PEM} --gas-limit=50000000 --function="accept" --arguments ${GIG_ID} ${SELLER_ADDRESS_HEX} --send --proxy=${PROXY} --chain=D
}

seller_claim() {
    erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${SELLER_PEM} --gas-limit=50000000 --function="claim" --arguments ${GIG_ID} --send --proxy=${PROXY} --chain=D
}

buyer_dispute() {
  erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${BUYER_PEM} --gas-limit=50000000 --function="dispute" --arguments ${GIG_ID} ${SELLER_ADDRESS_HEX} --send --proxy=${PROXY} --chain=D
}

seller_unlist() {
  erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${SELLER_PEM} --gas-limit=50000000 --function="unlist" --arguments ${GIG_ID} --send --proxy=${PROXY} --chain=D
}

buyer_refund() {
  erdpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${BUYER_PEM} --gas-limit=50000000 --function="refund" --arguments ${GIG_ID} ${SELLER_ADDRESS_HEX} --send --proxy=${PROXY} --chain=D
}