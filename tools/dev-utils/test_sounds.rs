#!/usr/bin/env cargo +stable

//! Test the dynamic sound loading system

use vaughan::gui::utils::{get_available_sounds, get_sound_display_name, play_notification_sound_by_name, list_available_sounds};

fn main() {
    println!("🎵 Testing Dynamic Sound System");
    println!("===============================");

    // List all available sounds
    list_available_sounds();

    let sounds = get_available_sounds();

    if !sounds.is_empty() {
        println!("\n🎯 Testing sounds (press Ctrl+C to stop):");

        for (i, sound) in sounds.iter().enumerate() {
            println!("\n{}. Testing '{}' ({})", i + 1, get_sound_display_name(sound), sound);

            match play_notification_sound_by_name(sound) {
                Ok(_) => println!("   ✅ Played successfully"),
                Err(e) => println!("   ❌ Failed to play: {}", e),
            }

            // Wait between sounds
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        println!("\n🎉 Sound testing complete!");
        println!("\n💡 To add new sounds:");
        println!("   1. Place .wav files in config/sounds/");
        println!("   2. Restart the wallet");
        println!("   3. Your new sounds will be automatically detected!");
    }
}