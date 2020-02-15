use crate::constants::{INDEX_DIR, INDEX_HEAP_SIZE};
use anyhow::Result;
use crossbeam::channel::Receiver;
use log::info;
use tantivy::schema::*;
use tantivy::{doc, Index};

/// The tantivy index builder which reads fully qualified
/// file paths from results channel and commits
/// them to index.
pub fn build_index(results: Receiver<String>) -> Result<(), tantivy::TantivyError> {
    info!("Starting indexer");
    let index_dir = INDEX_DIR;
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("full_file_path", TEXT | STORED);

    let schema = schema_builder.build();

    let index = Index::create_in_dir(index_dir, schema.clone())?;

    let mut index_writer = index.writer(INDEX_HEAP_SIZE)?;

    let fpath = match schema.get_field("full_file_path") {
        Some(x) => x,
        None => {
            return Err(tantivy::TantivyError::SchemaError(
                "Unable to get field".to_string(),
            ))
        }
    };

    info!("Iterating over results");
    for file_path in results.iter() {
        index_writer.add_document(doc!(
            fpath =>  file_path,
        ));
    }

    info!("Commiting the index");
    index_writer.commit()?;
    info!("Index created in {:?}", index_dir);

    let num_segments: usize = index.load_metas()?.segments.len();

    info!("Index has {} segments", num_segments);
    Ok(())
}
