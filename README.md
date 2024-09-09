# Secure Binder

SecureBinder is a critical component responsible for securely retrieving encrypted data from Encrypted Data Vaults (EDVs) and transforming it into a format suitable for oblivious transfer (OT) in garbled circuits. It ensures the integrity and confidentiality of data throughout the process by validating, transforming, and securely transmitting the data between components using advanced cryptographic techniques.

### Key Modules

- core/: Handles core logic for data validation, transformation, and binding.
- interfaces/: Manages interactions with the EDV and Oblivious Transfer protocol.
- infrastructure/: Contains cryptography functions and secure data handling services.
- tests/: Includes unit and integration tests for each feature, along with mock implementations.

## Getting Started

To get started with SecureBinder, first ensure you have Rust installed on your machine. If Rust is not installed, you can do so via [rustup](https://rustup.rs/).

```sh
# Clone the repository
git clone git@github.com:Gateway-DAO/secure-binder.git

# Navigate to the project directory
cd secure-binder

# Build the project
cargo build
```

### Running the Unit Tests

Since SecureBinder is built using a TDD approach, unit tests are a critical part of the project. You can run the tests as follows:

```sh
cargo test
```

### Usage

Once installed and built, Secure Binder can be used in conjunction with an EDV and garbled circuits, integrating with real-time data streams from an EDV.

```sh
cargo run
```

#### SecureBinder will:

- Retrieve data from the EDV using secure communication.
- Validate the data to ensure correctness.
- Transform the data into a format for use in the OT protocol.
- Securely transmit the data for further cryptographic processing.

## Architecture Overview

- Core Layer: Contains business logic and core functionality (like data transformation and OT).
- Interfaces Layer: Deals with external interactions (like the EDV and network).
- Infrastructure Layer: Deals with things like cryptography, storage, and secure communication.
- Tests Layer: Contains unit tests, integration tests, and mocks.

### Folder Structure

```
SecureBinder/
│
├── src/
│   ├── core/                        # Core business logic (e.g., validation, transformation)
│   ├── interfaces/                  # Interfaces for EDV, OT protocol, and communication
│   ├── infrastructure/              # Infrastructure services (cryptography, storage)
│   ├── tests/                       # Unit and integration tests, including mocks
│   ├── main.rs                      # Main entry point for the application
│   └── lib.rs                       # Library entry point
├── Cargo.toml                        # Rust dependencies and project metadata
└── README.md                         # Project README
```

## Contributing

We welcome contributions from the community! To contribute to SecureBinder, follow these steps:

```sh
#Fork the repository.

#Create a new branch
git checkout -b feature-branch

#Commit your changes
git commit -am 'Add new feature'

#Push to the branch
git push origin feature-branch

#Create a new pull request
Open a pull request.
```

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.
