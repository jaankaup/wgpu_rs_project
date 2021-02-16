#[repr(C)]
#[derive(Clone, Copy)]
pub struct Triangle {
    pub a: Vector3<f32>,
    pub b: Vector3<f32>,
    pub c: Vector3<f32>,
}

impl Triangle {

    pub fn closest_point_to_triangle(&self, p: &Vector3<f32>) -> Vector3<f32> {

        let a = self.a;
        let b = self.b;
        let c = self.c;


        let ab = b - a;
        let ac = c - a;
        let bc = c - b;

        // Surface normal ccw.
        let normal = (b-a).cross(c-a).normalize(); //ac.cross(ab).normalize();

        let snom = (p-a).dot(ab);
        let sdenom = (p-b).dot(a-b);

        let tnom = (p-a).dot(ac);
        let tdenom = (p-c).dot(a-c);

        if (snom <= 0.0 && tnom <= 0.0) {
            let result  = a;
            // let minus = normal.dot(result.normalize());
            // if minus < 0.0 { return (result, false); }
            // else { return (result, true); }
            return result;
        }

        let unom = (p-b).dot(bc);
        let udenom = (p-c).dot(b-c);

        if (sdenom <= 0.0 && unom <= 0.0) {
            let result = b;
            // let minus = normal.dot(result.normalize());
            // if minus < 0.0 { return (result, false); }
            // else { return (result, true); }
            return result;
        }
        if (tdenom <= 0.0 && udenom <= 0.0) {
            let result = c;
            // let minus = normal.dot(result.normalize());
            // if minus < 0.0 { return (result, false); }
            // else { return (result, true); }
            return result;
        }

        let n = (b-a).cross(c-a);
        let vc = n.dot((a-p).cross(b-p));
        if vc <= 0.0 && snom >= 0.0 && sdenom >= 0.0 {
            let result = a + snom / (snom + sdenom) * ab;
            // let minus = normal.dot(result.normalize());
            // if minus < 0.0 { return (result, false); }
            // else { return (result, true); }
            return result;
        }

        let va = n.dot((b-p).cross(c-p));
        if va <= 0.0 && unom >= 0.0 && udenom >= 0.0 {
            let result = b + unom / (unom + udenom) * bc;
            // let minus = normal.dot(result.normalize());
            // if minus < 0.0 { return (result, false); }
            // else { return (result, true); }
            return result;
        }

        let vb = n.dot((c-p).cross(a-p));
        if vb <= 0.0 && tnom >= 0.0 && tdenom >= 0.0 {
            let result = a + tnom / (tnom + tdenom) * ac;
            // let minus = normal.dot(result.normalize());
            // if minus < 0.0 { return (result, false); }
            // else { return (result, true); }
            return result;
        }

        let u = va / (va + vb + vc);
        let v = vb / (va + vb + vc);
        let w = 1.0 - u - v;
        let result = u * a + v * b + w * c;
        // let minus = normal.dot(result.normalize());
        // if minus < 0.0 { return (result, false); }
        // else { return (result, true); }
        result
    }

    pub fn distance_to_triangle(&self, p: &Vector3<f32>) -> (f32, bool) { // (distance, is_positive)
        // TODO: solve sign of distance.
        // Surface normal ccw.
        //let normal = (self.b-self.a).cross(self.c-self.a).normalize(); //ac.cross(ab).normalize();
        let normal = (self.b-self.a).cross(self.c-self.a).normalize(); //ac.cross(ab).normalize();
        let point = self.closest_point_to_triangle(&p);
        let dot_p = normal.dot(point - p);
        let sign = { 
            if dot_p < 0.0 { true } // CHECK THESE 
            else { false }
        };
        //println!("SIGN == {}", sign);
        (point.distance(*p), sign)

        // let (result, sign) = self.closest_point_to_triangle(&p).distance(*p);
        // if sign == false { -result }
        // else { result }
    }

