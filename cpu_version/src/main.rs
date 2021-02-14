use cpu_version::index_testing::*;

fn main() {

    let x_dim = 10;
    let y_dim = 5;
    let z_dim = 8;

    let array = Array3D::init(x_dim, y_dim, z_dim, 0.0);

    // for i in 0..x_dim*y_dim*z_dim {
    //     println!("array.get_3d_index({}) == {:?}", i, array.get_3d_index(i));
    // }

    // for i in 0..x_dim {
    // for j in 0..y_dim {
    // for k in 0..z_dim {
    //     let index1D = array.get_index(i,j,k);
    //     let index3D = array.get_3d_index(index1D);
    //     println!();
    //     println!("array.get_index({},{},{}) == {}", i, j, k, array.get_index(i,j,k));
    //     println!("array.get_3d_index({}) == {:?}", index1D, array.get_3d_index(index1D));
    //     assert!((i, j, k) == index3D, "Failed!"); 
    // }}};

    // let x = 1;
    // let y = 2;
    // let z = 3;
    // let index = array.get_index(x,y,z);
    // println!("array.get_index({},{},{}) == {}", x, y, z, array.get_index(x,y,z));
    // println!("array.get_3d_index({}) == {:?}", index, array.get_3d_index(123));
}
