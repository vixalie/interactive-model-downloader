use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct DownloadOptions {
    #[arg(help = "The model detail page URL.")]
    pub url: String,
    #[arg(
        short = 'o',
        long = "output",
        help = "The directory stores the download files.",
        default_value = "."
    )]
    pub output_path: PathBuf,
    #[arg(
        long = "fix-missing",
        short = 'f',
        help = "Fix missing directories.",
        default_value = "false"
    )]
    pub fix_missing_dirs: bool,
}

pub fn process_download_options(options: &DownloadOptions) {
    let target_url = reqwest::Url::parse(&options.url).expect("The given url is invalid.");

    let target_platform = crate::downloader::detect_platform(&target_url);

    match target_platform {
        Some(crate::downloader::Platform::Civitai) => {
            println!("Downloading from Civitai...");
            if !crate::configuration::check_civitai_key_exists() {
                println!("Civitai access key is not set. Please set it first.");
                return;
            }
            let (model_id, model_version_id) =
                match crate::civitai::try_parse_civitai_model_url(&target_url) {
                    Ok(result) => result,
                    Err(error) => {
                        panic!("{}", error);
                    }
                };
            println!(
                "Preceeding to download model {model_id}, version id {}",
                model_version_id.unwrap_or("[UNSET]".to_string())
            );
        }
        Some(crate::downloader::Platform::HuggingFace) => {
            if !crate::configuration::check_huggingface_key_exists() {
                println!("HuggingFace API key is not set. Please set it first.");
                return;
            }
            println!("Downloading from HuggingFace is not supported yet.");
        }
        _ => {
            println!("Unsupported platform.");
        }
    }
}
