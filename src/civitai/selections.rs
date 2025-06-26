use anyhow::anyhow;
use dialoguer::{MultiSelect, Select};

use super::{ModelVersionBrief, ModelVersionFile, model};

struct DownloadChoice(u64, String);

impl ToString for DownloadChoice {
    fn to_string(&self) -> String {
        self.1.clone()
    }
}

impl From<(u64, String)> for DownloadChoice {
    fn from(value: (u64, String)) -> Self {
        Self(value.0, value.1)
    }
}

pub fn select_model_version(
    model_meta: &model::Model,
    default_choice_id: Option<u64>,
) -> anyhow::Result<model::ModelVersionBreif> {
    let version_choices = model_meta
        .versions()?
        .iter()
        .map(ModelVersionBrief::choice)
        .map(DownloadChoice::from)
        .collect::<Vec<_>>();

    let default_choice_index = if let Some(default_choice) = default_choice_id {
        version_choices
            .iter()
            .position(|choice| choice.0 == default_choice)
            .unwrap_or(0)
    } else {
        0
    };

    let interact_selection = Select::new()
        .with_prompt("Select the version of model to download ")
        .max_length(7)
        .items(&version_choices)
        .default(default_choice_index)
        .interact()
        .unwrap();

    let selected_version_id = version_choices[interact_selection].0;

    model_meta
        .model_versions
        .iter()
        .find(|version| version.id == selected_version_id)
        .cloned()
        .ok_or(anyhow!("Failed to locate selected model version."))
}

pub fn select_model_version_files(
    selected_version: &model::ModelVersion,
) -> anyhow::Result<Vec<u64>> {
    let file_choices = selected_version
        .files()?
        .iter()
        .map(ModelVersionFile::choice)
        .map(DownloadChoice::from)
        .collect::<Vec<_>>();

    if file_choices.len() == 1 {
        return Ok(file_choices.iter().map(|choice| choice.0).collect());
    }
    let defaultes = file_choices
        .iter()
        .map(|choice| {
            selected_version
                .files
                .iter()
                .find(|file| file.id == choice.0)
                .and_then(|file| file.primary)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    let selected_files = MultiSelect::new()
        .with_prompt("Select files to download ")
        .max_length(7)
        .items(&file_choices)
        .defaults(defaultes.as_slice())
        .interact()
        .unwrap();

    Ok(selected_files
        .iter()
        .map(|index| file_choices[*index].0)
        .collect())
}
