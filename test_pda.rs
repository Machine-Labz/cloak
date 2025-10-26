use solana_sdk::pubkey::Pubkey;

fn main() {
    let program_id = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".parse::<Pubkey>().unwrap();
    
    // Old PDA (without mint)
    let (old_pda, _) = Pubkey::find_program_address(&[b"pool"], &program_id);
    
    // New PDA (with mint = Pubkey::default())
    let mint = Pubkey::default();
    let (new_pda, _) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], &program_id);
    
    println!("Program ID: {}", program_id);
    println!("Old PDA (without mint): {}", old_pda);
    println!("New PDA (with mint): {}", new_pda);
    println!("Different: {}", old_pda != new_pda);
    println!("Mint (default): {}", mint);
}
