use lyon::{
    geom::Point,
    lyon_tessellation::{
        geometry_builder::simple_builder, StrokeOptions, StrokeTessellator, VertexBuffers,
    },
    path::{path::Builder, Path},
};

pub struct Canvas {}

pub struct Line {
    builder: Builder,
    pub buffers: Option<VertexBuffers<Point<f32>, u16>>,
}

impl Line {
    pub fn start(x: f32, y: f32) -> Line {
        let mut builder = Path::builder();
        builder.begin(Point::new(x, y));
        Line {
            builder,
            buffers: None,
        }
    }

    pub fn to(&mut self, x: f32, y: f32) {
        self.builder.line_to(Point::new(x, y));
    }

    pub fn end(mut self, canvas: &mut Canvas) {
        self.builder.end(true);
        let path = self.builder.build();
        let mut buffers: VertexBuffers<Point<f32>, u16> = VertexBuffers::new();
        {
            let mut vertex_builder = simple_builder(&mut buffers);
            let mut tessellator = StrokeTessellator::new();
            let _ = tessellator.tessellate(&path, &StrokeOptions::default(), &mut vertex_builder);
        }
        self.buffers = Some(buffers);
        println!("{:?}", self.buffers);
    }
}

// TODO
// pub struct Rect {}
// pub fn draw_rect(canvas: &mut Canvas, rect: &Rect) {}
//
// pub struct Texture {}
// pub fn draw_texture(canvas: &mut Canvas, texture: &Texture) {}
//
// pub struct Text {}
// pub fn draw_text(canvas: &mut Canvas, text: &Text) {}
