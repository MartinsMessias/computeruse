//! Example 08: Audio & Volume Control
//! Demonstrates reading and modifying master volume, checking mute status, and toggling mute.

use computeruse::ComputerUse;

fn main() -> computeruse::Result<()> {
    println!("=== 08: AUDIO & VOLUME ===");

    // 1. Get current volume level (0-100)
    match ComputerUse::audio_get_volume() {
        Ok(vol) => {
            println!("Current master volume: {}%", vol);

            // 2. Adjust volume and restore
            println!("Setting volume to 80%...");
            let _ = ComputerUse::audio_set_volume(80);

            let new_vol = ComputerUse::audio_get_volume()?;
            println!("New master volume:     {}%", new_vol);

            println!("Restoring original volume to {}%...", vol);
            ComputerUse::audio_set_volume(vol)?;
        }
        Err(e) => println!("Audio endpoint not available: {}", e),
    }

    // 3. Mute status
    match ComputerUse::audio_get_mute() {
        Ok(muted) => {
            println!("\nCurrent mute status:   {}", if muted { "Muted" } else { "Unmuted" });

            // Toggle mute
            let toggled = ComputerUse::audio_toggle_mute()?;
            println!("Toggled mute status:   {}", if toggled { "Muted" } else { "Unmuted" });

            // Restore original mute state
            ComputerUse::audio_set_mute(muted)?;
            println!("Restored mute status:  {}", if muted { "Muted" } else { "Unmuted" });
        }
        Err(e) => println!("Audio mute query not available: {}", e),
    }

    println!("\nAudio operations test completed!");
    Ok(())
}
