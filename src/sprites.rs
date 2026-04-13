/// 8x8 sprite themes, ability icons, state icons, font, and mappings.
///
/// Each sprite frame is 8 strings of 8 chars where:
///   'P' = primary color pixel
///   'S' = skin/secondary color pixel
///   '.' = transparent (black)

use std::sync::LazyLock;

pub type Rgb = (u8, u8, u8);

#[derive(Debug, Clone)]
pub struct SpriteVariant {
    pub primary: Rgb,
    pub skin: Rgb,
    pub frames: Vec<[&'static str; 8]>,
}

#[derive(Debug, Clone)]
pub struct SpriteTheme {
    pub name: &'static str,
    pub sprites: Vec<SpriteVariant>,
}

fn theme(name: &'static str, frames: &[[&'static str; 8]], colors: &[(Rgb, Rgb)]) -> SpriteTheme {
    SpriteTheme {
        name,
        sprites: colors
            .iter()
            .map(|&(p, s)| SpriteVariant {
                primary: p,
                skin: s,
                frames: frames.to_vec(),
            })
            .collect(),
    }
}

pub fn sprite_themes() -> Vec<SpriteTheme> {
    vec![
        // 0: Slimes
        theme(
            "Slimes",
            &[
                ["........", "..PPPP..", ".PPPPPP.", "PPPPPPPP",
                 "PS.PPS.P", "PPPPPPPP", "PPPPPPPP", ".SSSSSS."],
                ["........", "..PPPP..", ".PPPPPP.", "PPPPPPPP",
                 "PS.PPS.P", "PPPPPPPP", "PPPPPPPP", "SSSSSSSS"],
                ["........", "........", "..PPPP..", ".PPPPPP.",
                 "PS.PPS.P", "PPPPPPPP", "PPPPPPPP", "SSSSSSSS"],
                ["........", "..PPPP..", ".PPPPPP.", "PPPPPPPP",
                 "PS.PPS.P", "PPPPPPPP", "PPPPPPPP", ".SSSSSS."],
                ["........", "..PPPP..", ".PPPPPP.", "PPPPPPPP",
                 "PS.PPS.P", ".PPPPPPP", ".PPPPPPP", "..SSSSSS"],
                ["........", "..PPPP..", ".PPPPPP.", "PPPPPPPP",
                 "PS.PPS.P", "PPPPPPP.", "PPPPPPP.", "SSSSSS.."],
            ],
            &[
                ((0,255,65),(0,140,80)),
                ((40,150,255),(20,60,180)),
                ((255,50,120),(180,20,100)),
                ((255,220,0),(200,130,0)),
                ((180,50,255),(100,10,180)),
            ],
        ),
        // 1: Ghosts
        theme(
            "Ghosts",
            &[
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PS.PPS.P",
                 "PPPPPPPP", "PPPPPPPP", "PPPPPPPP", "P.PP.PP."],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PS.PPS.P",
                 "PPPPPPPP", "PPPPPPPP", "PPPPPPPP", ".P.PP.PP"],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PS.PPS.P",
                 "PPPPPPPP", "PPPPPPPP", "PPPPPPPP", "PP.PP.P."],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "P.SPP.SP",
                 "PPPPPPPP", "PPPPPPPP", "PPPPPPPP", "P.PP.PP."],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "P.SPP.SP",
                 "PPPPPPPP", "PPPPPPPP", "PPPPPPPP", ".P.PP.PP"],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "P.SPP.SP",
                 "PPPPPPPP", "PPPPPPPP", "PPPPPPPP", "PP.PP.P."],
            ],
            &[
                ((255,40,60),(255,255,255)),
                ((30,220,255),(255,255,255)),
                ((255,120,255),(255,255,255)),
                ((255,200,20),(255,255,255)),
                ((60,255,80),(255,255,255)),
            ],
        ),
        // 2: Space Invaders (manual — different frame counts per variant)
        SpriteTheme {
            name: "Space Invaders",
            sprites: vec![
                SpriteVariant {
                    primary: (0,255,60), skin: (0,255,60),
                    frames: vec![
                        ["...PP...", "..PPPP..", ".PPPPPP.", ".P.PP.P.",
                         ".PPPPPP.", "..P..P..", ".P....P.", "........"],
                        ["...PP...", "..PPPP..", ".PPPPPP.", ".P.PP.P.",
                         ".PPPPPP.", "..P..P..", "P......P", "........"],
                    ],
                },
                SpriteVariant {
                    primary: (0,255,255), skin: (0,255,255),
                    frames: vec![
                        ["..P..P..", "...PP...", "..PPPP..", ".PP..PP.",
                         "PPPPPPPP", ".P.PP.P.", ".P....P.", "..P..P.."],
                        ["..P..P..", "...PP...", "..PPPP..", ".PP..PP.",
                         "PPPPPPPP", "P.P..P.P", ".P....P.", ".P....P."],
                    ],
                },
                SpriteVariant {
                    primary: (255,0,200), skin: (255,0,200),
                    frames: vec![
                        ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PP.PP.PP",
                         "PPPPPPPP", "..P..P..", ".P.PP.P.", "P......P"],
                        ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PP.PP.PP",
                         "PPPPPPPP", "..P..P..", ".PP..PP.", "........"],
                    ],
                },
                SpriteVariant {
                    primary: (255,220,0), skin: (255,220,0),
                    frames: vec![
                        ["..PPPP..", ".PP..PP.", "PPPPPPPP", "P.PPPP.P",
                         "PPPPPPPP", ".PP..PP.", ".P....P.", "P......P"],
                        ["..PPPP..", ".PP..PP.", "PPPPPPPP", "P.PPPP.P",
                         "PPPPPPPP", ".PP..PP.", "P.P..P.P", "........"],
                    ],
                },
                SpriteVariant {
                    primary: (255,40,40), skin: (255,40,40),
                    frames: vec![
                        ["P......P", ".P....P.", "..PPPP..", ".PPPPPP.",
                         "PP.PP.PP", "PPPPPPPP", ".P.PP.P.", "P......P"],
                        ["P......P", ".P....P.", "..PPPP..", ".PPPPPP.",
                         "PP.PP.PP", "PPPPPPPP", "P.P..P.P", "........"],
                    ],
                },
            ],
        },
        // 3: Pac-Men
        theme(
            "Pac-Men",
            &[
                ["..PPPP..", ".PPPPPP.", "PPSPPPPP", "PPPPPPPP",
                 "PPPP....", "PPPPPPPP", ".PPPPPP.", "..PPPP.."],
                ["..PPPP..", ".PPPPPP.", "PPSPPP..", "PPPPP...",
                 "PPPP....", "PPPPP...", ".PPPPPP.", "..PPPP.."],
                ["..PPPP..", ".PPPPPP.", "PPSPPPPP", "PPPPPPPP",
                 "PPPP....", "PPPPPPPP", ".PPPPPP.", "..PPPP.."],
                ["..PPPP..", ".PPPPPP.", "PPPPPSPP", "PPPPPPPP",
                 "....PPPP", "PPPPPPPP", ".PPPPPP.", "..PPPP.."],
                ["..PPPP..", ".PPPPPP.", "..PPPSPP", "...PPPPP",
                 "....PPPP", "...PPPPP", ".PPPPPP.", "..PPPP.."],
                ["..PPPP..", ".PPPPPP.", "PPPPPSPP", "PPPPPPPP",
                 "....PPPP", "PPPPPPPP", ".PPPPPP.", "..PPPP.."],
            ],
            &[
                ((255,255,0),(20,20,20)),
                ((255,60,160),(20,20,20)),
                ((0,200,255),(20,20,20)),
                ((80,255,40),(20,20,20)),
                ((255,140,0),(20,20,20)),
            ],
        ),
        // 4: Mushrooms
        theme(
            "Mushrooms",
            &[
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "..SSSS..", "..SSSS..", "..SSSS..", "........"],
                [".PPPP...", "PPPPPP..", "P.PP.PP.", "PPPPPPPP",
                 "..SSSS..", "..SSSS..", "..SSSS..", "........"],
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "..SSSS..", "..SSSS..", "........", "........"],
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "..SSSS..", "..SSSS..", "..SSSS..", "........"],
                ["...PPPP.", "..PPPPPP", ".PP.PP.P", "PPPPPPPP",
                 "..SSSS..", "..SSSS..", "..SSSS..", "........"],
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "..SSSS..", "..SSSS..", "........", "........"],
            ],
            &[
                ((255,20,20),(250,235,215)),
                ((20,220,20),(250,235,215)),
                ((255,155,0),(250,235,215)),
                ((30,80,255),(250,235,215)),
                ((230,30,230),(250,235,215)),
            ],
        ),
        // 5: Jumpman
        theme(
            "Jumpman",
            &[
                ["...PPP..", "..PPPPP.", "..SSSS..", "..SS.S..",
                 "..PPPP..", ".PPPPPP.", "..SS.SS.", "........"],
                ["...PPP..", "..PPPPP.", "..SSSS..", "..SS.S..",
                 "..PPPP..", ".PPPPPP.", "...SS.SS", "........"],
                ["........", "...PPP..", "..PPPPP.", "..SSSS..",
                 "..SS.S..", "..PPPP..", ".PPPPPP.", "..SS.SS."],
                ["...PPP..", "..PPPPP.", "..SSSS..", "..SS.S..",
                 "..PPPP..", ".PPPPPP.", "..SS.SS.", "........"],
                ["...PPP..", "..PPPPP.", "..SSSS..", "..SS.S..",
                 "..PPPP..", ".PPPPPP.", ".SS.SS..", "........"],
                ["........", "...PPP..", "..PPPPP.", "..SSSS..",
                 "..SS.S..", "..PPPP..", ".PPPPPP.", "..SS.SS."],
            ],
            &[
                ((255,20,20),(230,180,120)),
                ((40,200,40),(230,180,120)),
                ((255,255,255),(230,180,120)),
                ((255,200,0),(230,180,120)),
                ((60,60,220),(230,180,120)),
            ],
        ),
        // 6: Creepers
        theme(
            "Creepers",
            &[
                ["PPPPPPPP", "PSSPPSSP", "PSSPPSSP", "PPPPPPPP",
                 "PPPSSPPP", "PPSSSSPP", "PPSSSSPP", "PPPPPPPP"],
                ["PPPPPPPP", "PPPPPPPP", "PSSPPSSP", "PPPPPPPP",
                 "PPPSSPPP", "PPSSSSPP", "PPSSSSPP", "PPPPPPPP"],
                ["PPPPPPPP", "PPPPPPPP", "PSSPPSSP", "PPPPPPPP",
                 "PPPSSPPP", "PPSSSSPP", "PPSSSSPP", "PPPPPPPP"],
                ["PPPPPPPP", "PPPPPPPP", "PSSPPSSP", "PPPPPPPP",
                 "PPPPPPPP", "PSPPPPSP", "PPSSSSPP", "PPPPPPPP"],
                ["PPPPPPPP", "PSSPPSSP", "PSSPPSSP", "PPPPPPPP",
                 "PPPPPPPP", "PSPPPPSP", "PPSSSSPP", "PPPPPPPP"],
                ["PPPPPPPP", "PSSPPSSP", "PSSPPSSP", "PPPPPPPP",
                 "PPPSSPPP", "PPSSSSPP", "PPSSSSPP", "PPPPPPPP"],
            ],
            &[
                ((60,230,60),(5,80,5)),
                ((30,210,210),(5,60,60)),
                ((230,60,60),(80,10,10)),
                ((210,170,40),(65,50,5)),
                ((150,60,230),(40,12,75)),
            ],
        ),
        // 7: Frogger
        theme(
            "Frogger",
            &[
                [".PP..PP.", "PPPPPPPP", "PS.PP.SP", "PPPPPPPP",
                 "PPSSSSPP", "PPPPPPPP", ".PPPPPP.", "...PP..."],
                [".PP..PP.", "PPPPPPPP", "PS.PP.SP", "PPPPPPPP",
                 "PPSSSSPP", "PPPPPPPP", ".PPPPPP.", "..PPPP.."],
                [".PP..PP.", "PPPPPPPP", "PS.PP.SP", "PPPPPPPP",
                 "PPSSSSPP", "PPPPPPPP", "PPPPPPPP", ".PPPPPP."],
                [".PP..PP.", "PPPPPPPP", "PS.PP.SP", "PPPPPPPP",
                 "PPSSSSPP", "PPPPPPPP", "PPPPPPPP", ".PPPPPP."],
                [".PP..PP.", "PPPPPPPP", "PS.PP.SP", "PPPPPPPP",
                 "PPSSSSPP", "PPPPPPPP", ".PPPPPP.", "..PPPP.."],
                [".PP..PP.", "PPPPPPPP", "PP.PP.PP", "PPPPPPPP",
                 "PPSSSSPP", "PPPPPPPP", ".PPPPPP.", "...PP..."],
            ],
            &[
                ((20,210,20),(140,255,100)),
                ((40,230,150),(160,255,210)),
                ((90,180,30),(210,250,120)),
                ((20,170,90),(130,240,180)),
                ((100,230,40),(220,255,140)),
            ],
        ),
        // 8: Q*bert
        theme(
            "Q*bert",
            &[
                ["..PPPP..", ".PPPPPP.", "PS.PS.PP", "PPPPPPSS",
                 ".PPPPPP.", "..PPPP..", ".PP..PP.", "........"],
                ["........", "..PPPP..", ".PPPPPP.", "PS.PS.PP",
                 "PPPPPPSS", ".PPPPPP.", "..PPPP..", ".PP..PP."],
            ],
            &[
                ((255,140,0),(220,80,0)),
                ((255,50,50),(200,20,20)),
                ((50,180,255),(15,110,210)),
                ((255,220,30),(230,160,0)),
                ((210,80,255),(155,30,210)),
            ],
        ),
        // 9: Kirby
        theme(
            "Kirby",
            &[
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PP.PP.PP",
                 "PPPPPPPP", ".PPPPPP.", ".SS..SS.", "........"],
                [".PPPP...", "PPPPPP..", "PPPPPPP.", "P.PP.PPP",
                 "PPPPPPPP", ".PPPPPP.", ".SS..SS.", "........"],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PP.PP.PP",
                 "PPPPPPPP", ".PPPPPP.", "SSS..SSS", "........"],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PP.PP.PP",
                 "PPPPPPPP", ".PPPPPP.", ".SS..SS.", "........"],
                ["...PPPP.", "..PPPPPP", ".PPPPPPP", "PPP.PP.P",
                 "PPPPPPPP", ".PPPPPP.", ".SS..SS.", "........"],
                ["..PPPP..", ".PPPPPP.", "PPPPPPPP", "PP.PP.PP",
                 "PPPPPPPP", ".PPPPPP.", "SSS..SSS", "........"],
            ],
            &[
                ((255,140,210),(255,0,20)),
                ((80,180,255),(255,0,20)),
                ((255,60,40),(255,240,0)),
                ((50,230,50),(255,0,20)),
                ((255,255,60),(255,0,20)),
            ],
        ),
        // 10: Zelda Hearts
        theme(
            "Zelda Hearts",
            &[
                [".PP..PP.", "PSSPPPPP", "PSPPPPPP", "PPPPPPPP",
                 ".PPPPPP.", "..PPPP..", "...PP...", "........"],
                [".PP..PP.", "PSPPPPPP", "PSSPPPPP", "PPPPPPPP",
                 ".PPPPPP.", "..PPPP..", "...PP...", "........"],
                [".PP..PP.", "PSSPPPPP", "PPSPPPPP", "PPPPPPPP",
                 ".PPPPPP.", "..PPPP..", "...PP...", "........"],
            ],
            &[
                ((255,0,30),(255,130,140)),
                ((255,90,110),(255,180,190)),
                ((50,50,255),(140,140,255)),
                ((255,215,0),(255,240,130)),
                ((60,255,60),(160,255,160)),
            ],
        ),
        // 11: Tamagotchi
        theme(
            "Tamagotchi",
            &[
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "PP.SS.PP", "PPPPPPPP", "........", ".S....S."],
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "PP.SS.PP", "PPPPPPPP", "........", "S......S"],
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "PPPSPPPP", ".PPPPPP.", "........", ".S....S."],
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "PP.SS.PP", "PPPPPPPP", "........", ".S....S."],
                ["........", "..PPPP..", ".PPPPPP.", "PP.PP.PP",
                 "PPPPPPPP", "PP.SS.PP", ".PPPPPP.", ".S....S."],
                ["..PPPP..", ".PPPPPP.", "PP.PP.PP", "PPPPPPPP",
                 "PP.SS.PP", "PPPPPPPP", "........", "S......S"],
            ],
            &[
                ((240,240,240),(60,60,60)),
                ((255,200,50),(180,100,20)),
                ((100,220,255),(30,100,160)),
                ((255,130,180),(200,50,100)),
                ((150,255,130),(50,160,40)),
            ],
        ),
    ]
}

