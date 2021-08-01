use index_tables::{create_fmm_index_table, create_hash_table};

fn index_to_uvec3(the_index: u32, dim_x: u32, dim_y: u32) -> [u32 ; 3] {
  let mut index = the_index;
  let wh = dim_x * dim_y;
  let z = index / wh;
  index -= z * wh;
  let y = index / dim_x;
  index -= y * dim_x;
  let x = index;
  [x, y, z]
}

fn main() {
    // let table = create_fmm_index_table(4,4,4,3,3,3);
    let (mapping,yhyy,jooo) = create_hash_table(4,4,4,3,3,3);
    println!("heko");
    //for x in 0..mapping.len() {
    //    println!("[{:?}] == {:?}", x, mapping[x]);
    //}
    for x in 0..jooo.len() {
        println!("[{:?}] == {:?}", x, jooo[x]);
    }

    let x_dim = 16;
    let y_dim = 1;
    let z_dim = 16;

    let total = x_dim * y_dim * z_dim;

    for i in 0..total {
        println!("index_to_uvec3({}, {}, {})) -> {:?}", i, 16, 1, index_to_uvec3(i, 16, 1)); 
    }

}
