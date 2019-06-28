//! Immutable or dynamic vertex and index data.

use crate::math::prelude::Aabb3;
use crate::video::assets::shader::Attribute;
use crate::video::errors::{Error, Result};
use crate::video::MAX_VERTEX_ATTRIBUTES;
use smallvec::SmallVec;

impl_handle!(MeshHandle);

/// The setup parameters of mesh object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshParams {
    /// Usage hints.
    pub hint: MeshHint,
    /// How a single vertex structure looks like.
    pub layout: VertexLayout,
    /// Index format
    pub index_format: IndexFormat,
    /// How the input vertex data is used to assemble primitives.
    pub primitive: MeshPrimitive,
    /// The number of vertices in this mesh.
    pub num_verts: usize,
    /// The number of indices in this mesh.
    pub num_idxes: usize,
    /// The start indices of sub-meshes.
    pub sub_mesh_offsets: SmallVec<[usize; 8]>,
    /// Trivial bounding box of vertices.
    pub aabb: Aabb3<f32>,
}

/// Continuous data of vertices and its indices.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshData {
    /// The bytes of vertices.
    pub vptr: Box<[u8]>,
    /// The bytes of indices.
    pub iptr: Box<[u8]>,
}

impl Default for MeshParams {
    fn default() -> Self {
        MeshParams {
            hint: MeshHint::Immutable,
            layout: VertexLayout::default(),
            index_format: IndexFormat::U16,
            primitive: MeshPrimitive::Triangles,
            num_verts: 0,
            num_idxes: 0,
            aabb: Aabb3::zero(),
            sub_mesh_offsets: SmallVec::new(),
        }
    }
}

impl MeshParams {
    pub fn validate(&self, data: Option<&MeshData>) -> Result<()> {
        if let Some(v) = data {
            if v.vptr.len() > self.vertex_buffer_len() {
                return Err(Error::OutOfBounds);
            }

            if v.iptr.len() > self.index_buffer_len() {
                return Err(Error::OutOfBounds);
            }
        }

        for v in &self.sub_mesh_offsets {
            if *v >= self.num_idxes {
                return Err(Error::OutOfBounds);
            }
        }

        Ok(())
    }

    #[inline]
    pub fn vertex_buffer_len(&self) -> usize {
        self.num_verts * self.layout.stride() as usize
    }

    #[inline]
    pub fn index_buffer_len(&self) -> usize {
        self.num_idxes * self.index_format.stride() as usize
    }
}

/// Mesh index.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MeshIndex {
    SubMesh(usize),
    Ptr(usize, usize),
    All,
}

/// Hint abouts the intended update strategy of the data.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum MeshHint {
    /// The resource is initialized with data and cannot be changed later, this
    /// is the most common and most efficient usage. Optimal for render targets
    /// and resourced memory.
    Immutable,
    /// The resource is initialized without data, but will be be updated by the
    /// CPU in each frame.
    Stream,
    /// The resource is initialized without data and will be written by the CPU
    /// before use, updates will be infrequent.
    Dynamic,
}

/// Defines how the input vertex data is used to assemble primitives.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshPrimitive {
    /// Separate points.
    Points,
    /// Separate lines.
    Lines,
    /// Line strips.
    LineStrip,
    /// Separate triangles.
    Triangles,
    /// Triangle strips.
    TriangleStrip,
}

impl MeshPrimitive {
    pub fn assemble(self, indices: u32) -> u32 {
        match self {
            MeshPrimitive::Points => indices,
            MeshPrimitive::Lines => indices / 2,
            MeshPrimitive::LineStrip => indices - 1,
            MeshPrimitive::Triangles => indices / 3,
            MeshPrimitive::TriangleStrip => indices - 2,
        }
    }

    pub fn assemble_triangles(self, indices: u32) -> u32 {
        match self {
            MeshPrimitive::Points | MeshPrimitive::Lines | MeshPrimitive::LineStrip => 0,
            MeshPrimitive::Triangles => indices / 3,
            MeshPrimitive::TriangleStrip => indices - 2,
        }
    }
}

