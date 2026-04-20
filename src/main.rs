use steam_tui::steam_id;

fn main() {
    println!("steam-tui");
    // Silence dead-code lint until the prompt loop is wired up.
    // Remove once main() actually calls parse_input.
    let _ = steam_id::parse_input;
}
