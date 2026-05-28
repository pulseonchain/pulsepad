#[test]
fn print_discriminators() {
    use anchor_lang::Discriminator;

    // In anchor 0.29, we can check the program ID
    println!("Program ID: {}", cto_bonding::ID.to_string());
}
