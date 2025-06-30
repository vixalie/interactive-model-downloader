use std::path::{Path, PathBuf};

use clap::Args;

#[derive(Args, Default)]
pub struct RenewOptions {
    #[arg(help = "The model file request to renew metadata.")]
    pub target_file: PathBuf,
}

fn is_legal_model_file<P: AsRef<Path>>(file_path: P) -> bool {
    let extensions = vec!["ckpt", "safetensors", "pt", "bin"];
    let file_extension = file_path.as_ref().extension();
    if file_extension.is_none() {
        return false;
    }
    let file_extension = file_extension.unwrap().to_string_lossy();
    extensions
        .iter()
        .any(|ext| ext.eq_ignore_ascii_case(&file_extension))
}

pub async fn process_model_meta_renew(options: &RenewOptions) {
    println!("Note: This feature only supports updating models downloaded from Civitai.com.");

    if !options.target_file.is_file() || !is_legal_model_file(&options.target_file) {
        println!("The target file must be a model file.");
        return;
    }

    let civitai_client = crate::downloader::make_client()
        .await
        .expect("failed to initialize client");

    crate::civitai::complete_file_meta(&civitai_client, &options.target_file)
        .await
        .expect("Failed to retreive tearget file metadata");
    println!("All Done.");
}
