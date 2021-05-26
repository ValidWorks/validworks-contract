<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->



<!-- PROJECT LOGO -->
<br />
<p align="center">
  <a href="https://cryptologos.cc/logos/elrond-egld-egld-logo.png">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">ValidWorks Smart Contracte</h3>

  <p align="center">
    Smart contract for the escrow services built on Elrond
  </p>
</p>



<!-- TABLE OF CONTENTS -->
<details open="open">
  <summary>Table of Contents</summary>
  <ol>
        <li><a href="#Policy">Policy</a></li>
    <li>
      <a href="#Walkthrough">Walkthrough</a>
      <ul>
        <li><a href="#Open">Open</a></li>
      </ul>
            <ul>
        <li><a href="#InOrder">InOrder</a></li>
      </ul>
                  <ul>
        <li><a href="#Delivered">Delivered</a></li>
      </ul>
                  <ul>
        <li><a href="#DeliveryAccepted">DeliveryAccepted</a></li>
      </ul>
    </li>
    <li><a href="#demo">Demo</a></li>
    <li><a href="#usage">Usage</a>
      <ul>
        <li><a href="#ExampleUsage">Example Usage</a></li>
      </ul></li>
  </ol>
</details>



<!-- POLICY -->
## Policy

By enforcing a 20% deposit on top of the listing price, buyers are disincentivised from making false disputes despite satisfactory deliveries from the seller. On making a dispute, buyers will be lose the deposit and only get refunded in the price of the gig. 

<!-- WALKTRHOUGH -->
## Walkthrough

There are 4 stages to a Gig, namely Open, InOrder, Delivered and DeliveryAccepted. At every stage, both sellers and buyers will only certain number of actions available to them. Calling functions not available at a particular stage will result in failed transactions.

### Open

A seller will LIST the Gig by providing an ID (for tracking orders), price and delivery time. After listing, the Gig will be in the Open stage. At this stage, the seller will only be able to UNLIST the Gig and potential buyers will be able to ORDER.

The smart contract will check if the buyer has made payment in the amount of the price of the Gig + deposit (1.2 * price) else it would fail the transaction.

### InOrder

When a buyer has placed an order, the Gig was go into the InOrder stage. The deadline for delivery will be set based on the delivery time promised by the seller in the listing. 

At this stage, the seller may DELIVER. After the deadline has been past and the seller has not delivered, the buyer may REFUND and get a full refund inclusive of deposit.

Also, at any point before the seller may also DISPUTE to cancel the Gig but get refunded only in the price of the Gig and lose deposit.

### Delivered

After the seller has delivered, the Gig is now in the Delivered stage. The seller will have 3 days to accept/dispute the delivery. If the buyer disputes the delivery, the buyer gets refunded only in the price of the gig and lose deposit. The Gig will return to the Open stage. If the buyer accepts the delivery, the Gig goes into the DeliveryAccepted stage. The deposit will be refunded to the buyer and the amount in the price of the Gig will be made available to CLAIM by the seller. 

If the buyer does neither in the 3 day window, the amount in the price of the gig will be automatically made available to CLAIM by the seller. The seller can do so and the Gig will return to the Open stage.

### DeliveryAccepted

At this stage, the seller can CLAIM payment in the price of the Gig and the Gig will be returned to the Open stage.

<!-- DEMO -->
## Demo

Please refer to interaction/testnet.snippets.sh for demo using erdpy. The snippets have been set up to use 2 different addresses for seller and buyer respectively. 


<!-- USAGE -->
## Usage

To use our smart contract in your frontend, you will need erdjs. Import the functions you'll need from the ErdJsUtils file.

Note the current implementation requires Ledger hardware wallet although repurposing the same code base for keystore login should only be a matter of changing a provider.

### Example Usage

```
    buyerDispute(
      user.get("erdAddress"),
      gig.getOnChainId(),
      gig.getSellerAddr()
    )
      .then((reply) => {
        console.log(reply.getHash().toString());
        if (reply.getStatus().isSuccessful()) {
          gig.setStatus("Open");
          gig.removeBuyerId();
        }
      })
      .catch((err) => {
        console.log(err);
      });
```

Here buyerDispute is called from ErdJsUtils which returns a Transaction object (refer to erdjs docs if needed) after the the contract call has been executed on the blockchain. All the helper functions are asynchronous so you can define what happens immediately after the execution.


