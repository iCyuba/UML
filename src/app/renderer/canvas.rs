use crate::presentation::Colors;

pub struct Canvas {
    // The canvas properties can only be accessed by renderers.
    pub(super) size: (u32, u32),
    pub(super) scale: f64,
    pub(super) colors: &'static Colors,
    pub(super) scene: vello::Scene,
}

impl Canvas {
    #[inline]
    pub fn reset(&mut self) {
        self.scene.reset();
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    #[inline]
    pub fn scale(&self) -> f64 {
        self.scale
    }

    #[inline]
    pub fn colors(&self) -> &'static Colors {
        self.colors
    }

    #[inline]
    pub fn scene(&mut self) -> &mut vello::Scene {
        &mut self.scene
    }
}

impl AsRef<Canvas> for Canvas {
    #[inline]
    fn as_ref(&self) -> &Canvas {
        self
    }
}

impl AsRef<vello::Scene> for Canvas {
    #[inline]
    fn as_ref(&self) -> &vello::Scene {
        &self.scene
    }
}
