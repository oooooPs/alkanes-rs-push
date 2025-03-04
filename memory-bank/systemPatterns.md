# ALKANES-RS System Patterns

## System Architecture

ALKANES-RS follows a modular architecture organized as a Rust workspace with multiple crates, each serving a specific purpose in the overall system. The architecture can be broken down into several key layers:

### 1. Indexer Layer

The top-level crate serves as the main indexer implementation for the METASHREW environment. It processes Bitcoin blocks, extracts ALKANES protocol messages, and updates the system state.

Key components:
- **Block Processor**: Parses Bitcoin blocks and transactions
- **Message Extractor**: Identifies and validates ALKANES protocol messages
- **State Manager**: Updates the global state based on transaction processing

### 2. Runtime Layer

The runtime layer provides the execution environment for ALKANES smart contracts. It handles contract instantiation, execution, and state management.

Key components:
- **WebAssembly VM**: Executes compiled smart contracts
- **Fuel Metering**: Tracks and limits computation resources
- **Context Management**: Provides execution context for contracts

### 3. Standard Library Layer

The standard library layer provides pre-built smart contracts for common use cases, serving as both examples and building blocks for more complex applications.

Key components:
- **Token Standards**: Implementations of token interfaces
- **Authentication**: Access control mechanisms
- **Proxy Patterns**: Contract delegation and upgradeability
- **DeFi Primitives**: AMM pools, factories, and other financial components

### 4. Support Layer

The support layer provides shared utilities and interfaces used across the system.

Key components:
- **Protocol Definitions**: Data structures and constants
- **Serialization Utilities**: Encoding and decoding functions
- **Storage Abstractions**: Key-value storage interfaces
- **Error Handling**: Standardized error types and reporting

## Key Design Patterns

### 1. Table Relationships in Protorune

The protorune system uses several key tables to track rune ownership and balances:

- **OUTPOINT_TO_RUNES**: Maps Bitcoin outpoints to their rune balances
  - Populated during normal transaction indexing
  - Used by functions like `protorunes_by_address` to check balances
  - Updated whenever a transaction affects rune balances

- **RUNE_ID_TO_OUTPOINTS**: Maps rune IDs to the outpoints that hold them
  - Populated by the `add_rune_outpoint` function
  - Called during runestone processing and edict handling
  - Used by functions like `protorune_holders` to find holders of a specific rune
  - Only populated for transactions recognized as runestones

- **OUTPOINTS_FOR_ADDRESS**: Maps addresses to their associated outpoints
  - Populated during normal transaction indexing in `index_spendables`
  - Used by functions like `protorunes_by_address` to find outpoints for an address
  - Updated for all transactions, not just runestones

- **OUTPOINT_SPENDABLE_BY**: Maps outpoints to the addresses that can spend them
  - Populated during normal transaction indexing
  - Used to determine which address owns an outpoint

These table relationships are critical for the proper functioning of view functions like `protorune_holders` and `protorunes_by_address`. The key insight is that different tables are populated through different paths:

- `OUTPOINTS_FOR_ADDRESS` and `OUTPOINT_SPENDABLE_BY` are populated for ALL transactions with valid addresses
- `RUNE_ID_TO_OUTPOINTS` is only populated for runestone transactions or when edicts are processed

### 2. Alkanes and AlkaneId

Alkanes are executable programs in the ALKANES metaprotocol that also function as transferable assets:

- **Dual Nature**: Alkanes are both executable programs and transferable assets
- **Asset Transfer**: They conform to a standard of behavior for asset transfer consistent with runes
- **Balance Sheet**: They can hold a balance sheet of alkanes the way that a UTXO can
- **Storage and Execution**: They have the ability to read/write to storage slots they own and execute against other alkanes

**AlkaneId Structure**:
- Alkanes are addressed by their AlkaneId (same structure as ProtoruneRuneId)
- Their addresses are always `[2, n]` or `[3, n]`, where n is a u128 value
- `[2, 0]` is a special address for the genesis ALKANE with incentives for block optimization
- The ALKANES metaprotocol is instantiated with the creation of ALKANE at `[2, 0]`
- Alkanes created with `[1, 0]`, `[5, n]`, or `[6, n]` acquire a `[2, n]` address, where n is the current sequence number
- The sequence number in the `[2, n]` identifier increases by 1 for each new alkane instantiated
- Since `[2, 0]` is already taken by the genesis ALKANE, the first available sequence number for new alkanes would be 1
- If an alkane is instantiated with the `[3, n]` cellpack, the value of n can be any u128 value that has not already been taken, and the address will be `[4, n]`

**Important**: The `block` parameter in AlkaneId is NOT the same as the block height of the chain. It's a sequence number or a specific u128 value used for addressing.

### 2. Cellpack Structure

Cellpacks are protomessages that interact with alkanes:

- **Format**: A cellpack is a protomessage whose calldata is an encoded list of leb128 varints
- **Header**: The first two varints are either the AlkaneId targeted with the protomessage, or a pair of varints with special meanings
- **Inputs**: The remaining varints after the target are considered inputs to the alkane
- **Opcodes**: By convention, the first input after the cellpack target is interpreted as an opcode
- **Initialization**: The 0 opcode following the cellpack target should call the initialization logic for the alkane

### 3. Standard Contract Dependencies

Some standard contracts in ALKANES have specific dependencies and requirements:

#### Owned Token Contracts
- **Auth Token Dependency**: Owned token contracts must be deployed with an auth token
- **Initialization Process**:
  - When an owned token is initialized (opcode 0), it automatically deploys an auth token
  - The auth token is used for ownership verification through the `only_owner()` method
  - Without a properly initialized auth token, owned token operations will revert
