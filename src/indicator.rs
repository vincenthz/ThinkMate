use iced::{
    advanced::{
        layout::{Limits, Node},
        renderer,
        widget::Tree,
        Clipboard, Layout, Shell, Widget,
    },
    event::Status,
    mouse::Cursor,
    window, Border, Color, Element, Event, Length, Rectangle, Shadow, Size, Vector,
};

pub struct Indicator {
    width: Length,
    height: Length,
    circle_radius: f32,
    color: Color,
}

impl Default for Indicator {
    fn default() -> Self {
        Self {
            width: Length::Fixed(20.0),
            height: Length::Fixed(20.0),
            circle_radius: 2.0,
            color: Color::WHITE,
        }
    }
}

impl Indicator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    #[must_use]
    pub fn circle_radius(mut self, radius: f32) -> Self {
        self.circle_radius = radius;
        self
    }

    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

fn is_visible(bounds: &Rectangle) -> bool {
    bounds.width > 0.0 && bounds.height > 0.0
}

fn fill_circle(
    renderer: &mut impl renderer::Renderer,
    position: Vector,
    radius: f32,
    color: Color,
) {
    if radius > 0. {
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: position.x,
                    y: position.y,
                    width: radius * 2.0,
                    height: radius * 2.0,
                },
                border: Border {
                    radius: radius.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
            },
            color,
        );
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Indicator
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(limits.width(self.width).height(self.height).resolve(
            self.width,
            self.height,
            Size::new(f32::INFINITY, f32::INFINITY),
        ))
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if !is_visible(&bounds) {
            return;
        }

        let size = if bounds.width < bounds.height {
            bounds.width
        } else {
            bounds.height
        } / 2.0;
        let center = bounds.center();
        let distance_from_center = size - self.circle_radius;
        let position = Vector::new(
            center.x + distance_from_center - self.circle_radius,
            center.y + distance_from_center - self.circle_radius,
        );

        fill_circle(renderer, position, self.circle_radius, self.color);
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        _cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let bounds = layout.bounds();

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            if is_visible(&bounds) {
                return Status::Captured;
            }
        }

        Status::Ignored
    }
}

impl<'a, Message, Theme, Renderer> From<Indicator> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
{
    fn from(indicator: Indicator) -> Self {
        Self::new(indicator)
    }
}
