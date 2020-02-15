use crate::constants::INDEX_HEAP_SIZE;
use anyhow::Result;
use crossbeam::channel::Receiver;
use log::info;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs;
use tantivy::schema::*;
use tantivy::{doc, Index};

/// The tantivy index builder which reads fully qualified
/// file paths from results channel and commits
/// them to index.
pub fn build_index(
    results: Receiver<String>,
    _index_dir: String,
) -> Result<(), tantivy::TantivyError> {
    info!("Starting indexer");
    let index_dir = _index_dir;
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("full_file_path", TEXT | STORED);

    let schema = schema_builder.build();

    let rng = thread_rng();
    let index_suffix = rng.sample_iter(&Alphanumeric).take(5).collect::<String>();

    let index_directory = format!("{}/{}", &index_dir, index_suffix);
    fs::create_dir(&index_directory)?;
    info!("Index directory created in {}", &index_directory);

    let index = Index::create_in_dir(&index_directory, schema.clone())?;

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
