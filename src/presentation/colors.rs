use vello::peniko::Color;

pub struct Colors {
    pub workspace_background: Color,
    pub workspace_dot: Color,
    pub workspace_text: Color,
}

impl Colors {
    pub const LIGHT: Colors = Colors {
        workspace_background: Color::WHITE,
        workspace_dot: Color::rgb8(203, 213, 225),
        workspace_text: Color::BLACK,
    };

    pub const DARK: Colors = Colors {
        workspace_background: Color::rgb8(24, 24, 27),
        workspace_dot: Color::rgb8(63, 63, 70),
        workspace_text: Color::WHITE,
    };
}
