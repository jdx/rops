use std::path::Path;

use clap::{ArgAction, Args};

use crate::*;

#[derive(Args)]
pub struct EncryptArgs {
    #[command(flatten)]
    pub integration_keys: IntegrationKeys,
    #[command(flatten)]
    pub partial_encryption_args: Option<PartialEncryptionArgs>,
    /// Requires a partial encryption setting
    #[arg(long, display_order = 11, requires = "partial_encryption", action(ArgAction::SetTrue))]
    pub mac_only_encrypted: Option<bool>,
    #[command(flatten)]
    pub input_args: InputArgs,
    #[arg(long, short, requires = "file", action(ArgAction::SetTrue), display_order = 0)]
    /// Encrypt file in place rather than printing the result to stdout.
    pub in_place: Option<bool>,
}

impl ConfigArg for EncryptArgs {
    fn config_path(&self) -> Option<&Path> {
        self.input_args.config_path()
    }
}

impl MergeConfig for EncryptArgs {
    fn merge_config(&mut self, config: Config) {
        for creation_rule in config.creation_rules {
            if let Some(file_path) = &self.input_args.file {
                if creation_rule.path_regex.is_match(&file_path.to_string_lossy()) {
                    self.integration_keys.merge(creation_rule.integration_keys);

                    if self.mac_only_encrypted.is_none() {
                        self.mac_only_encrypted = creation_rule.mac_only_encrypted;
                    }

                    if self.partial_encryption_args.is_none() {
                        if let Some(partial_encryption_config) = creation_rule.partial_encryption {
                            self.partial_encryption_args = Some(partial_encryption_config.into());
                        }
                    }

                    break;
                }
            }
        }
    }
}

#[cfg(feature = "test-utils")]
mod mock {
    use rops::*;

    use super::*;

    impl MockTestUtil for EncryptArgs {
        fn mock() -> Self {
            Self {
                integration_keys: MockTestUtil::mock(),
                partial_encryption_args: None,
                mac_only_encrypted: None,
                input_args: MockTestUtil::mock(),
                in_place: None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rops::*;

    use super::*;

    #[test]
    fn merges_integration_keys_from_config() {
        let mut encrypted_args = EncryptArgs::mock();
        assert_eq!(1, encrypted_args.integration_keys.age.len());
        encrypted_args.merge_config(Config::mock_other());
        assert_eq!(2, encrypted_args.integration_keys.age.len());
    }

    #[test]
    fn merges_partial_encryption_from_config() {
        let mut encrypted_args = EncryptArgs::mock();
        assert!(encrypted_args.partial_encryption_args.is_none());
        encrypted_args.merge_config(Config::mock());
        assert_eq!(
            PartialEncryptionConfig::mock(),
            encrypted_args.partial_encryption_args.unwrap().into()
        );
    }

    #[test]
    fn merges_mac_only_encrypted_from_config() {
        let mut encrypted_args = EncryptArgs::mock();
        assert!(encrypted_args.mac_only_encrypted.is_none());
        encrypted_args.merge_config(Config::mock_other());
        assert!(encrypted_args.mac_only_encrypted.is_some());
    }
}
