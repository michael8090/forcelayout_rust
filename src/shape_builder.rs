use lyon::{
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
    },
    path::{path::Builder, FillRule, Path},
};

use crate::{mesh::Mesh, WithId};

pub struct ShapeBuilder {
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
    tolerance: f32,
}

impl ShapeBuilder {
    pub fn new() -> Self {
        ShapeBuilder {
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
            tolerance: 0.02,
        }
    }
    pub fn build_fill<F>(&mut self, id: i32, build: F) -> Mesh
    where
        F: Fn(&mut Builder),
    {
        let mut mesh = Mesh::default();
        let mut builder = Path::builder();
        build(&mut builder);
        let path = builder.build();
        self.fill_tess
            .tessellate_path(
                &path,
                &FillOptions::tolerance(self.tolerance).with_fill_rule(FillRule::NonZero),
                &mut BuffersBuilder::new(&mut mesh.geometry, WithId()),
            )
            .unwrap();

        mesh.id = id;
        mesh
    }

    pub fn build_stroke<F>(&mut self, id: i32, build: F) -> Mesh
    where
        F: Fn(&mut Builder),
    {
        let mut mesh = Mesh::default();
        let mut builder = Path::builder();
        build(&mut builder);
        let path = builder.build();
        self.stroke_tess
            .tessellate_path(
                &path,
                &StrokeOptions::tolerance(self.tolerance),
                &mut BuffersBuilder::new(&mut mesh.geometry, WithId()),
            )
            .unwrap();
        mesh.id = id;
        mesh
    }

    // pub fn build_label<F>(&mut self, id: i32, build: F) -> Mesh
    // where
    //     F: Fn(&mut Builder),
    // {
    //     let mut mesh = Mesh::default();
    //     let mut builder = Path::builder();
    //     build(&mut builder);
    //     let path = builder.build();
    //     self.stroke_tess
    //         .tessellate_path(
    //             &path,
    //             &StrokeOptions::tolerance(self.tolerance),
    //             &mut BuffersBuilder::new(&mut mesh.geometry, WithId()),
    //         )
    //         .unwrap();
    //     mesh.id = id;
    //     mesh
    // }
}
