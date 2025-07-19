// A representation of a block in our blockchain
// Generic over Header and Extrinsic types - this makes it flexible
pub struct Block<Header, Extrinsic> {
    // The block header contains metadata about the block
    pub header: Header,
    // The extrinsics are a collection of transactions to execute
    pub extrinsics: Vec<Extrinsic>,
}

// Header struct that contains metadata of the block
// Generic over BlockNumber type - can be u32, u64, etc.
pub struct Header<BlockNumber> {
    pub block_number: BlockNumber,
    // Future additions could include:
    // pub parent_hash: [u8; 32],
    // pub state_root: [u8; 32],
    // pub timestamp: u64,
}

// Extrinsic struct that contains information about the transaction to execute
// Generic over Caller and Call types - flexible for different account and call types
pub struct Extrinsic<Caller, Call> {
    pub caller: Caller, // Who is making the transaction
    pub call: Call,     // What action they want to perform
}

// Result type for runtime operations
pub type DispatchResult = Result<(), &'static str>;

// A trait for handling incoming extrinsics
// This is the core dispatch mechanism - any pallet that can handle transactions implements this
pub trait Dispatch {
    // Who is calling the function - this could be String, u32, or any account type
    type Caller;
    // What function or transaction is being called - this is pallet-specific
    type Call;

    // The main dispatch function that executes a call on behalf of a caller
    fn dispatch(&mut self, caller: Self::Caller, call: Self::Call) -> DispatchResult;
}

/*
EXPLANATION OF GENERICS IN THIS FILE:

1. Block<Header, Extrinsic>:
   - This is like a template that can work with any Header and Extrinsic type
   - Instead of hardcoding specific types, we use placeholders
   - Example: Block<MyHeader, MyExtrinsic> or Block<OtherHeader, OtherExtrinsic>

2. Header<BlockNumber>:
   - Can work with u32, u64, or any other number type
   - Example: Header<u32> or Header<u64>

3. Extrinsic<Caller, Call>:
   - Can work with any caller type (String, u32, AccountId, etc.)
   - Can work with any call type (different pallets have different calls)
   - Example: Extrinsic<String, BalanceCall> or Extrinsic<u32, SystemCall>

4. Dispatch trait:
   - Associated types (type Caller, type Call) let implementers specify their types
   - The trait works the same way regardless of what types are used
   - This means any pallet can implement Dispatch with their own types

WHY USE GENERICS?
- Code reuse: One Block struct works for all blockchain types
- Type safety: Compiler ensures we use the right types
- Flexibility: Easy to change types without rewriting code
- Performance: No runtime overhead - generics are compiled away
*/
