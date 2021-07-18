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
                result.push([coord[0], coord[1], coord[2]-1, -index_offset_z + j as i32]);
                // println!("0 : push {:?}", [coord[0], coord[1], coord[2]-1, index_offset_z + j as i32]);
            }
            1 => { 
                let coord = work_x_index_to_uvec3((j + x_dim*y_dim * 3) as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0], coord[1], coord[2]+1, index_offset_z + (48 + j) as i32]);
                // println!("1 : push {:?}", [coord[0], coord[1], coord[2]+1, -index_offset_z + j as i32]);
            }
            2 => {
                //let index = ((x_dim - 1) * (j+1)) as i32;
                //let index = ((x_dim * y_dim * z_dim - x_dim + 1) + (j*4+3)) as i32;
                let index = (j*4+3) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0]+1, coord[1], coord[2], index_offset_x + index]);
                // println!("2 : push {:?}", [coord[0]+1, coord[1], coord[2], index_offset_x + index]);
            }
            3 => { 
                let index = (x_dim * j) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0]-1, coord[1], coord[2], -index_offset_x + index]);
                // println!("3 : push {:?}", [coord[0]-1, coord[1], coord[2], -index_offset_x + index]);
            }
            4 => {
                let index = (j % y_dim + (j / y_dim) * x_dim*y_dim) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0], coord[1]-1, coord[2], -index_offset_y + index]);
                // println!("4 : push {:?}", [coord[0], coord[1]-1, coord[2], -index_offset_y + index]);
            }

            5 => {
                let index = (x_dim * y_dim - x_dim + j % y_dim + (j / y_dim) * x_dim*y_dim) as i32;
                let coord = work_x_index_to_uvec3(index as u32, x_dim as u32, y_dim as u32, z_dim as u32);
                result.push([coord[0], coord[1]+1, coord[2], index_offset_y + index]);
                // println!("5 : push {:?}", [coord[0], coord[1]+1, coord[2], index_offset_y + index]);
            }
            _ => { panic!("Index out of bounds. >= 5."); }
        }
    }}

    for i in 0..result.len() as usize {
        println!("{:?}",result[i]);
    }


    // for i in 0..result.len() as usize {
    //     print!("{:?} => ",i);
    //     println!("{:?}",reverse_index(&result[i], x_dim, y_dim, z_dim, &result));
    // }

    //println!("result_len == {:?}",result.len());
    result
}

