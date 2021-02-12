use cpu_version::index_testing::*;

fn main() {

    let array = Array3D::init(10, 10, 10, 0.0);
    let x = 1;
    let y = 2;
    let z = 3;
    let index = array.get_index(x,y,z);
    println!("array.get_index({},{},{}) == {}", x, y, z, array.get_index(x,y,z));
    println!("array.get_3d_index({}) == {:?}", index, array.get_3d_index(123));
}
