use index_tables::{create_fmm_index_table, create_hash_table};

fn main() {
    // let table = create_fmm_index_table(4,4,4,3,3,3);
    let (mapping,_,_) = create_hash_table(4,4,4,3,3,3);
    println!("heko");
    for x in 0..mapping.len() {
        println!("[{:?}] == {:?}", x, mapping[x]);
    }
}