// -- Static instances (allocated once) --

pub static THEMES: LazyLock<Vec<SpriteTheme>> = LazyLock::new(sprite_themes);

// -- Ability/status icons --

pub struct IconDef {
    pub color: Rgb,
    pub color2: Option<Rgb>,
    pub frames: Vec<[&'static str; 8]>,
}

pub fn ability_defs() -> Vec<IconDef> {
    vec![
        // 0: star
        IconDef {
            color: (255, 220, 30),
            color2: None,
            frames: vec![
                ["...XX...", "..XXXX..", "XXXXXXXX", ".XXXXXX.",
                 "..XXXX..", ".XX..XX.", "XX....XX", "........"],
            ],
        },
        // 1: sword (H=handle color, N=blade color2)
        IconDef {
            color: (140, 90, 40),
            color2: Some((160, 180, 210)),
            frames: vec![
                ["...N....", "..NNN...", "..NNN...", "..NNN...",
                 "..NNN...", ".HHHHH..", "..HHH...", "..HHH..."],
            ],
        },
        // 2: potion
        IconDef {
            color: (180, 60, 255),
            color2: None,
            frames: vec![
                ["...XX...", "..X..X..", "...XX...", "..XXXX..",
                 ".XXXXXX.", ".XXXXXX.", "..XXXX..", "........"],
            ],
        },
        // 3: question block (6-frame animated)
        IconDef {
            color: (255, 200, 40),
            color2: None,
            frames: vec![
                ["XXXXXXXX", "X......X", "XXXX...X", "X..X...X",
                 "X.X....X", "X......X", "X.X....X", "XXXXXXXX"],
                ["XXXXXXXX", "X......X", "X.XXX..X", "X...X..X",
                 "X..X...X", "X......X", "X..X...X", "XXXXXXXX"],
                ["XXXXXXXX", "X......X", "X..XXX.X", "X....X.X",
                 "X...X..X", "X......X", "X...X..X", "XXXXXXXX"],
                ["XXXXXXXX", "X......X", "X...XXXX", "X.....XX",
                 "X....X.X", "X......X", "X....X.X", "XXXXXXXX"],
                ["XXXXXXXX", "X......X", "XX...XXX", "XX.....X",
                 "X.....XX", "X......X", "X.....XX", "XXXXXXXX"],
                ["XXXXXXXX", "X......X", "XXX...XX", "X.X....X",
                 "XX.....X", "X......X", "XX.....X", "XXXXXXXX"],
            ],
        },
        // 4: compass (N=needle color2)
        IconDef {
            color: (40, 100, 255),
            color2: Some((255, 40, 40)),
            frames: vec![
                ["..XXXX..", ".X....X.", "X....N.X", "X...N..X",
                 "X......X", "X......X", ".X....X.", "..XXXX.."],
                ["..XXXX..", ".X....X.", "X......X", "X...NNNX",
                 "X......X", "X......X", ".X....X.", "..XXXX.."],
                ["..XXXX..", ".X....X.", "X......X", "X...NNNX",
                 "X......X", "X......X", ".X....X.", "..XXXX.."],
                ["..XXXX..", ".X....X.", "X....N.X", "X...N..X",
                 "X......X", "X......X", ".X....X.", "..XXXX.."],
                ["..XXXX..", ".X..N.X.", "X...N..X", "X...N..X",
                 "X......X", "X......X", ".X....X.", "..XXXX.."],
                ["..XXXX..", ".X..N.X.", "X...N..X", "X...N..X",
                 "X......X", "X......X", ".X....X.", "..XXXX.."],
            ],
        },
    ]
}

// ZZZ icon (idle state)
pub fn zzz_icon() -> IconDef {
    IconDef {
        color: (60, 130, 255),
        color2: None,
        frames: vec![
            [".....XX.", "......X.", "......XX",
             "........", "........", "...XX...", "....X...", "....XX.."],
            ["........", "........", "........",
             ".....XX.", "......X.", "XX....XX", ".X......", ".XX....."],
            ["........", "........", "XX......",
             ".X......", ".XX.....", "...XX...", "....X...", "....XX.."],
            ["..XX....", "...X....", "...XX...",
             "........", "........", "XX......", ".X......", ".XX....."],
            [".....XX.", "......X.", "XX....XX",
             ".X......", ".XX.....", "........", "........", "........"],
            ["..XX....", "...X....", "...XX...",
             ".....XX.", "......X.", "......XX", "........", "........"],
        ],
    }
}

// FIRE icon (requesting state)
pub fn fire_icon() -> IconDef {
    IconDef {
        color: (220, 30, 0),
        color2: Some((255, 160, 20)),
        frames: vec![
            ["...X....", "..XX....", "..XNXX..", ".XXNNXX.",
             ".XXNNXX.", ".XXNNXX.", "..XNNX..", "...XX..."],
            ["..X.....", "..XXX...", "..XNXX..", ".XXNNXX.",
             ".XXNNXX.", ".XXNNXX.", "..XNNX..", "...XX..."],
            ["..XX....", "..XXX...", ".XXNXX..", ".XXNNXX.",
             ".XXNNXX.", ".XXNNXX.", "..XNNX..", "...XX..."],
            ["....X...", "...XX...", "..XXNX..", ".XXNNXX.",
             ".XXNNXX.", ".XXNNXX.", "..XNNX..", "...XX..."],
            [".....X..", "...XXX..", "..XNXX..", ".XXNNXX.",
             ".XXNNXX.", ".XXNNXX.", "..XNNX..", "...XX..."],
            ["...XX...", "..XXX...", "..XNNX..", ".XXNNXX.",
             ".XXNNXX.", ".XXNNXX.", "..XNNX..", "...XX..."],
        ],
    }
}

pub static ABILITIES: LazyLock<Vec<IconDef>> = LazyLock::new(ability_defs);
pub static ZZZ: LazyLock<IconDef> = LazyLock::new(zzz_icon);
pub static FIRE: LazyLock<IconDef> = LazyLock::new(fire_icon);

// -- 3x5 pixel font --

pub fn font_glyph(ch: char) -> Option<[&'static str; 5]> {
    match ch {
        'A' => Some([".X.", "X.X", "XXX", "X.X", "X.X"]),
        'B' => Some(["XX.", "X.X", "XX.", "X.X", "XX."]),
        'C' => Some([".XX", "X..", "X..", "X..", ".XX"]),
        'D' => Some(["XX.", "X.X", "X.X", "X.X", "XX."]),
        'E' => Some(["XXX", "X..", "XX.", "X..", "XXX"]),
        'F' => Some(["XXX", "X..", "XX.", "X..", "X.."]),
        'G' => Some([".XX", "X..", "X.X", "X.X", ".XX"]),
        'H' => Some(["X.X", "X.X", "XXX", "X.X", "X.X"]),
        'I' => Some(["XXX", ".X.", ".X.", ".X.", "XXX"]),
        'J' => Some(["..X", "..X", "..X", "X.X", ".X."]),
        'K' => Some(["X.X", "X.X", "XX.", "X.X", "X.X"]),
        'L' => Some(["X..", "X..", "X..", "X..", "XXX"]),
        'M' => Some(["X.X", "XXX", "XXX", "X.X", "X.X"]),
        'N' => Some(["X.X", "XXX", "XXX", "X.X", "X.X"]),
        'O' => Some([".X.", "X.X", "X.X", "X.X", ".X."]),
        'P' => Some(["XX.", "X.X", "XX.", "X..", "X.."]),
        'Q' => Some([".X.", "X.X", "X.X", "XX.", ".XX"]),
        'R' => Some(["XX.", "X.X", "XX.", "X.X", "X.X"]),
        'S' => Some([".XX", "X..", ".X.", "..X", "XX."]),
        'T' => Some(["XXX", ".X.", ".X.", ".X.", ".X."]),
        'U' => Some(["X.X", "X.X", "X.X", "X.X", ".X."]),
        'V' => Some(["X.X", "X.X", "X.X", ".X.", ".X."]),
        'W' => Some(["X.X", "X.X", "XXX", "XXX", "X.X"]),
        'X' => Some(["X.X", "X.X", ".X.", "X.X", "X.X"]),
        'Y' => Some(["X.X", "X.X", ".X.", ".X.", ".X."]),
        'Z' => Some(["XXX", "..X", ".X.", "X..", "XXX"]),
        '.' => Some(["...", "...", "...", "...", "X.X"]),
        '0' => Some([".X.", "X.X", "X.X", "X.X", ".X."]),
        '1' => Some([".X.", "XX.", ".X.", ".X.", "XXX"]),
        '2' => Some(["XX.", "..X", ".X.", "X..", "XXX"]),
        '3' => Some(["XX.", "..X", ".X.", "..X", "XX."]),
        '4' => Some(["X.X", "X.X", "XXX", "..X", "..X"]),
        '5' => Some(["XXX", "X..", "XX.", "..X", "XX."]),
        '6' => Some([".XX", "X..", "XX.", "X.X", ".X."]),
        '7' => Some(["XXX", "..X", ".X.", ".X.", ".X."]),
        '8' => Some([".X.", "X.X", ".X.", "X.X", ".X."]),
        '9' => Some([".X.", "X.X", ".XX", "..X", "XX."]),
        _ => None,
    }
}

// -- Tool-to-icon mapping --

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

pub const ICON_STAR: usize = 0;
pub const ICON_SWORD: usize = 1;
pub const ICON_POTION: usize = 2;
pub const ICON_CHEST: usize = 3;
pub const ICON_COMPASS: usize = 4;

pub fn tool_to_icon(tool_name: &str) -> usize {
    if tool_name.is_empty() {
        return ICON_CHEST;
    }
    // Case-insensitive substring match without allocation
    let mappings: &[(&str, usize)] = &[
        ("task", ICON_STAR), ("agent", ICON_STAR), ("spawn", ICON_STAR),
        ("subagent", ICON_STAR), ("dispatch", ICON_STAR),
        ("bash", ICON_SWORD), ("execute", ICON_SWORD), ("computer", ICON_SWORD),
        ("write", ICON_SWORD), ("edit", ICON_SWORD), ("str_replace", ICON_SWORD),
        ("insert", ICON_SWORD), ("create", ICON_SWORD),
        ("think", ICON_POTION), ("reason", ICON_POTION), ("analyze", ICON_POTION),
        ("read", ICON_CHEST), ("memory", ICON_CHEST), ("store", ICON_CHEST),
        ("list", ICON_CHEST), ("glob", ICON_CHEST), ("ls", ICON_CHEST),
        ("search", ICON_COMPASS), ("web", ICON_COMPASS), ("fetch", ICON_COMPASS),
        ("browse", ICON_COMPASS), ("url", ICON_COMPASS), ("http", ICON_COMPASS),
    ];
    for &(keyword, idx) in mappings {
        if contains_ignore_ascii_case(tool_name, keyword) {
            return idx;
        }
    }
    ICON_CHEST
}

// -- Agent colors --

pub const AGENT_COLORS: [Rgb; 10] = [
    (255, 80, 80),    // red
    (80, 255, 80),    // green
    (80, 180, 255),   // blue
    (255, 220, 40),   // yellow
    (255, 100, 255),  // magenta
    (80, 255, 220),   // cyan
    (255, 160, 40),   // orange
    (180, 130, 255),  // lavender
    (255, 255, 255),  // white
    (160, 255, 80),   // lime
];

pub fn agent_color(agent_id: &str) -> Rgb {
    let n: usize = agent_id
        .strip_prefix('P')
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    AGENT_COLORS[n % AGENT_COLORS.len()]
}

// -- Layout constants --

pub const DISPLAY_SIZE: usize = 64;
pub const COL_W: usize = 12;
pub const AGENT_X: [usize; 5] = [0, 13, 26, 39, 52];
pub const HOST_LABEL_Y: usize = 0;
pub const TOP_SPRITE_Y: usize = 6;
pub const TOP_ID_Y: usize = 15;
pub const TOP_STATUS_Y: usize = 22;
pub const BOT_SPRITE_Y: usize = 32;
pub const BOT_ID_Y: usize = 41;
pub const BOT_STATUS_Y: usize = 48;
pub const ICON_DX: usize = 2;

pub struct SlotLayout {
    pub sprite_y: usize,
    pub id_y: usize,
    pub status_y: usize,
}

pub const SLOT_LAYOUTS: [SlotLayout; 2] = [
    SlotLayout { sprite_y: TOP_SPRITE_Y, id_y: TOP_ID_Y, status_y: TOP_STATUS_Y },
    SlotLayout { sprite_y: BOT_SPRITE_Y, id_y: BOT_ID_Y, status_y: BOT_STATUS_Y },
];