/// Vertex indices can be either 16- or 32-bit. You should always prefer
/// 16-bit indices over 32-bit indices, since the latter may have performance
/// penalties on some platforms, and they take up twice as much memory.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum IndexFormat {
    U16,
    U32,
}

impl IndexFormat {
    pub fn stride(self) -> usize {
        match self {
            IndexFormat::U16 => 2,
            IndexFormat::U32 => 4,
        }
    }

    pub fn encode<T>(values: &[T]) -> &[u8]
    where
        T: Copy,
    {
        let len = values.len() * ::std::mem::size_of::<T>();
        unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
    }
}

/// The data type in the vertex component.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VertexFormat {
    Byte,
    UByte,
    Short,
    UShort,
    Float,
}

/// The details of a vertex attribute.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct VertexAttribute {
    /// The name of this description.
    pub name: Attribute,
    /// The data type of each component of this element.
    pub format: VertexFormat,
    /// The number of components per generic vertex element.
    pub size: u8,
    /// Whether fixed-point data values should be normalized.
    pub normalized: bool,
}

impl Default for VertexAttribute {
    fn default() -> Self {
        VertexAttribute {
            name: Attribute::Position,
            format: VertexFormat::Byte,
            size: 0,
            normalized: false,
        }
    }
}

/// `VertexLayout` defines how a single vertex structure looks like.  A vertex
/// layout is a collection of vertex components, and each vertex component
/// consists of a vertex attribute and the vertex format.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct VertexLayout {
    stride: u8,
    len: u8,
    offset: [u8; MAX_VERTEX_ATTRIBUTES],
    elements: [VertexAttribute; MAX_VERTEX_ATTRIBUTES],
}

impl VertexLayout {
    /// Creates a new an empty `VertexLayoutBuilder`.
    #[inline]
    pub fn build() -> VertexLayoutBuilder {
        VertexLayoutBuilder::new()
    }

    /// Stride of single vertex structure.
    #[inline]
    pub fn stride(&self) -> u8 {
        self.stride
    }

    /// Returns the number of elements in the layout.
    #[inline]
    pub fn len(&self) -> u8 {
        self.len
    }

    /// Checks if the vertex layout is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Relative element offset from the layout.
    pub fn offset(&self, name: Attribute) -> Option<u8> {
        for i in 0..self.elements.len() {
            match self.elements[i].name {
                v if v == name => return Some(self.offset[i]),
                _ => (),
            }
        }

        None
    }

    /// Returns named `Attribute` from the layout.
    pub fn element(&self, name: Attribute) -> Option<VertexAttribute> {
        for i in 0..self.elements.len() {
            match self.elements[i].name {
                v if v == name => return Some(self.elements[i]),
                _ => (),
            }
        }

        None
    }
}

/// Helper structure to build a vertex layout.
#[derive(Default)]
pub struct VertexLayoutBuilder(VertexLayout);

impl VertexLayoutBuilder {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with(
        mut self,
        name: Attribute,
        format: VertexFormat,
        size: u8,
        normalized: bool,
    ) -> Self {
        assert!(size > 0 && size <= 4);

        let desc = VertexAttribute {
            name,
            format,
            size,
            normalized,
        };

        for i in 0..self.0.len {
            let i = i as usize;
            if self.0.elements[i].name == name {
                self.0.elements[i] = desc;
                return self;
            }
        }

        assert!((self.0.len as usize) < MAX_VERTEX_ATTRIBUTES);
        self.0.elements[self.0.len as usize] = desc;
        self.0.len += 1;

        self
    }

    #[inline]
    pub fn finish(mut self) -> VertexLayout {
        self.0.stride = 0;
        for i in 0..self.0.len {
            let i = i as usize;
            let len = self.0.elements[i].size * size_of_vertex(self.0.elements[i].format);
            self.0.offset[i] = self.0.stride;
            self.0.stride += len;
        }
        self.0
    }
}

