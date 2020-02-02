extern crate tantivy;

use crossbeam::channel::Receiver;
// use tantivy::collector::TopDocs;
// use tantivy::query::QueryParser;
use tantivy::schema::*;
// use tantivy::{doc,Index, ReloadPolicy};
use tantivy::{doc,Index};
// use tantivy::error;


pub fn build_index(results: Receiver<String>) -> Result<(), tantivy::TantivyError> {

    println!("Starting indexer");
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("rpath", TEXT | STORED);

    let schema = schema_builder.build();

    let index = Index::create_in_dir("/tmp/index", schema.clone())?;

    let mut index_writer = index.writer(50_000_000)?;

    let fpath = schema.get_field("rpath").unwrap();

    // let mut fpath_doc = Document::default();

    for fl in results.iter() {
        println!("Adding {}", fl);
        index_writer.add_document(doc!(
            fpath =>  fl,
        ));

    }

    index_writer.commit()?;

    Ok(())


}