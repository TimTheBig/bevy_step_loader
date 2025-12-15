#![warn(clippy::all)]

use thiserror::Error;

use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    mesh::{Indices, Mesh, VertexAttributeValues},
    prelude::*,
    render::render_resource::PrimitiveTopology, tasks::AsyncComputeTaskPool,
};
use foxtrot_step::step_file::StepFile;

pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<StepLoader>();
    }
}

#[derive(Default)]
pub struct StepLoader;

impl AssetLoader for StepLoader {
    type Asset = Mesh;
    type Settings = ();
    type Error = StepError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        #[allow(unused_variables)] load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(AsyncComputeTaskPool::try_get().ok_or(StepError::NoAsyncComputeTaskPool)?
        .spawn(async move {
            let flatten = StepFile::strip_flatten(&bytes);
            let step = StepFile::parse(&flatten);

            step_to_triangle_mesh(&step)
        }).await)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["step", "stp"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum StepError {
    #[error("Failed to load STEP")]
    Io(#[from] std::io::Error),
    #[error("AsyncComputeTaskPool must be initialized before loading a STEP model")]
    NoAsyncComputeTaskPool
}

fn step_to_triangle_mesh(step: &StepFile) -> Mesh {
    let (mesh, _) = foxtrot_tri::triangulate::triangulate(step);

    let mut bevy_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    let vertex_count = mesh.triangles.len();

    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(vertex_count);

    for (i, face) in mesh.triangles.iter().enumerate() {
        for (j, &vert) in face.verts.iter().enumerate() {
            let vertex = mesh.verts[vert as usize];

            positions.push(vertex.pos.cast().into());
            normals.push(vertex.norm.cast().into());
            colors.push(vertex.color.cast().into());

            indices.push((i * 3 + j) as u32);
        }
    }

    let uvs = vec![[0.0, 0.0]; vertex_count];

    bevy_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    bevy_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    bevy_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x3(colors),
    );

    bevy_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(uvs)
    );
    bevy_mesh.insert_indices(Indices::U32(indices));

    bevy_mesh
}