fn size_of_vertex(format: VertexFormat) -> u8 {
    match format {
        VertexFormat::Byte | VertexFormat::UByte => 1,
        VertexFormat::Short | VertexFormat::UShort => 2,
        VertexFormat::Float => 4,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let layout = VertexLayout::build()
            .with(Attribute::Position, VertexFormat::Float, 3, true)
            .with(Attribute::Texcoord0, VertexFormat::Float, 2, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset(Attribute::Position), Some(0));
        assert_eq!(layout.offset(Attribute::Texcoord0), Some(12));
        assert_eq!(layout.offset(Attribute::Normal), None);

        let element = layout.element(Attribute::Position).unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element(Attribute::Normal), None);
    }

    #[test]
    fn rewrite() {
        let layout = VertexLayout::build()
            .with(Attribute::Position, VertexFormat::Byte, 1, false)
            .with(Attribute::Texcoord0, VertexFormat::Float, 2, true)
            .with(Attribute::Position, VertexFormat::Float, 3, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset(Attribute::Position), Some(0));
        assert_eq!(layout.offset(Attribute::Texcoord0), Some(12));
        assert_eq!(layout.offset(Attribute::Normal), None);

        let element = layout.element(Attribute::Position).unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element(Attribute::Normal), None);
    }
}

#[macro_use]
pub mod macros {
    use super::*;

    #[doc(hidden)]
    #[derive(Default)]
    pub struct CustomVertexLayoutBuilder(VertexLayout);

    impl CustomVertexLayoutBuilder {
        #[inline]
        pub fn new() -> Self {
            Default::default()
        }

        pub fn with(
            &mut self,
            name: Attribute,
            format: VertexFormat,
            size: u8,
            normalized: bool,
            offset_of_field: u8,
        ) -> &mut Self {
            assert!(size > 0 && size <= 4);

            let desc = VertexAttribute {
                name,
                format,
                size,
                normalized,
            };

            for i in 0..self.0.len {
                let i = i as usize;
                if self.0.elements[i].name == name {
                    self.0.elements[i] = desc;
                    return self;
                }
            }

            assert!((self.0.len as usize) < MAX_VERTEX_ATTRIBUTES);
            self.0.offset[self.0.len as usize] = offset_of_field;
            self.0.elements[self.0.len as usize] = desc;
            self.0.len += 1;

            self
        }

        #[inline]
        pub fn finish(&mut self, stride: u8) -> VertexLayout {
            self.0.stride = stride;
            self.0
        }
    }

