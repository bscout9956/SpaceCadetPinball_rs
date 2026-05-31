const CONTROLLER_DB: &[u8] = include_bytes!("gamecontrollerdb.txt");

pub fn load_controller_db(sdl_context: &sdl2::Sdl) -> Result<(), String> {
    let rw = sdl2::rwops::RWops::from_bytes(CONTROLLER_DB)
        .map_err(|e| format!("Failed to load control RWops: {}", e))?;

    let game_controller_subsystem = sdl_context.game_controller()?;

    game_controller_subsystem
        .load_mappings_from_rw(rw)
        .map_err(|e| format!("Failed to load control RWops: {}", e))?;

    Ok(())
}