// Return tuple. The first Vec includes the offsets for each data element.
// The second is mapping from translated (+1, +1, +1) coordinates to offset table.
pub fn create_hash_table(x_dim: u32,
                         y_dim: u32,
                         z_dim: u32,
                         global_dim_x: u32,
                         global_dim_y: u32,
                         global_dim_z: u32) -> (Vec<i32>, Vec<u32>, Vec<[i32; 4]>) {

    let table = create_fmm_index_table(x_dim,y_dim,z_dim,global_dim_x,global_dim_y,global_dim_z);
    let mut table_2 = table.to_vec().into_iter();
    let mut table_3 = table.to_vec().into_iter();

    // println!("printing coordinates");
    // for coordinate in table.iter() {
    //     println!("{:?}", coordinate);
    // }
    //let mut table_2 = table.to_vec().into_iter();

    let x_offset = x_dim + 2; 
    let y_offset = y_dim + 2; 
    let z_offset = z_dim + 2; 

    // let xy_size = (x_dim * y_dim * 2) as usize; 
    // let xz_size = (y_dim * z_dim * 2) as usize; 
    // let yz_size = (x_dim * z_dim * 2) as usize; 

    let mut mapping: Vec<i32> = vec![0 ; x_offset as usize * y_offset as usize * z_offset as usize];

    // NOT FINISHED YET!
    // let mut thread_id_mapping: Vec<u32> =
    //     vec![0 ; x_dim as usize * y_dim as usize * z_dim as usize + xy_size + xz_size + yz_size];
    let mut thread_id_mapping: Vec<u32> = vec![777 ; x_offset as usize * y_offset as usize * z_offset as usize];

    let mut indices: Vec<i32> = Vec::new();

    for coordinate in table.iter() {

        let c = [(coordinate[0] + 1) as u32, (coordinate[1] + 1) as u32, (coordinate[2] + 1) as u32];
        let hash_value = c[0] + c[1] * x_offset + c[2] * x_offset * y_offset;
        let result = table_2.find(|&x| x[0] == coordinate[0] && x[1] == coordinate[1] && x[2] == coordinate[2]).unwrap(); 
        mapping[hash_value as usize] = result[3]; 
    }

    // Define regions: center and ghost zones for each side. Ignore corners.

    for coordinate in table.iter() {

        // let mut found_index = 0;
        for i in 0..table.len() {
            let temp = table[i]; 
            if temp[0] == coordinate[0] && temp[1] == coordinate[1] && temp[2] == coordinate[2] {
                let translated_coordinate = [temp[0] + 1, temp[1] + 1, temp[2] + 1, temp[3]];
                // if (translated_coordinase[0] == 0 && translated_coordinase[2] == 0) || 
                //    (translated_coordinase[x_offset-1] == 0 && translated_coordinase[y_offset-1] == 0) || 
                //    (translated_coordinase[0] == 0 && translated_coordinase[2] == 0) || 
                //    (translated_coordinase[x_offset-1] == 0 && translated_coordinase[y_offset-1] == 0) || 
                let translated_hash_index = translated_coordinate[0] + translated_coordinate[1] * x_offset as i32  + translated_coordinate[2] * x_offset as i32 * y_offset as i32; 
                indices.push(translated_hash_index);
                thread_id_mapping[translated_hash_index as usize] = i as u32;
                break;
            }
        }
        // let result = table_3.find(|&x| x[0] == coordinate[0] && x[1] == coordinate[1] && x[2] == coordinate[2]).unwrap(); 

        // let translated_coordinate = [result[0] + 1, result[1] + 1, result[2] + 1, result[3]];
        // let translated_hash_index = translated_coordinate[0] + translated_coordinate[1] * x_offset as i32  + translated_coordinate[2] * x_offset as i32 * y_offset as i32; 
        // indices.push(translated_hash_index);
        // println!("translated_hash_index {:?}", translated_hash_index);

        // // Center.
        // if (translated_coordinate[0] >= 1 && translated_coordinate[0] <= x_dim as i32)
        //     && (translated_coordinate[1] >= 1 && translated_coordinate[1] <= y_dim as i32)
        //     && (translated_coordinate[2] >= 1 && translated_coordinate[2] <= z_dim as i32) {
        //     println!("{:?} is insider", translated_coordinate);
        // }
        // else {
        //     println!("{:?} is outsider", translated_coordinate);
        // }
    }

    indices.sort();

    let mut max_index = 0;

    for index in indices.iter() {
        if index > &max_index { max_index = *index; }
        // println!("index :: {:?}", index);
    }

    for temp in table.iter() {
        let translated_coordinate = [temp[0] + 1, temp[1] + 1, temp[2] + 1, temp[3]];
        let translated_hash_index = translated_coordinate[0] + translated_coordinate[1] * x_offset as i32  + translated_coordinate[2] * x_offset as i32 * y_offset as i32; 
        println!("temp (table) == {:?} table[thread_id_mapping[{:?}] == {:?}", temp, translated_hash_index, table[thread_id_mapping[translated_hash_index as usize] as usize]);
    }

    (mapping, thread_id_mapping, table)
}

// Create hash table for thread index accessing non corned tile indicex.
// pub fn thread_ids_to_hash_table(offset_hash_table: Vec<i32>

// pub fn reverse_index(coordinate: &[i32;4], x_dim: u32, y_dim: u32, z_dim: u32, hash_table: &Vec<[i32;4]>) -> u32 {
// 
//     let x_offset = x_dim + 2; 
//     let y_offset = y_dim + 2; 
// 
//     let mut mapping = [0 as i32 ; 216];
// 
//     let c = [(coordinate[0] + 1) as u32, (coordinate[1] + 1) as u32, (coordinate[2] + 1) as u32];
//     let hash_value = c[0] + c[1] * x_offset + c[2] * x_offset * y_offset;
// 
//     let mut into_iter = hash_table.into_iter();
// 
//     let result = into_iter.find(|&x| x[0] == coordinate[0] && x[1] == coordinate[1] && x[2] == coordinate[2]).unwrap(); 
//     mapping[hash_value as usize] = result[3]; 
// 
//     hash_value
// }
