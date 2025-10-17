use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program_id_str = if args.len() > 1 {
        &args[1]
    } else {
        "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp"
    };

    let program_id = Pubkey::from_str(program_id_str).expect("Invalid program ID");

    let (pool_pda, _) = Pubkey::find_program_address(&[b"pool"], &program_id);
    let (roots_ring_pda, _) = Pubkey::find_program_address(&[b"roots_ring"], &program_id);
    let (nullifier_shard_pda, _) = Pubkey::find_program_address(&[b"nullifier_shard"], &program_id);
    let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], &program_id);

    println!("Program ID: {}", program_id);
    println!("Pool PDA: {}", pool_pda);
    println!("Roots Ring PDA: {}", roots_ring_pda);
    println!("Nullifier Shard PDA: {}", nullifier_shard_pda);
    println!("Treasury PDA: {}", treasury_pda);
}