- **Implementation**:
  - Owned tokens implement the `AuthenticatedResponder` trait
  - This trait provides methods for deploying auth tokens and checking ownership
  - The auth token ID is stored in the `/auth` storage pointer of the owned token

#### Live Environment vs Test Environment
- In live environments, DIESEL (a specific alkane) is always deployed at `[2, 0]`
- In test environments, tokens can be deployed at `[2, 0]` for testing purposes
- This difference must be considered when writing and testing contracts

### 3. Trait-Based Abstraction

ALKANES-RS makes extensive use of Rust traits to define interfaces and behavior:

- **AlkaneResponder Trait**: Core interface for smart contract implementation
  ```rust
  pub trait AlkaneResponder {
      fn context(&self) -> Result<Context>;
      fn execute(&self) -> Result<CallResponse>;
      // Additional methods...
  }
  ```

- **Token Trait**: Interface for token standard implementation
  ```rust
  pub trait Token {
      fn name(&self) -> String;
      fn symbol(&self) -> String;
  }
  ```

- **Extcall Trait**: Interface for external contract calls
  ```rust
  pub trait Extcall {
      fn __call(cellpack: i32, outgoing_alkanes: i32, checkpoint: i32, fuel: u64) -> i32;
      fn call(cellpack: &Cellpack, outgoing_alkanes: &AlkaneTransferParcel, fuel: u64) -> Result<CallResponse>;
  }
  ```

This trait-based approach enables polymorphism and code reuse while maintaining type safety.

### 2. Message Context Pattern

The system uses a message context pattern to encapsulate transaction data and execution environment:

```rust
#[derive(Clone, Default)]
pub struct AlkaneMessageContext(());

impl MessageContext for AlkaneMessageContext {
    fn protocol_tag() -> u128 {
        1
    }
    fn handle(_parcel: &MessageContextParcel) -> Result<(Vec<RuneTransfer>, BalanceSheet)> {
        // Implementation...
    }
}
```

This pattern allows for:
- Clean separation of protocol-specific logic
- Consistent handling of messages across different contexts
- Extension through trait implementation

### 3. WASM-based Smart Contracts

Smart contracts in ALKANES are compiled to WebAssembly, providing:

- **Portability**: Contracts can be executed in any environment with a WASM runtime
- **Security**: Sandboxed execution prevents many common vulnerabilities
- **Performance**: Near-native execution speed with controlled resource usage

The `declare_alkane!` macro simplifies contract implementation:

```rust
#[macro_export]
macro_rules! declare_alkane {
    ($struct_name:ident) => {
        #[no_mangle]
        pub extern "C" fn __execute() -> i32 {
            let mut response = to_arraybuffer_layout(&$struct_name::default().run());
            Box::leak(Box::new(response)).as_mut_ptr() as usize as i32 + 4
        }
    };
}
```

### 4. Storage Abstraction

The system provides a key-value storage abstraction for contract state:

```rust
pub trait AlkaneResponder {
    fn load(&self, k: Vec<u8>) -> Vec<u8>;
    fn store(&self, k: Vec<u8>, v: Vec<u8>);
    // Other methods...
}
```

This pattern:
- Simplifies state management for contract developers
- Provides a consistent interface across different storage backends
- Enables efficient state caching and batching

### 5. Fuel Metering

Computation is metered using a fuel system to prevent DoS attacks:

```rust
pub fn index_block(block: &Block, height: u32) -> Result<()> {
    // ...
    FuelTank::initialize(&block);
    // ...
}
```

The FuelTank manages fuel allocation and consumption:
- Block-level fuel allocation
- Transaction-level fuel tracking
- Automatic refueling between transactions

### 6. Protocol Extension

ALKANES is designed as an extension of the protorunes protocol:

```rust
pub fn index_block(block: &Block, height: u32) -> Result<()> {
    // ...
    Protorune::index_block::<AlkaneMessageContext>(block.clone(), height.into())?;
    // ...
}
```

This pattern allows:
- Leveraging existing protocol infrastructure
- Maintaining compatibility with the base protocol
- Adding specialized functionality while preserving core behavior

## Component Relationships

### Indexer and Runtime Interaction

The indexer processes blocks and extracts messages, which are then passed to the runtime for execution:

1. Indexer identifies ALKANES messages in transactions
2. Messages are parsed and validated
3. Runtime executes the corresponding contract code
4. State changes are recorded and persisted

### Contract Interaction Model

Contracts interact with each other through a message-passing model:

1. Caller contract prepares a message with parameters and tokens
2. Runtime delivers the message to the target contract
3. Target contract executes and returns a response
4. Runtime updates state based on the execution result

### Network Configuration

The system supports multiple networks through feature flags and configuration:

```rust
#[cfg(feature = "mainnet")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("bc"),
        p2sh_prefix: 0x05,
        p2pkh_prefix: 0x00,
    });
}
```

This allows for:
- Network-specific parameters
- Conditional compilation for different targets
- Consistent behavior across networks

## Error Handling Strategy

ALKANES-RS uses Rust's `anyhow` for error handling:

```rust
fn execute(&self) -> Result<CallResponse> {
    match operation {
        // ...
        _ => Err(anyhow!("unrecognized opcode"))
    }
}
```

This provides:
- Rich error context
- Propagation of errors across boundaries
- Consistent error reporting

## Testing Approach

The system employs multiple testing strategies:

1. **Unit Tests**: Testing individual components in isolation
2. **Integration Tests**: Testing interactions between components
3. **End-to-End Tests**: Testing the full system with simulated blocks
4. **WASM Tests**: Testing the compiled WASM contracts

This comprehensive approach ensures correctness at all levels of the system.