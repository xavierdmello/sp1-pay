use sp1_pay_script::fetch_google_jwt_cert;
use sp1_sdk::{HashableKey, ProverClient};
use sp1_helper::build_program;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
///
/// This file is generated by running `cargo prove build` inside the `program` directory.
pub const FIBONACCI_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() {
    // Build program
    build_program("../program");

    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (_, vk) = client.setup(FIBONACCI_ELF);

    let cert = fetch_google_jwt_cert().await.unwrap();
    println!("CERT={:?}\nSP1_PAY_PROGRAM_VKEY={}", cert, vk.bytes32());
}
