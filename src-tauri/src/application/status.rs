use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ApplicationError {
    #[error("database is unavailable")]
    DatabaseUnavailable,
    #[error("library configuration could not be read")]
    ConfigurationUnavailable,
}

pub(crate) trait DatabaseHealth {
    fn check_health(&self) -> Result<(), ApplicationError>;
}

pub(crate) trait LibraryConfiguration {
    fn has_configured_library(&self) -> Result<bool, ApplicationError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlatformInfo {
    pub(crate) os: &'static str,
    pub(crate) architecture: &'static str,
    pub(crate) supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApplicationStatus {
    pub(crate) database_healthy: bool,
    pub(crate) library_configured: bool,
    pub(crate) platform: PlatformInfo,
}

pub(crate) struct GetApplicationStatus<'a, Health, Configuration> {
    health: &'a Health,
    configuration: &'a Configuration,
}

impl<'a, Health, Configuration> GetApplicationStatus<'a, Health, Configuration>
where
    Health: DatabaseHealth,
    Configuration: LibraryConfiguration,
{
    pub(crate) fn new(health: &'a Health, configuration: &'a Configuration) -> Self {
        Self {
            health,
            configuration,
        }
    }

    pub(crate) fn execute(&self) -> Result<ApplicationStatus, ApplicationError> {
        self.health.check_health()?;
        let library_configured = self.configuration.has_configured_library()?;
        let os = std::env::consts::OS;
        let architecture = std::env::consts::ARCH;

        Ok(ApplicationStatus {
            database_healthy: true,
            library_configured,
            platform: PlatformInfo {
                os,
                architecture,
                supported: matches!(
                    (os, architecture),
                    ("windows", "x86_64") | ("macos", "x86_64")
                ),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HealthyDatabase;

    impl DatabaseHealth for HealthyDatabase {
        fn check_health(&self) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    struct NoLibrary;

    impl LibraryConfiguration for NoLibrary {
        fn has_configured_library(&self) -> Result<bool, ApplicationError> {
            Ok(false)
        }
    }

    #[test]
    fn returns_no_library_status_from_ports() {
        let status = GetApplicationStatus::new(&HealthyDatabase, &NoLibrary)
            .execute()
            .unwrap();

        assert!(status.database_healthy);
        assert!(!status.library_configured);
        assert_eq!(status.platform.os, std::env::consts::OS);
    }
}
