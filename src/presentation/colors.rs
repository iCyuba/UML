use vello::peniko::Color;

pub struct Colors {
    pub workspace_background: Color,
    pub workspace_dot: Color,
    pub workspace_text: Color,

    pub toolbox_background: Color,
    pub toolbox_border: Color,
    pub accent: Color,
    pub icon_active: Color,
    pub icon_inactive: Color,
    pub drop_shadow: Color,
}

impl Colors {
    pub const LIGHT: Colors = Colors {
        workspace_background: Color::WHITE,
        workspace_dot: Color::rgb8(203, 213, 225),
        workspace_text: Color::BLACK,

        toolbox_background: Color::rgb8(255, 255, 255),
        toolbox_border: Color::rgb8(230, 230, 230),
        accent: Color::rgb8(13, 153, 255),
        icon_active: Color::WHITE,
        icon_inactive: Color::BLACK,
        drop_shadow: Color::rgba8(0, 0, 0, 60),
    };

    pub const DARK: Colors = Colors {
        workspace_background: Color::rgb8(24, 24, 27),
        workspace_dot: Color::rgb8(63, 63, 70),
        workspace_text: Color::WHITE,

        toolbox_background: Color::rgb8(44, 44, 44),
        toolbox_border: Color::rgb8(68, 68, 68),
        accent: Color::rgb8(12, 140, 233),
        icon_active: Color::WHITE,
        icon_inactive: Color::WHITE,
        drop_shadow: Color::rgba8(0, 0, 0, 200),
    };
}
