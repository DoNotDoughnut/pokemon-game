fn main() {
    println!("cargo:rerun-if-changed=assets");

    font_builder::compile("assets/game/fonts", "build/data/fonts.bin");
    #[cfg(feature = "audio")]
    audio_builder::compile("assets/game/music", "build/data/audio.bin");
    let dex = dex_builder::compile(
        "assets/game/pokedex/pokemon",
        "assets/game/pokedex/moves",
        "assets/game/pokedex/items",
        "assets/game/pokedex/trainers",
        Some("build/data/dex.bin"),
        cfg!(feature = "audio"),
    );
    world_builder::compile(dex, "assets/game/world", "build/data/world.bin");

    // embed_resource::compile("build/resources.rc");
    winres::WindowsResource::new()
        .set_icon("build/icon.ico")
        .compile()
        .unwrap();
}
