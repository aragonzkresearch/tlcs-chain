use std::{
    io::Write,
    process::{Command, Stdio},
};

// Functions which wrap a c implementation of tlcs
// This is hacked code used for prototyping - it should not be used in production.

pub fn generate_participant_data(loe_round: u32) -> Vec<u8> {
    let output = Command::new("./src/x/tlcs/crypto/scripts/prover4blockchain.sh")
        .arg(loe_round.to_string())
        .output()
        .expect("the binary + libs + scripts should be installed");

    output.stdout
}

pub fn verify_participant_data(loe_round: u32, participant_data: Vec<u8>) -> bool {
    let mut child = Command::new("./src/x/tlcs/crypto/scripts/verifier4blockchain.sh")
        .arg(loe_round.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the binary + libs + scripts should be installed");

    let child_stdin = child.stdin.as_mut().expect("stdin will always be captured");
    child_stdin
        .write_all(&participant_data)
        .expect("write will always succeed");
    // Close stdin to finish and avoid indefinite blocking
    drop(child_stdin);

    let output = child
        .wait_with_output()
        .expect("will always return a response");

    if String::from_utf8_lossy(&output.stdout) == "1" {
        return true;
    }

    false
}

pub fn aggregate_participant_data(
    number_participants: u32,
    all_participant_data: Vec<u8>,
) -> Vec<u8> {
    let mut command = Command::new("./src/x/tlcs/crypto/scripts/aggregator4blockchain.sh");

    for _ in 1..number_participants {
        command.arg("1");
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the binary + libs + scripts should be installed");

    let child_stdin = child.stdin.as_mut().expect("stdin will always be captured");
    child_stdin
        .write_all(&all_participant_data)
        .expect("write will always succeed");
    // Close stdin to finish and avoid indefinite blocking
    drop(child_stdin);

    let output = child
        .wait_with_output()
        .expect("will always return a response");

    return hex::decode(output.stdout).expect("will return valid hex");
}

pub fn make_secret_key(
    all_participant_data: Vec<u8>,
    number_participants: usize,
    loe_round: u32,
    signature: Vec<u8>,
    public_key: Vec<u8>,
) -> Vec<u8> {
    let mut command = Command::new("./src/x/tlcs/crypto/scripts/invert4blockchain.sh");

    command
        .arg(loe_round.to_string())
        .arg(hex::encode(signature))
        .arg(hex::encode(public_key));

    for _ in 1..number_participants {
        command.arg("1");
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the binary + libs + scripts should be installed");

    let child_stdin = child.stdin.as_mut().expect("stdin will always be captured");
    child_stdin
        .write_all(&all_participant_data)
        .expect("write will always succeed");
    // Close stdin to finish and avoid indefinite blocking
    drop(child_stdin);

    let output = child
        .wait_with_output()
        .expect("will always return a response");

    let mut secret_key = output.stdout; // expect this to be of the form sk:abc..def\n

    secret_key.drain(0..3); //remove sk:
    secret_key.pop(); //remove newline

    return hex::decode(secret_key).expect("will return valid hex");
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn verify_participant_data_works() {
        let participant_data = generate_participant_data(2);
        let verified = verify_participant_data(2, participant_data);

        assert!(verified);
    }

    #[test]
    #[serial]
    fn aggregate_participant_data_works() {
        let mut all_participant_data = generate_participant_data(2);
        let mut participant_data_2 = generate_participant_data(2);
        all_participant_data.append(&mut participant_data_2);
        let public_key = aggregate_participant_data(2, all_participant_data);

        assert!(public_key.len() == 33)
    }

    #[test]
    #[serial]
    fn invert_works() {
        let mut all_participant_data = generate_participant_data(2);
        let mut participant_data_2 = generate_participant_data(2);
        all_participant_data.append(&mut participant_data_2);
        let public_key = aggregate_participant_data(2, all_participant_data.clone());

        // retrieved from https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/2
        let signature = hex::decode("a050676d1a1b6ceedb5fb3281cdfe88695199971426ff003c0862460b3a72811328a07ecd53b7d57fc82bb67f35efaf1").unwrap();

        let secret_key = make_secret_key(all_participant_data, 2, 2, signature, public_key);

        assert!(secret_key.len() == 32)
    }
}