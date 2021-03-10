use lyon::{lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator}, path::{FillRule, Path, path::Builder}};

use crate::{WithId, mesh::Mesh};

pub fn build_fill<F>(id: i32, build: F) -> Mesh 
where F: Fn(&mut Builder) {
    let mut fill_tess = FillTessellator::new();
    let tolerance = 0.02;

    let mut mesh = Mesh::default();
    let mut builder = Path::builder();
    build(&mut builder);
    let path = builder.build();
    fill_tess.tessellate_path(
        &path,
        &FillOptions::tolerance(tolerance).with_fill_rule(FillRule::NonZero),
        &mut BuffersBuilder::new(&mut mesh.geometry, WithId(id)),
    ).unwrap();
    mesh.id = id;
    mesh
}

pub fn build_stroke<F>(id: i32,build: F) -> Mesh 
where F: Fn(&mut Builder) {
    let mut stroke_tess = StrokeTessellator::new();
    let tolerance = 0.02;

    let mut mesh = Mesh::default();
    let mut builder = Path::builder();
    build(&mut builder);
    let path = builder.build();
    stroke_tess.tessellate_path(
        &path,
        &StrokeOptions::tolerance(tolerance),
        &mut BuffersBuilder::new(&mut mesh.geometry, WithId(id)),
    ).unwrap();
    mesh.id = id;
    mesh
}
