use solana_sdk::pubkey::Pubkey;

fn main() {
    let program_id = "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".parse::<Pubkey>().unwrap();
    let mint = Pubkey::default(); // Native SOL
    
    println!("Program ID: {}", program_id);
    println!("Mint (default): {}", mint);
    
    // Derive PDA the same way the test does
    let (pool_pda, pool_bump) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], &program_id);
    println!("Pool PDA: {}", pool_pda);
    println!("Pool bump: {}", pool_bump);
    
    // Derive PDA the same way the program does
    let (pool_pda2, pool_bump2) = Pubkey::find_program_address(&[b"pool", mint.as_ref()], &program_id);
    println!("Pool PDA (program way): {}", pool_pda2);
    println!("Pool bump (program way): {}", pool_bump2);
    
    println!("Match: {}", pool_pda == pool_pda2);
}