    #[macro_export]
    macro_rules! impl_vertex {
        ($name: ident { $($field: ident => [$attribute: tt; $format: tt; $size: tt; $normalized: tt],)* }) => (
            #[repr(C)]
            #[derive(Debug, Copy, Clone, Default)]
            pub struct $name {
                $($field: $crate::impl_vertex_field!{VertexFormat::$format, $size}, )*
            }

            impl $name {
                #[allow(dead_code)]
                pub fn new($($field: $crate::impl_vertex_field!{VertexFormat::$format, $size}, ) *) -> Self {
                    $name {
                        $($field: $field,)*
                    }
                }

                #[allow(dead_code)]
                pub fn layout() -> $crate::video::assets::mesh::VertexLayout {
                    let mut builder = $crate::video::assets::mesh::macros::CustomVertexLayoutBuilder::new();

                    $( builder.with(
                        $crate::video::assets::shader::Attribute::$attribute,
                        $crate::video::assets::mesh::VertexFormat::$format,
                        $size,
                        $normalized,
                        $crate::offset_of!($name, $field) as u8); ) *

                    builder.finish(::std::mem::size_of::<$name>() as u8)
                }

                #[allow(dead_code)]
                pub fn attributes() -> $crate::video::assets::shader::AttributeLayout {
                    let builder = $crate::video::assets::shader::AttributeLayoutBuilder::new();

                    $(
                        let builder = builder.with(
                            $crate::video::assets::shader::Attribute::$attribute,
                            $size);
                    ) *

                    builder.finish()
                }

                #[allow(dead_code)]
                pub fn encode(values: &[Self]) -> &[u8] {
                    let len = values.len() * ::std::mem::size_of::<Self>();
                    unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
                }
            }
        )
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! offset_of {
        ($ty:ty, $field:ident) => {{
            use std;
            let ptr: *const $ty = std::ptr::null();
            unsafe { &(*ptr).$field as *const _ as usize }
        }};
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! impl_vertex_field {
        (VertexFormat::Short,1) => {
            i16
        };
        (VertexFormat::UByte,1) => {
            u8
        };
        (VertexFormat::UShort,1) =>{
            u32
        };
        (VertexFormat::Float,1) =>{
            f32
        };
        (VertexFormat::Byte,2) => {
            [i8; 2]
        };
        (VertexFormat::Byte,3) => {
            [i8; 3]
        };
        (VertexFormat::Byte,4) => {
            [i8; 4]
        };
        (VertexFormat::UByte,2) => {
            [u8; 2]
        };
        (VertexFormat::UByte,3) => {
            [u8; 3]
        };
        (VertexFormat::UByte,4) => {
            [u8; 4]
        };
        (VertexFormat::Short,2) => {
            [i16; 2]
        };
        (VertexFormat::Short,3) => {
            [i16; 3]
        };
        (VertexFormat::Short,4) => {
            [i16; 4]
        };
        (VertexFormat::UShort,2) => {
            [u16; 2]
        };
        (VertexFormat::UShort,3) => {
            [u16; 3]
        };
        (VertexFormat::UShort,4) => {
            [u16; 4]
        };
        (VertexFormat::Float,2) => {
            [f32; 2]
        };
        (VertexFormat::Float,3) => {
            [f32; 3]
        };
        (VertexFormat::Float,4) => {
            [f32; 4]
        };
    }

    #[cfg(test)]
    mod test {
        use super::super::*;

        impl_vertex! {
            Vertex {
                position => [Position; Float; 3; false],
                texcoord => [Texcoord0; Float; 2; false],
            }
        }

        impl_vertex! {
            Vertex2 {
                position => [Position; Float; 2; false],
                color => [Color0; UByte; 4; true],
                texcoord => [Texcoord0; Byte; 2; false],
            }
        }

        fn as_bytes<T>(values: &[T]) -> &[u8]
        where
            T: Copy,
        {
            let len = values.len() * ::std::mem::size_of::<T>();
            unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
        }

        #[test]
        fn basic() {
            let layout = Vertex::layout();
            assert_eq!(layout.stride(), 20);
            assert_eq!(layout.offset(Attribute::Position), Some(0));
            assert_eq!(layout.offset(Attribute::Texcoord0), Some(12));
            assert_eq!(layout.offset(Attribute::Normal), None);

            let bytes: [f32; 5] = [1.0, 1.0, 1.0, 0.0, 0.0];
            let bytes = as_bytes(&bytes);
            assert_eq!(
                bytes,
                Vertex::encode(&[Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0])])
            );

            let bytes: [f32; 10] = [1.0, 1.0, 1.0, 0.0, 0.0, 2.0, 2.0, 2.0, 3.0, 3.0];
            let bytes = as_bytes(&bytes);
            assert_eq!(
                bytes,
                Vertex::encode(&[
                    Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0]),
                    Vertex::new([2.0, 2.0, 2.0], [3.0, 3.0])
                ])
            );
        }

        #[test]
        fn representation() {
            let layout = Vertex::layout();
            assert_eq!(layout.stride() as usize, ::std::mem::size_of::<Vertex>());

            let layout = Vertex2::layout();
            let _v = Vertex2::new([1.0, 1.0], [0, 0, 0, 0], [0, 0]);
            let _b = Vertex2::encode(&[]);
            assert_eq!(layout.stride() as usize, ::std::mem::size_of::<Vertex2>());
        }
    }
}
