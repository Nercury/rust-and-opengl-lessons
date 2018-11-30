use nalgebra as na;
use crate::resources::ResourcePathBuf;

#[derive(Clone, Debug)]
pub struct MeshSet {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub name: Option<String>,
    pub material_index: Option<usize>,
    pub vertices: Vec<Vertex>,
    pub primitives: Vec<Primitive>,
}

#[derive(Clone, Debug)]
pub enum Primitive {
    Triangle(u32, u32, u32),
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: na::Vector3<f32>,
    pub normal: Option<na::Vector3<f32>>,
    pub tangents: Option<Tangents>,
    pub uv: Option<na::Vector2<f32>>,
}

#[derive(Copy, Clone, Debug)]
pub struct Tangents {
    pub tangent: na::Vector3<f32>,
    pub bitangent: na::Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub name: Option<String>,
    pub diffuse_map: Option<ResourcePathBuf>,
    pub bump_map: Option<ResourcePathBuf>,
}

impl Tangents {
    pub fn nans() -> Self {
        Tangents {
            tangent: [::std::f32::NAN, ::std::f32::NAN, ::std::f32::NAN].into(),
            bitangent: [::std::f32::NAN, ::std::f32::NAN, ::std::f32::NAN].into(),
        }
    }

    pub fn from_triangle(
        p0: &na::Vector3<f32>,
        p1: &na::Vector3<f32>,
        p2: &na::Vector3<f32>,
        uv0: &na::Vector2<f32>,
        uv1: &na::Vector2<f32>,
        uv2: &na::Vector2<f32>,
    ) -> Tangents {
        // position differences p1->p2 and p1->p3

        let v = p1 - p0;
        let w = p2 - p0;

        // texture offset p1->p2 and p1->p3
        let mut sx = uv1.x - uv0.x;
        let mut sy = uv0.y - uv1.y;
        let mut tx = uv2.x - uv0.x;
        let mut ty = uv0.y - uv2.y;

        let dir_correction = if (tx * sy - ty * sx) < 0.0 { -1.0 } else { 1.0 };

        // when t1, t2, t3 in same position in UV space, just use default UV direction.
        if sx * ty == sy * tx {
            sx = 0.0;
            sy = 1.0;
            tx = 1.0;
            ty = 0.0;
        }

        let mut tangent = na::Vector3::new(
            (w.x * sy - v.x * ty) * dir_correction,
            (w.y * sy - v.y * ty) * dir_correction,
            (w.z * sy - v.z * ty) * dir_correction,
        );
        let mut bitangent = na::Vector3::new(
            (w.x * sx - v.x * tx) * dir_correction,
            (w.y * sx - v.y * tx) * dir_correction,
            (w.z * sx - v.z * tx) * dir_correction,
        );
        tangent.normalize_mut();
        bitangent.normalize_mut();

        Tangents { tangent, bitangent }
    }
}

impl Mesh {
    pub fn triangle_indices(&self) -> Vec<u32> {
        let mut result = Vec::with_capacity(self.primitives.len() * 3);

        for primitive in self.primitives.iter() {
            match *primitive {
                Primitive::Triangle(a, b, c) => {
                    result.push(a);
                    result.push(b);
                    result.push(c);
                }
            }
        }

        result
    }

    pub fn calculate_tangents(&mut self) {
        for primitive in self.primitives.iter() {
            match *primitive {
                Primitive::Triangle(ai, bi, ci) => {
                    let a = self.vertices[ai as usize];
                    let b = self.vertices[bi as usize];
                    let c = self.vertices[ci as usize];

                    if let (Some(auv), Some(buv), Some(cuv), Some(an), Some(bn), Some(cn)) =
                        (a.uv, b.uv, c.uv, a.normal, b.normal, c.normal)
                    {
                        // this was shamelessly "inspired" by assimp library https://github.com/assimp/assimp/blob/master/code/CalcTangentsProcess.cpp

                        let face_tangent_vectors =
                            Tangents::from_triangle(&a.pos, &b.pos, &c.pos, &auv, &buv, &cuv);

                        fn is_special_float(v: f32) -> bool {
                            v.is_nan() || v.is_infinite()
                        }

                        // for each vertice's (normal, index)...

                        for (n, i) in &[(an, ai), (bn, bi), (cn, ci)] {
                            // project tangent and bitangent into the plane formed by the vertex' normal
                            let mut local_tangent = face_tangent_vectors.tangent
                                - n * face_tangent_vectors.tangent.dot(&n);
                            let mut local_bitangent = face_tangent_vectors.bitangent
                                - n * face_tangent_vectors.bitangent.dot(&n);
                            local_tangent.normalize_mut();
                            local_bitangent.normalize_mut();

                            // reconstruct tangent/bitangent according to normal and bitangent/tangent when it's infinite or NaN.
                            let invalid_tangent = is_special_float(local_tangent.x)
                                || is_special_float(local_tangent.y)
                                || is_special_float(local_tangent.z);
                            let invalid_bitangent = is_special_float(local_bitangent.x)
                                || is_special_float(local_bitangent.y)
                                || is_special_float(local_bitangent.z);

                            if invalid_tangent != invalid_bitangent {
                                if invalid_tangent {
                                    local_tangent = n.cross(&local_bitangent);
                                    local_tangent.normalize_mut();
                                } else {
                                    local_bitangent = local_tangent.cross(&n);
                                    local_bitangent.normalize_mut();
                                }
                            }

                            let ref mut existing_tangent_vectors =
                                self.vertices[*i as usize].tangents;
                            *existing_tangent_vectors = Some(Tangents {
                                tangent: local_tangent,
                                bitangent: local_bitangent,
                            });
                        }
                    }
                }
            }
        }
    }
}
