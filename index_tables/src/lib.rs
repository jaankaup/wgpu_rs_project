// For given block index,
pub fn work_x_index_to_uvec3(work_index: u32, dim_x: u32, dim_y: u32, dim_z: u32) -> [i32;3] {

    let mut result: [i32;3] = [0;3];
    let mut index = work_index;
    let wh = dim_x * dim_y;
    let z = index / wh;
    index -= z * wh;
    let y = index / dim_x;
    index -= y * dim_x;
    let x = index;
    return [x as i32, y as i32, z as i32];
}

pub fn create_fmm_index_table(
    x_dim: u32,
    y_dim: u32,
    z_dim: u32,
    global_dim_x: u32,
    global_dim_y: u32,
    global_dim_z: u32) -> Vec<[i32;4]> {

    // Local offsets.
    let y_offset: i32 = x_dim as i32; 
    let z_offset: i32 = (x_dim * y_dim) as i32; 

    let block_size = x_dim * y_dim * z_dim;

    let index_offset_z: i32 = (block_size * global_dim_x * global_dim_y - x_dim * y_dim) as i32;
    let index_offset_y: i32 = (block_size * global_dim_x - (x_dim * y_dim - x_dim)) as i32;
    let index_offset_x: i32 = (1 + (block_size - x_dim)) as i32;

    let mut result: Vec<[i32;4]> = Vec::new();

    // Compute indices for the actual block data (not neighbor data).
    for i in 0..block_size as i32  {
        let coordinate = work_x_index_to_uvec3(i as u32, x_dim as u32, y_dim as u32, z_dim as u32);
        result.push([coordinate[0], coordinate[1], coordinate[2], i]);
    }

    // Compute the ghost zone data.
    for i in 0..6 {
    for j in 0..x_dim*y_dim {
        match i {
            0 => { 
                let coord = work_x_index_to_uvec3(j as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0], coord[1], coord[2]-1, index_offset_z + j as i32]);
            }
            1 => { 
                let coord = work_x_index_to_uvec3((j + x_dim*y_dim * 3) as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0], coord[1], coord[2]+1, -index_offset_z + j as i32]);
            }
            2 => {
                let index = ((x_dim - 1) * (j+1)) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0]+1, coord[1], coord[2], index_offset_x + index]);
            }
            3 => { 
                let index = (x_dim * j) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0]-1, coord[1], coord[2], -index_offset_x + index]);
            }
            4 => {
                let index = (j % y_dim + (j / y_dim) * x_dim*y_dim) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0], coord[1]-1, coord[2], -index_offset_y + index]);
            }

            5 => {
                let index = (x_dim * y_dim - x_dim + j % y_dim + (j / y_dim) * x_dim*y_dim) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0], coord[1]+1, coord[2], index_offset_y + index]);
            }
            _ => { result.push([66666,66666,666666,666666]); }
        }
    }}

    for i in 0..result.len() as usize {
        println!("{:?}",result[i]);
    }

    // for i in 0..result.len() as usize {
    //     print!("{:?} => ",i);
    //     println!("{:?}",reverse_index(&result[i], x_dim, y_dim, z_dim, &result));
    // }

    println!("result_len == {:?}",result.len());
    result
}

// Create offset table depending on global and local dimensions.
// TODO: implement local hashing on GPU code.
pub fn create_hash_table(x_dim: u32,
                         y_dim: u32,
                         z_dim: u32,
                         global_dim_x: u32,
                         global_dim_y: u32,
                         global_dim_z: u32) -> Vec<i32> {

    let table = create_fmm_index_table(x_dim,y_dim,z_dim,global_dim_x,global_dim_y,global_dim_z);
    let mut table_2 = table.to_vec().into_iter();

    let x_offset = x_dim + 2; 
    let y_offset = y_dim + 2; 
    let z_offset = z_dim + 2; 

    let mut mapping: Vec<i32> = vec![0 ; x_offset as usize * y_offset as usize * z_offset as usize]; // [0 as i32 ; 216];

    for coordinate in table.iter() {

        let c = [(coordinate[0] + 1) as u32, (coordinate[1] + 1) as u32, (coordinate[2] + 1) as u32];
        let hash_value = c[0] + c[1] * x_offset + c[2] * x_offset * y_offset;
        let result = table_2.find(|&x| x[0] == coordinate[0] && x[1] == coordinate[1] && x[2] == coordinate[2]).unwrap(); 
        mapping[hash_value as usize] = result[3]; 
    }

    mapping
}

pub fn reverse_index(coordinate: &[i32;4], x_dim: u32, y_dim: u32, z_dim: u32, hash_table: &Vec<[i32;4]>) -> u32 {

    let x_offset = x_dim + 2; 
    let y_offset = y_dim + 2; 

    let mut mapping = [0 as i32 ; 216];

    let c = [(coordinate[0] + 1) as u32, (coordinate[1] + 1) as u32, (coordinate[2] + 1) as u32];
    let hash_value = c[0] + c[1] * x_offset + c[2] * x_offset * y_offset;

    let mut into_iter = hash_table.into_iter();

    let result = into_iter.find(|&x| x[0] == coordinate[0] && x[1] == coordinate[1] && x[2] == coordinate[2]).unwrap(); 
    mapping[hash_value as usize] = result[3]; 

    hash_value
}
