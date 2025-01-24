pub enum GameRenderLayers {
    Main = 0,
    Terrain = 1,
}

impl Into<usize> for GameRenderLayers {
    fn into(self) -> usize {
        match self {
            GameRenderLayers::Main => 0,
            GameRenderLayers::Terrain => 1,
        }
    }
}
