use std::{
    io::{IsTerminal, Read},
    path::Path,
};

use anyhow::bail;
use clap::{Parser, ValueEnum};
use rops::{AgeIntegration, EncryptedFile, FileFormat, JsonFileFormat, RopsFile, RopsFileBuilder, YamlFileFormat};

use crate::*;

pub fn run() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    match args.cmd {
        CliCommand::Encrypt(encrypt_args) => {
            let explicit_file_path = args.file.as_deref();
            let plaintext_string = get_plaintext_string(explicit_file_path)?;

            match get_format(explicit_file_path, args.format)? {
                Format::Yaml => {
                    let encrypted_rops_file = encrypt_rops_file::<YamlFileFormat>(&plaintext_string, encrypt_args)?;
                    println!("{}", encrypted_rops_file);
                }
                Format::Json => {
                    let encrypted_rops_file = encrypt_rops_file::<JsonFileFormat>(&plaintext_string, encrypt_args)?;
                    println!("{}", encrypted_rops_file);
                }
            };

            fn encrypt_rops_file<F: FileFormat>(
                plaintext_str: &str,
                encrypt_args: EncryptArgs,
            ) -> anyhow::Result<RopsFile<EncryptedFile<DefaultCipher, DefaultHasher>, F>> {
                RopsFileBuilder::new(plaintext_str)?
                    .add_integration_keys::<AgeIntegration>(encrypt_args.age_keys)
                    .encrypt()
                    .map_err(Into::into)
            }
        }
        CliCommand::Decrypt(_) => todo!(),
    }

    Ok(())
}

fn get_plaintext_string(file_path: Option<&Path>) -> anyhow::Result<String> {
    let mut stdin_guard = std::io::stdin().lock();

    let plaintext_string = match &file_path {
        Some(plaintext_path) => {
            if !stdin_guard.is_terminal() {
                bail!(RopsCliError::MultipleInputs)
            }
            drop(stdin_guard);
            std::fs::read_to_string(plaintext_path)?
        }
        None => {
            if stdin_guard.is_terminal() {
                bail!(RopsCliError::MissingInput)
            }
            let mut stdin_string = String::new();
            stdin_guard.read_to_string(&mut stdin_string)?;
            stdin_string
        }
    };

    Ok(plaintext_string)
}

fn get_format(explicit_file_path: Option<&Path>, explicit_format: Option<Format>) -> Result<Format, RopsCliError> {
    match explicit_format {
        Some(format) => Ok(format),
        None => match explicit_file_path {
            Some(file_path) => file_path
                .extension()
                .and_then(|file_extension| <Format as ValueEnum>::from_str(file_extension.to_str().expect("invalid unicode"), true).ok())
                .ok_or_else(|| UndeterminedFormatError::NoFileExtention(file_path.to_path_buf()).into()),
            None => Err(UndeterminedFormatError::FoundNeither.into()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_format_by_extesion() {
        assert_eq!(Format::Yaml, get_format(Some(Path::new("test.yaml")), None).unwrap())
    }

    #[test]
    fn infers_format_by_extesion_alias() {
        assert_eq!(Format::Yaml, get_format(Some(Path::new("test.yml")), None).unwrap())
    }

    #[test]
    fn both_missing_is_undetermined_format() {
        assert_eq!(
            RopsCliError::UndeterminedFormat(UndeterminedFormatError::FoundNeither),
            get_format(None, None).unwrap_err()
        )
    }

    #[test]
    fn errors_on_missing_file_extension() {
        assert!(matches!(
            get_format(Some(Path::new("test")), None).unwrap_err(),
            RopsCliError::UndeterminedFormat(UndeterminedFormatError::NoFileExtention(_))
        ))
    }
}
