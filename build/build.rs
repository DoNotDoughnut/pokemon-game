fn main() {

    println!("cargo:rerun-if-changed=assets");

    font_builder::compile("assets/fonts", "build/data/fonts.bin");
    #[cfg(feature = "audio")]
    audio_builder::compile("assets/music", "build/data/audio.bin");
    let dex = dex_builder::compile("assets/pokedex/pokemon", "assets/pokedex/moves", "assets/pokedex/items", "assets/pokedex/trainers", "build/data/dex.bin", cfg!(feature = "audio"));
    world_builder::compile(dex, "assets/world", "build/data/world.bin");

    // embed_resource::compile("build/resources.rc");
    winres::WindowsResource::new()
        .set_icon("build/icon.ico")
        .compile().unwrap();

}