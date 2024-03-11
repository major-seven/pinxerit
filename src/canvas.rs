use lyon::{
    geom::Point,
    lyon_tessellation::{
        geometry_builder::simple_builder, StrokeOptions, StrokeTessellator, VertexBuffers,
    },
    path::{path::Builder, Path},
};

use crate::tessellate::TessellateVertex;

pub struct Canvas {
    pub tessellates: Vec<Tessellate>,
}

impl Canvas {
    pub fn new() -> Canvas {
        Canvas {
            tessellates: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Tessellate {
    pub vertices: Vec<TessellateVertex>,
    pub indices: Vec<u16>, 
}

pub struct Line {
    builder: Builder,
    color: [f32; 4],
}

impl Line {
    pub fn start(x: f32, y: f32, color: [f32; 4]) -> Line {
        let mut builder = Path::builder();
        builder.begin(Point::new(x, y));
        Line {
            builder,
            color,
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
            let stroke_options = StrokeOptions::default().with_line_width(0.01);
            let _ = tessellator.tessellate(
                &path,
                &stroke_options,
                &mut vertex_builder).unwrap();
        }
    
            let pad = buffers.indices.len() % 4;
            for _ in 0..pad {
                buffers.indices.push(buffers.indices.last().unwrap().clone());
            }
        canvas.tessellates.push(Tessellate {
            vertices: buffers.vertices.iter().map(|v| {
                TessellateVertex {
                    color: self.color,
                    position: [v.x, v.y, 0.1],
                }
            }).collect(),
            indices: buffers.indices.to_vec(),
        });
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
