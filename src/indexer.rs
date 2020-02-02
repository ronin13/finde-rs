use crossbeam::channel::Receiver;
use tantivy::schema::*;
use tantivy::{doc,Index};
use crate::constants::INDEX_DIR;

pub fn build_index(results: Receiver<String>) -> Result<(), tantivy::TantivyError> {

    println!("Starting indexer");
    let index_dir = INDEX_DIR;
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("full_file_path", TEXT | STORED);

    let schema = schema_builder.build();

    let index = Index::create_in_dir(index_dir, schema.clone())?;

    let mut index_writer = index.writer(50_000_000)?;

    let fpath = schema.get_field("full_file_path").unwrap();


    for file_path in results.iter() {
        index_writer.add_document(doc!(
            fpath =>  file_path,
        ));

    }

    index_writer.commit()?;
    println!("Index created in {:?}", index_dir);

    let num_segments:usize = index.load_metas()?.segments.len();

    println!("Index has {} segments", num_segments);
    Ok(())

}