    pub fn divide_triangle_to_points(&self, max_n: u32) -> Vec<Vector3<f32>> {

        let epsilon: f32 = 0.3;

        let mut points: Vec<Vector3<f32>> = Vec::new();
        let ab = self.b - self.a; 
        let ac = self.c - self.a;
        let bc = self.c - self.b;
        // let ab = self.a - self.b; 
        // let ac = self.a - self.c;
        // let bc = self.b - self.c;

        let n = ab.cross(ac).normalize(); 
        let s: f32 = 0.5 as f32 * ab.cross(ac).magnitude(); 
        
        let mut n: u32 = (s/epsilon).sqrt().ceil() as u32;

        if n > max_n { n = max_n; } 

        let s1 = 1.0 / (n as f32) * ab;
        let s2 = 1.0 / (n as f32) * bc;
        let s3 = 1.0 / (n as f32) * ac;

        let mut ps = 1.0/3.0 * (self.a + ( self.a + s1) + (self.a + s3));
        points.push(ps.clone());
        let mut i = 2;

        while i <= n {
            ps = ps + s1;
            points.push(ps.clone());
            let mut p = ps.clone();
            let mut j = 2;

            while j <= i {
                p = p + s2;
                points.push(p.clone());
                j += 1;
            }
            i += 1;
        }

        println!("Points");
        for p in points.iter() {
            println!("Vector {{ x: {}, y: {}, z: {} }}", p.x, p.y, p.z);
        }

        points
    }

//    pub fn to_f32_vec(&self, vt: &VertexType) -> Vec<f32> {
//        let mut result: Vec<f32> = Vec::new();
//        match vt {
//            VertexType::vvv() => {
//                result.push(self.a.x); result.push(self.a.y); result.push(self.a.z);  
//                result.push(self.b.x); result.push(self.b.y); result.push(self.b.z);  
//                result.push(self.c.x); result.push(self.c.y); result.push(self.c.z);  
//            }
//            VertexType::vvvv() => {
//                result.push(self.a.x); result.push(self.a.y); result.push(self.a.z); result.push(1.0);  
//                result.push(self.b.x); result.push(self.b.y); result.push(self.b.z); result.push(1.0);  
//                result.push(self.c.x); result.push(self.c.y); result.push(self.c.z); result.push(1.0);  
//            }
//            VertexType::vvvnnn() => {
//                let n = (self.b-self.a).cross(self.c-self.a).normalize();
//                result.push(self.a.x); result.push(self.a.y); result.push(self.a.z); 
//                result.push(n.x); result.push(n.y); result.push(n.z); 
//                result.push(self.b.x); result.push(self.b.y); result.push(self.b.z); 
//                result.push(n.x); result.push(n.y); result.push(n.z); 
//                result.push(self.c.x); result.push(self.c.y); result.push(self.c.z); 
//                result.push(n.x); result.push(n.y); result.push(n.z); 
//            }
//            VertexType::vvvvnnnn() => {
//                let n = (self.b-self.a).cross(self.c-self.a).normalize();
//                result.push(self.a.x); result.push(self.a.y); result.push(self.a.z); result.push(1.0);  
//                result.push(n.x); result.push(n.y); result.push(n.z); result.push(0.0);  
//                result.push(self.b.x); result.push(self.b.y); result.push(self.b.z); result.push(1.0);  
//                result.push(n.x); result.push(n.y); result.push(n.z); result.push(0.0);  
//                result.push(self.c.x); result.push(self.c.y); result.push(self.c.z); result.push(1.0);  
//                result.push(n.x); result.push(n.y); result.push(n.z); result.push(0.0);  
//            }
//        }
//        result
//    }


    //pub fn create_sample_points(&self, h: f32) -> Vec<Vector3<f32>> {

    //    

    //    let v0 = self.b - self.a;
    //    let v1 = self.c - self.a;
    //    let v2 = p - a;
    //    let d00 = v0.dot(v0);
    //    let d01 = v0.dot(v1);
    //    let d11 = v1.dot(v1);
    //    let d20 = v2.dot(v0);
    //    let d21 = v2.dot(v1);
    //    let denom = d00 * d11 - d01 * d01;
    //    let v = (d11 * d20 - d01 * d21) / denom;
    //    let w = (d00 * d21 - d01 * d20) / denom;
    //    let u = 1.0 - v - w;
    //    let result = Vector3::<f32>::new(u,v,w);
    //    result
    //    }
}

unsafe impl Pod for Triangle {}
unsafe impl Zeroable for Triangle {}

#[derive(Clone, Copy)]
pub struct Plane {
    n: Vector3<f32>,    
    d: f32,    
}

impl Plane {
    pub fn new(a: &Vector3<f32>, b: &Vector3<f32>, c: &Vector3<f32>) -> Self {
        let n = (b-a).cross(c-a).normalize();
        let d = n.dot(*a);
        Self {
            n: n,
            d: d,
        }
    }

    pub fn closest_point_to_plane(self, q: &Vector3<f32>) -> Vector3<f32> {
        let t = (self.n.dot(*q) - self.d) / self.n.dot(self.n);
        q - t * self.n
    }
}


/// Return min vector from a and b components.
fn min_vec(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32> {
    let result = Vector3::<f32>::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    result
}

/// Return max vector from a and b components.
fn max_vec(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32> {
    let result = Vector3::<f32>::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));
    result
}

pub fn barycentric_cooordinates(a: &Vector3<f32>, b: &Vector3<f32>, c: &Vector3<f32>, r: &Vector3<f32>) -> Vector3<f32> {
    let n = (b - a).cross(c - a);
    let rab = n.dot((a-r).cross(b-r));
    let rbc = n.dot((b-r).cross(c-r));
    let rca = n.dot((c-r).cross(a-r));
    let abc = rab + rbc + rca;
    let u = rbc/abc;
    let v = rca/abc;
    let w = rab/abc;
    let result = Vector3::<f32>::new(u,v,w);
    result
}

// /// Compute barycentric coordinates (u, v, w) for point p with respect to triangle (a, b, c)
// pub fn barycentric_cooordinates(a: &Vector3<f32>, b: &Vector3<f32>, c: &Vector3<f32>, p: &Vector3<f32>) -> Vector3<f32> {
//     let v0 = b - a;
//     let v1 = c - a;
//     let v2 = p - a;
//     let d00 = v0.dot(v0);
//     let d01 = v0.dot(v1);
//     let d11 = v1.dot(v1);
//     let d20 = v2.dot(v0);
//     let d21 = v2.dot(v1);
//     let denom = d00 * d11 - d01 * d01;
//     let v = (d11 * d20 - d01 * d21) / denom;
//     let w = (d00 * d21 - d01 * d20) / denom;
//     let u = 1.0 - v - w;
//     let result = Vector3::<f32>::new(u,v,w);
//     result
// }

// pub struct Ray {
//     origin: Vector3<f32>,
//     dir: Vector3<f32>,
// }
