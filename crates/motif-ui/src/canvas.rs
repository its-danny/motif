use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Geometry, Path, Stroke};
use iced::{Element, Fill, Point, Rectangle, Renderer, Theme};

use crate::app::Message;
use crate::theme;

const PIXELS_PER_BEAT: f32 = 100.0;
const LANE_HEIGHT: f32 = 24.0;
const BEATS_PER_BAR: u32 = 4;

pub struct PianoRollGrid {
    grid_cache: Cache,
}

impl Default for PianoRollGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl PianoRollGrid {
    pub fn new() -> Self {
        Self {
            grid_cache: Cache::new(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
    }
}

impl canvas::Program<Message> for PianoRollGrid {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let grid = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            // Background — zinc-950
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::ZINC_950);

            // Horizontal lane dividers — alternating subtle stripe
            let mut lane: u32 = 0;
            loop {
                let y = lane as f32 * LANE_HEIGHT;
                if y > bounds.height {
                    break;
                }

                // Alternate lane backgrounds for depth
                if lane % 2 == 1 {
                    frame.fill_rectangle(
                        Point::new(0.0, y),
                        iced::Size::new(bounds.width, LANE_HEIGHT),
                        theme::ZINC_900,
                    );
                }

                let line = Path::line(Point::new(0.0, y), Point::new(bounds.width, y));
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(theme::ZINC_800),
                );

                lane += 1;
            }

            // Vertical beat lines
            let mut beat: u32 = 0;
            loop {
                let x = beat as f32 * PIXELS_PER_BEAT;
                if x > bounds.width {
                    break;
                }

                let is_bar = beat.is_multiple_of(BEATS_PER_BAR);
                let (color, width) = if is_bar {
                    (theme::ZINC_700, 1.0)
                } else {
                    (theme::ZINC_800, 1.0)
                };

                let line = Path::line(Point::new(x, 0.0), Point::new(x, bounds.height));
                frame.stroke(&line, Stroke::default().with_width(width).with_color(color));

                beat += 1;
            }
        });

        vec![grid]
    }
}
